use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::*;
use cw20::BalanceResponse as Cw20BalanceResponse;
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg, Cw20ReceiveMsg};
use yieldpay_core::pool_msg::{
    Cw20HookMsg, NftCallback, NFT_REPLY_COLLECTION_ACTIVE, NFT_REPLY_COLLECTION_REDEEMED,
};
use yieldpay_core::tax::deduct_tax;
use yieldpay_core::token;

use std::ops::{Div, Mul, Sub};
use std::str::FromStr;

use crate::config;
use crate::config::{last_claimed_read, last_claimed_store, LastClaimed};
use crate::error::ContractError;
use crate::querier::anchor;
use crate::querier::nft::{nft_exists, quick_mint_msg, switch_collection_msg};

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Redeem {}) => {
            // only asset contract can execute this message
            let config: config::Config = config::read(deps.storage).unwrap();
            if deps.api.addr_canonicalize(info.sender.as_str()).unwrap() != config.dp_token {
                return Err(ContractError::Unauthorized {
                    action: "receive".to_string(),
                    expected: deps
                        .api
                        .addr_humanize(&config.dp_token)
                        .unwrap()
                        .to_string(),
                    actual: info.sender.to_string(),
                });
            }

            redeem(deps, env, info, cw20_msg.sender, cw20_msg.amount)
        }
        _ => Err(ContractError::NotAllowOtherCw20ReceiveAction {
            action: "redeem".to_string(),
        }),
    }
}

pub fn deposit(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = config::read(deps.storage).unwrap();

    // check deposit
    let received: Uint128 = info
        .funds
        .iter()
        .find(|c| c.denom == config.stable_denom)
        .map(|c| c.amount)
        .unwrap_or_else(Uint128::zero);

    if received.is_zero() {
        return Err(ContractError::NotAllowZeroAmount {});
    }
    if info.funds.len() > 1 {
        return Err(ContractError::NotAllowOtherDenoms {
            denom: config.stable_denom,
        });
    }

    let dp_mint_amount = deduct_tax(
        deps.as_ref(),
        Coin {
            denom: config.stable_denom.clone(),
            amount: received,
        },
    )?
    .amount;

    // If there are NFTs. give them an 'active' one, potentially switching a 'inactive' one if it's there
    let nft_msg = if let Some(nft_contract) = config.nft_contract {
        if let Some(active) = config.nft_collection_active {
            if let Some(redeemed) = config.nft_collection_redeemed {
                let exists_active = nft_exists(deps.as_ref(), &nft_contract, &info.sender, active)?;
                if exists_active.tokens.is_empty() {
                    let exists_redeemed =
                        nft_exists(deps.as_ref(), &nft_contract, &info.sender, redeemed)?;
                    if exists_redeemed.tokens.is_empty() {
                        let mint_msg = quick_mint_msg(
                            &format!("{}/{}", config.pool_name, env.block.height),
                            &info.sender,
                            active,
                        );
                        Some(CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: deps
                                .api
                                .addr_humanize(&nft_contract)
                                .unwrap()
                                .to_string(),
                            msg: to_binary(&mint_msg)?,
                            funds: vec![],
                        }))
                    } else {
                        let token = exists_redeemed.tokens.first().unwrap();
                        let switch_msg = switch_collection_msg(token, active);
                        Some(CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: deps
                                .api
                                .addr_humanize(&nft_contract)
                                .unwrap()
                                .to_string(),
                            msg: to_binary(&switch_msg)?,
                            funds: vec![],
                        }))
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let response = Response::new()
        .add_messages(anchor::deposit_stable_msg(
            deps.as_ref(),
            &config.money_market,
            &config.stable_denom,
            received,
        )?)
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&config.dp_token)
                .unwrap()
                .to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: info.sender.to_string(),
                amount: dp_mint_amount,
            })?,
            funds: vec![],
        }))
        .add_attribute("action", "deposit")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("amount", dp_mint_amount.to_string());

    if let Some(nft_mint) = nft_msg {
        Ok(response.add_message(nft_mint))
    } else {
        Ok(response)
    }
}

