use cosmwasm_bignumber::Decimal256;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use std::str::FromStr;
/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "gofundust-pool-anchor";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

use cosmwasm_std::{
    to_binary, Addr, Binary, CanonicalAddr, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn,
    Response, StdError, StdResult, SubMsg, WasmMsg,
};

use cw2::{get_contract_version, set_contract_version};
use cw20::MinterResponse;
use gofund_ust_core::pool_anchor_msg::{InstantiateMsg, MigrateMsg};
use gofund_ust_core::pool_msg::{ExecuteMsg, QueryMsg};
use protobuf::Message;
use terraswap::token::InstantiateMsg as Cw20InstantiateMsg;

#[allow(unused_imports)]
use crate::config::{last_claimed_store, read, LastClaimed};
use crate::error::ContractError;
use crate::handler::core as CoreHandler;
use crate::handler::query as QueryHandler;
use crate::migrations::ConfigV100;
use crate::response::MsgInstantiateContractResponse;
use crate::{config, querier};

const INSTANTIATE_REPLY_ID: u64 = 1;

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let symbol_name = msg.pool_name.replace(' ', "");
    if symbol_name.len() > 9 {
        return Err(ContractError::PoolNameTooLarge);
    }

    let nft_contract_addr = if let Some(contract) = msg.nft_contract {
        if deps.api.addr_validate(&contract).is_err() {
            return Err(ContractError::NftContractInvalid);
        }

        Some(deps.api.addr_canonicalize(&contract)?)
    } else {
        None
    };
    let mut config = config::Config {
        this: deps.api.addr_canonicalize(env.contract.address.as_str())?,
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        beneficiary: deps.api.addr_canonicalize(msg.beneficiary.as_str())?,
        fee_collector: deps.api.addr_canonicalize(msg.fee_collector.as_str())?,
        fee_amount: Decimal256::from_str(&msg.fee_amount)?,
        fee_max: msg.fee_max,
        fee_reset_every_num_blocks: msg.fee_reset_every_num_blocks,
        money_market: deps.api.addr_canonicalize(msg.money_market.as_str())?,
        stable_denom: String::default(),
        atoken: CanonicalAddr::from(vec![]),
        dp_token: CanonicalAddr::from(vec![]),
        pool_title: msg.pool_title,
        pool_name: msg.pool_name.clone(),
        pool_description: msg.pool_description,
        owner_can_change_config: msg.owner_can_change_config,

        nft_contract: nft_contract_addr,
        nft_collection_active: msg.nft_collection_active,
        nft_collection_redeemed: msg.nft_collection_redeemed,
    };

    let market_config =
        querier::anchor::config(deps.as_ref(), &config.money_market).map_err(|_o| {
            ContractError::InstantiateError {
                action: "query anchor".to_string(),
            }
        })?;

    config.stable_denom = market_config.stable_denom.clone();
    config.atoken = deps
        .api
        .addr_canonicalize(market_config.aterra_contract.as_str())
        .map_err(|_o| ContractError::InstantiateError {
            action: "aterra_contract".to_string(),
        })?;

    config::store(deps.storage, &config)?;

    let last_claimed = LastClaimed {
        last_claimed_at_block_height: env.block.height,
        fees_collected: Default::default(),
        total_earned_at_last_claimed: Default::default(),
    };
    last_claimed_store(deps.storage, &last_claimed)?;

    Ok(Response::new().add_submessage(SubMsg {
        // Create Deposit token
        msg: WasmMsg::Instantiate {
            admin: None,
            code_id: msg.dp_code_id,
            funds: vec![],
            label: "GoFund US(T) Deposit Token".to_string(),
            msg: to_binary(&Cw20InstantiateMsg {
                name: format!("GoFundUST Deposit Token - {}", msg.pool_name),
                symbol: format!("gf-{}", symbol_name),
                decimals: 6,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.to_string(),
                    cap: None,
                }),
            })
            .map_err(|_o| ContractError::InstantiateError {
                action: "dp_token".to_string(),
            })?,
        }
        .into(),
        gas_limit: None,
        id: INSTANTIATE_REPLY_ID,
        reply_on: ReplyOn::Success,
    }))
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => CoreHandler::receive(deps, env, info, msg),
        ExecuteMsg::Deposit {} => CoreHandler::deposit(deps, env, info),

        ExecuteMsg::Earn {} => CoreHandler::earn(deps, env, info),
        ExecuteMsg::Configure {
            beneficiary,
            fee_collector,
        } => CoreHandler::configure(deps, env, info, beneficiary, fee_collector),
        ExecuteMsg::ConfigDetails { title, description } => {
            CoreHandler::configure_details(deps, env, info, title, description)
        }
        ExecuteMsg::SetNftContract {
            nft_contract,
            nft_collection_active,
            nft_collection_redeemed,
        } => CoreHandler::set_nft_contract(
            deps,
            env,
            info,
            nft_contract,
            nft_collection_active,
            nft_collection_redeemed,
        ),
        ExecuteMsg::ClearNftContract => CoreHandler::clear_nft_contract(deps, env, info),
        ExecuteMsg::CollectablesNew {
            sender,
            collection_id,
            msg,
        } => CoreHandler::set_nft_collection(deps, env, info, sender, collection_id, msg),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        INSTANTIATE_REPLY_ID => {
            // get new token's contract address
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(
                msg.result.unwrap().data.unwrap().as_slice(),
            )
            .map_err(|_| {
                ContractError::Std(StdError::parse_err(
                    "MsgInstantiateContractResponse",
                    "failed to parse data",
                ))
            })?;
            let token_addr = Addr::unchecked(res.get_contract_address());

            CoreHandler::register_dp_token(deps, env, token_addr)
        }
        _ => Err(ContractError::InvalidReplyId { id: msg.id }),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::DepositAmountOf { owner } => QueryHandler::deposit_amount(deps, env, owner), // dp_token.balanceOf(msg.sender)
        QueryMsg::TotalDepositAmount {} => QueryHandler::total_deposit_amount(deps, env), // dp_token.totalSupply()
        QueryMsg::Config {} => QueryHandler::config(deps, env),                           // config
        QueryMsg::Claimable {} => QueryHandler::claimable(deps, env), // config.strategy.reward()
        QueryMsg::LastClaimed {} => QueryHandler::last_claimed(deps, env),
        QueryMsg::Fee {} => QueryHandler::fee(deps, env),
        //QueryMsg::DebugRedeem { owner, amount } =>             QueryHandler::debug_redeem(deps, env, owner, Uin256::from(amount)),
        /*
        QueryMsg::DebugAnchorEpoch {} => QueryHandler::debug_anchor_epoch_state(deps, env),
        QueryMsg::DebugATokenBalance {} => QueryHandler::debug_atoken_balance(deps, env),
        QueryMsg::DebugDPTotalSupply {} => QueryHandler::debug_dp_total_supply(deps, env),
        QueryMsg::DebugPoolValueLocked {} => QueryHandler::debug_pool_value_locked(deps, env),
        QueryMsg::DebugEarnable {} => QueryHandler::debug_earnable(deps, env),
           */
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let contract_version = get_contract_version(deps.storage)?;

    match contract_version.contract.as_ref() {
        #[allow(clippy::single_match)]
        "gofundust-pool-anchor" => match contract_version.version.as_ref() {
            "0.1.1" => {
                let config_v100 = ConfigV100::load(deps.storage)?;

                config::store(deps.storage, &config_v100.migrate_from())?;
            }
            _ => (),
        },
        _ => {
            return Err(ContractError::MigrationError {
                current_name: contract_version.contract,
                current_version: contract_version.version,
            })
        }
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("previous_contract_name", &contract_version.contract)
        .add_attribute("previous_contract_version", &contract_version.version)
        .add_attribute("new_contract_name", CONTRACT_NAME)
        .add_attribute("new_contract_version", CONTRACT_VERSION))
}