pub fn redeem(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    sender: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage).unwrap();

    let sender_addr = deps.api.addr_validate(&sender).unwrap();
    if amount.is_zero() {
        return Err(ContractError::RedeemZero {});
    }

    let epoch_state = anchor::epoch_state(deps.as_ref(), &config.money_market)?;
    if epoch_state.exchange_rate.is_zero() {
        return Err(ContractError::RedeemEpochIsZero {});
    }
    let thousand_x_exchange = epoch_state
        .exchange_rate
        .mul(Decimal256::from_uint256(1000u64));

    let market_redeem_amount = cosmwasm_bignumber::Uint256::from(amount)
        .div(thousand_x_exchange)
        .mul(cosmwasm_bignumber::Uint256::from(1000u64));
    let user_redeem_amount = market_redeem_amount.mul(epoch_state.exchange_rate);
    let adjusted_amount = user_redeem_amount;
    /*
       let user_redeem_amount = deduct_tax(
           deps.as_ref(),
           Coin {
               denom: config.stable_denom.clone(),
               amount: deduct_tax(
                   deps.as_ref(),
                   Coin {
                       denom: config.stable_denom.clone(),
                       amount,
                   },
               )
               .map_err(|o| ContractError::RedeemTaxError { msg: o.to_string() })
               .unwrap()
               .amount,
           },
       )
       .map_err(|o| ContractError::RedeemTaxError { msg: o.to_string() })
       .unwrap();

    */
    let nft_msg = if let Some(nft_contract) = config.nft_contract {
        if let Some(active) = config.nft_collection_active {
            if let Some(redeemed) = config.nft_collection_redeemed {
                let balance_qry_msg = &Cw20QueryMsg::Balance {
                    address: sender.clone(),
                };
                let balance_qry =
                    deps.querier
                        .query::<Cw20BalanceResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
                            contract_addr: deps
                                .api
                                .addr_humanize(&config.dp_token)
                                .unwrap()
                                .to_string(),
                            msg: to_binary(&balance_qry_msg)?,
                        }))?;
                // anything less than 10c is dust
                if balance_qry.balance <= Uint128::from(100_000u64) {
                    let exists_redeemed =
                        nft_exists(deps.as_ref(), &nft_contract, &sender_addr, redeemed)?;
                    // if there is a redeemed token, then don't add another
                    // this might leave a 'active' one, but we don't burn
                    if exists_redeemed.tokens.is_empty() {
                        let exists_active =
                            nft_exists(deps.as_ref(), &nft_contract, &sender_addr, active)?;
                        // this shouldn't really occur.
                        if exists_active.tokens.is_empty() {
                            let mint_msg = quick_mint_msg(
                                &format!("{}/{}-r", config.pool_name, env.block.height),
                                &sender_addr,
                                redeemed,
                            );
                            Some(CosmosMsg::Wasm(WasmMsg::Execute {
                                contract_addr: deps
                                    .api
                                    .addr_humanize(&nft_contract)
                                    .unwrap()
                                    .to_string(),
                                msg: to_binary(&mint_msg)?,
                                funds: vec![],
                            }))
                        } else {
                            let token = exists_active.tokens.first().unwrap();
                            let switch_msg = switch_collection_msg(token, redeemed);
                            Some(CosmosMsg::Wasm(WasmMsg::Execute {
                                contract_addr: deps
                                    .api
                                    .addr_humanize(&nft_contract)
                                    .unwrap()
                                    .to_string(),
                                msg: to_binary(&switch_msg)?,
                                funds: vec![],
                            }))
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    let resp = Response::new()
        .add_messages(anchor::redeem_stable_msg(
            deps.as_ref(),
            &config.money_market,
            &config.atoken,
            market_redeem_amount.into(),
        )?)
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: sender.clone(),
            amount: vec![coin(
                u128::from(user_redeem_amount),
                config.stable_denom.clone(),
            )],
        }))
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&config.dp_token)
                .unwrap()
                .to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Burn {
                amount: Uint128::from(adjusted_amount),
            })?,
            funds: vec![],
        }));

    if let Some(nft_add) = nft_msg {
        Ok(resp
            .add_message(nft_add)
            .add_attribute("action", "redeem")
            .add_attribute("sender", sender)
            .add_attribute("NFT", "added/switched")
            .add_attribute("amount", user_redeem_amount.to_string()))
    } else {
        Ok(resp
            .add_attribute("action", "redeem")
            .add_attribute("sender", sender)
            .add_attribute("NFT", "skipped")
            .add_attribute("amount", user_redeem_amount.to_string()))
    }
}

pub fn earn(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // calculate deduct(total_aust_amount * exchange_rate) - (total_dp_balance)
    let config = config::read(deps.storage).unwrap();
    let sender_canon = deps.api.addr_canonicalize(info.sender.as_str()).unwrap();
    if config.beneficiary != sender_canon && config.fee_collector != sender_canon {
        return Err(ContractError::Unauthorized {
            action: "earn".to_string(),
            expected: deps
                .api
                .addr_humanize(&config.beneficiary)
                .unwrap()
                .to_string(),
            actual: info.sender.to_string(),
        });
    }

    // assets
    let epoch_state = anchor::epoch_state(deps.as_ref(), &config.money_market)?;
    let atoken_balance = token::balance_of(
        deps.as_ref(),
        deps.api.addr_humanize(&config.atoken).unwrap().to_string(),
        env.contract.address.to_string(),
    )?;
    let dp_total_supply: Uint256 = Uint256::from_str(
        &token::total_supply(
            deps.as_ref(),
            deps.api
                .addr_humanize(&config.dp_token)
                .unwrap()
                .to_string(),
        )?
        .to_string(),
    )?;

    let pool_value_locked = Uint256::from(
        deduct_tax(
            deps.as_ref(),
            Coin {
                denom: config.stable_denom.clone(),
                amount: (atoken_balance.mul(epoch_state.exchange_rate)).into(),
            },
        )?
        .amount,
    );
    let earnable = pool_value_locked.sub(dp_total_supply);
    // fee = 0 means use fee_max as a fixed_fee.

    let last_claimed = last_claimed_read(deps.storage).unwrap();
    let (fee, updated_last_claimed) = calc_fee(
        earnable,
        config.fee_amount,
        config.fee_max,
        config.fee_reset_every_num_blocks,
        env.block.height,
        last_claimed,
    );
    last_claimed_store(deps.storage, &updated_last_claimed).unwrap();

    // if there is no fee. then don't do a send to the fee collection.
    if fee.is_zero() {
        Ok(Response::new()
            .add_messages(anchor::redeem_stable_msg(
                deps.as_ref(),
                &config.money_market,
                &config.atoken,
                earnable.div(epoch_state.exchange_rate).into(),
            )?)
            .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: deps
                    .api
                    .addr_humanize(&config.beneficiary)
                    .unwrap()
                    .to_string(),
                amount: vec![deduct_tax(
                    deps.as_ref(),
                    Coin {
                        denom: config.stable_denom.clone(),
                        amount: earnable.sub(fee).into(),
                    },
                )?],
            }))
            .add_attribute("action", "earn")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("amount", earnable.sub(fee).to_string())
            .add_attribute("fee", fee.to_string()))
    } else {
        let fee_minus_one = fee.sub(Uint256::from(1u64));
        Ok(Response::new()
            .add_messages(anchor::redeem_stable_msg(
                deps.as_ref(),
                &config.money_market,
                &config.atoken,
                earnable.div(epoch_state.exchange_rate).into(),
            )?)
            .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: deps
                    .api
                    .addr_humanize(&config.beneficiary)
                    .unwrap()
                    .to_string(),
                amount: vec![deduct_tax(
                    deps.as_ref(),
                    Coin {
                        denom: config.stable_denom.clone(),
                        amount: earnable.sub(fee).into(),
                    },
                )?],
            }))
            .add_message(CosmosMsg::Bank(BankMsg::Send {
                to_address: deps
                    .api
                    .addr_humanize(&config.fee_collector)
                    .unwrap()
                    .to_string(),
                amount: vec![deduct_tax(
                    deps.as_ref(),
                    Coin {
                        denom: config.stable_denom.clone(),
                        amount: fee_minus_one.into(),
                    },
                )?],
            }))
            .add_attribute("action", "earn")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("amount", earnable.sub(fee).to_string())
            .add_attribute("fee", fee_minus_one.to_string()))
    }
}
/// Calculate fee to charge, taking into account monthly caps
/// returns: fee to charge, and fees collected in the current period
pub fn calc_fee(
    earnable: Uint256,
    fee_amount: Decimal256,
    fee_max: Uint256,
    _fee_reset_every_num_blocks: u64,
    _current_block_height: u64,
    last_claimed_current: LastClaimed,
) -> (Uint256, LastClaimed) {
    let mut fee: Uint256;
    if fee_amount > Decimal256::zero() {
        fee = earnable.mul(fee_amount);
        if !fee_max.is_zero() && fee > fee_max {
            fee = fee_max;
        }
    } else {
        fee = fee_max;
    }
    (
        fee,
        LastClaimed {
            last_claimed_at_block_height: last_claimed_current.last_claimed_at_block_height,
            fees_collected: last_claimed_current.fees_collected + fee,
            total_earned_at_last_claimed: last_claimed_current.total_earned_at_last_claimed
                + earnable.sub(fee),
        },
    )
}

pub fn configure(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    beneficiary: Option<String>,
    fee_collector: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage).unwrap();
    let sender_canon = deps.api.addr_canonicalize(info.sender.as_str()).unwrap();
    if config.beneficiary != sender_canon
        && config.fee_collector != sender_canon
        && !(config.owner_can_change_config && config.owner == sender_canon)
    {
        return Err(ContractError::Unauthorized {
            action: "configure".to_string(),
            expected: deps.api.addr_humanize(&config.owner).unwrap().to_string(),
            actual: info.sender.to_string(),
        });
    }

    if let Some(beneficiary) = beneficiary {
        if config.owner == sender_canon || config.beneficiary == sender_canon {
            config.beneficiary = deps.api.addr_canonicalize(beneficiary.as_str()).unwrap();
        } else {
            return Err(ContractError::Unauthorized {
                action: "configure_beneficiary".to_string(),
                expected: deps.api.addr_humanize(&config.owner).unwrap().to_string(),
                actual: info.sender.to_string(),
            });
        }
    }
    if let Some(fee_collector) = fee_collector {
        if config.owner == sender_canon || config.fee_collector == sender_canon {
            config.fee_collector = deps.api.addr_canonicalize(fee_collector.as_str()).unwrap();
        } else {
            return Err(ContractError::Unauthorized {
                action: "configure_fee_collector".to_string(),
                expected: deps.api.addr_humanize(&config.owner).unwrap().to_string(),
                actual: info.sender.to_string(),
            });
        }
    }
    config::store(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn configure_details(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    title: Option<String>,
    description: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage).unwrap();
    let sender_canon = deps.api.addr_canonicalize(info.sender.as_str()).unwrap();
    if config.beneficiary != sender_canon
        && !(config.owner_can_change_config && config.owner == sender_canon)
    {
        return Err(ContractError::Unauthorized {
            action: "configure_details".to_string(),
            expected: deps.api.addr_humanize(&config.owner).unwrap().to_string(),
            actual: info.sender.to_string(),
        });
    }

    if let Some(title) = title {
        config.pool_title = title;
    }
    if let Some(description) = description {
        config.pool_description = description;
    }
    config::store(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn clear_nft_contract(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage).unwrap();
    let sender_canon = deps.api.addr_canonicalize(info.sender.as_str()).unwrap();
    if config.beneficiary != sender_canon
        && !(config.owner == sender_canon && config.owner_can_change_config)
    {
        return Err(ContractError::Unauthorized {
            action: "clear_nft_contract".to_string(),
            expected: deps
                .api
                .addr_humanize(&config.beneficiary)
                .unwrap()
                .to_string(),
            actual: info.sender.to_string(),
        });
    }
    let old_nft_contract = if let Some(c) = config.nft_contract {
        deps.api.addr_humanize(&c)?.to_string()
    } else {
        "-None-".to_string()
    };
    let nft_collection_active = if let Some(c) = config.nft_collection_active {
        format!("{}", c)
    } else {
        "-None-".to_string()
    };
    let nft_collection_redeemed = if let Some(c) = config.nft_collection_redeemed {
        format!("{}", c)
    } else {
        "-None-".to_string()
    };
    config.nft_contract = None;
    config.nft_collection_active = None;
    config.nft_collection_redeemed = None;
    config::store(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "clear_nft_details".to_string())
        .add_attribute("previous_nft_contract", old_nft_contract)
        .add_attribute("previous_active_collection", nft_collection_active)
        .add_attribute("previous_redeemed_collection", nft_collection_redeemed))
}

pub fn set_nft_contract(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    nft_contract: String,
    nft_collection_active: Option<u64>,
    nft_collection_redeemed: Option<u64>,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage).unwrap();
    let sender_canon = deps.api.addr_canonicalize(info.sender.as_str()).unwrap();
    if config.beneficiary != sender_canon {
        if config.owner == sender_canon {
            if !(config.owner_can_change_config || config.nft_contract.is_none()) {
                return Err(ContractError::Unauthorized {
                    action: "clear_nft_contract/owner can't change".to_string(),
                    expected: deps
                        .api
                        .addr_humanize(&config.beneficiary)
                        .unwrap()
                        .to_string(),
                    actual: info.sender.to_string(),
                });
            }
        } else {
            return Err(ContractError::Unauthorized {
                action: "clear_nft_contract".to_string(),
                expected: deps
                    .api
                    .addr_humanize(&config.beneficiary)
                    .unwrap()
                    .to_string(),
                actual: info.sender.to_string(),
            });
        }
    }

    let old_nft_contract = if let Some(c) = config.nft_contract {
        deps.api.addr_humanize(&c)?.to_string()
    } else {
        "-None-".to_string()
    };
    let old_nft_collection_active = if let Some(c) = config.nft_collection_active {
        format!("{}", c)
    } else {
        "-None-".to_string()
    };
    let old_nft_collection_redeemed = if let Some(c) = config.nft_collection_redeemed {
        format!("{}", c)
    } else {
        "-None-".to_string()
    };
    config.nft_contract = Some(deps.api.addr_canonicalize(&nft_contract)?);
    config.nft_collection_active = nft_collection_active;
    config.nft_collection_redeemed = nft_collection_redeemed;
    config::store(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "set_nft_details".to_string())
        .add_attribute("nft_contract", nft_contract)
        .add_attribute(
            "active_collection",
            nft_collection_active.map_or_else(|| "-".to_string(), |f| format!("{}", f)),
        )
        .add_attribute(
            "redeemed_collection",
            nft_collection_redeemed.map_or_else(|| "-".to_string(), |f| format!("{}", f)),
        )
        .add_attribute("previous_nft_contract", old_nft_contract)
        .add_attribute("previous_active_collection", old_nft_collection_active)
        .add_attribute("previous_redeemed_collection", old_nft_collection_redeemed))
}

pub fn set_nft_collection(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    sender: String,
    collection_id: String,
    msg: Binary,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage).unwrap();

    if let Some(ref nft_addr) = config.nft_contract {
        // 1. NFT contract should be directly calling us. so info.sender should be there
        let sender_canon = deps.api.addr_canonicalize(info.sender.as_str()).unwrap();
        if nft_addr != &sender_canon {
            return Err(ContractError::Unauthorized {
                action: "set_nft_collection only can come from the NFT contract".to_string(),
                expected: deps.api.addr_humanize(nft_addr).unwrap().to_string(),
                actual: info.sender.to_string(),
            });
        }
        // 2. we trust the NFT contract to pass us who called them, and that should be the 'owner' of the fund
        let owner_sender = deps.api.addr_canonicalize(&sender).unwrap();
        if owner_sender != config.owner {
            return Err(ContractError::Unauthorized {
                action: "set_nft_collection only can originate from the fund owner".to_string(),
                expected: deps.api.addr_humanize(&config.owner).unwrap().to_string(),
                actual: sender,
            });
        }

        let new_collection_id: u64 = collection_id.parse::<u64>()?;
        let callback: NftCallback = from_binary(&msg)?;

        match callback.active_redeemed {
            NFT_REPLY_COLLECTION_ACTIVE => {
                config.nft_collection_active = Some(new_collection_id);
            }
            NFT_REPLY_COLLECTION_REDEEMED => {
                config.nft_collection_redeemed = Some(new_collection_id)
            }
            _ => {
                return Err(ContractError::NftCollectionInvalidOption(
                    new_collection_id,
                    callback.active_redeemed,
                ))
            }
        }

        config::store(deps.storage, &config)?;

        Ok(Response::new()
            .add_attribute("action", "set_nft_collection".to_string())
            .add_attribute("nft_contract", info.sender)
            .add_attribute("factory", sender)
            .add_attribute(
                "active_collection",
                config
                    .nft_collection_active
                    .map_or_else(|| "-".to_string(), |f| format!("{}", f)),
            )
            .add_attribute(
                "redeemed_collection",
                config
                    .nft_collection_redeemed
                    .map_or_else(|| "-".to_string(), |f| format!("{}", f)),
            ))
    } else {
        Err(ContractError::NftContractInvalid)
    }
}

pub fn register_dp_token(
    deps: DepsMut,
    _env: Env,
    address: Addr,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage).unwrap();
    if config.dp_token != CanonicalAddr::from(vec![]) {
        return Err(ContractError::Unauthorized {
            action: "register_dp_token".to_string(),
            expected: "<empty>".to_string(),
            actual: deps
                .api
                .addr_humanize(&config.dp_token)
                .unwrap()
                .to_string(),
        });
    }

    config.dp_token = deps.api.addr_canonicalize(address.as_str()).unwrap();
    config::store(deps.storage, &config)?;

    Ok(Response::new().add_attribute("dp_token", address.to_string()))
}
