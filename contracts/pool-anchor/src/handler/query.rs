use cosmwasm_bignumber::Uint256;
use cosmwasm_std::*;
//use gofund_ust_core::pool_resp as resp;
use gofund_ust_core::pool_resp::{
    ClaimableRewardResponse, DepositAmountResponse, FeeResponse, TotalDepositAmountResponse,
};

use gofund_ust_core::tax::deduct_tax;
use gofund_ust_core::token;
use std::ops::{Mul, Sub};

use crate::config;
use crate::config::last_claimed_read;
use crate::handler::core::calc_fee;
use crate::querier::anchor;

pub fn deposit_amount(deps: Deps, _env: Env, owner: String) -> StdResult<Binary> {
    let config: config::Config = config::read(deps.storage).unwrap();

    to_binary(&DepositAmountResponse {
        amount: token::balance_of(
            deps,
            deps.api
                .addr_humanize(&config.dp_token)
                .unwrap()
                .to_string(),
            owner,
        )?,
    })
}

pub fn total_deposit_amount(deps: Deps, _env: Env) -> StdResult<Binary> {
    let config: config::Config = config::read(deps.storage).unwrap();

    to_binary(&TotalDepositAmountResponse {
        amount: token::total_supply(
            deps,
            deps.api
                .addr_humanize(&config.dp_token)
                .unwrap()
                .to_string(),
        )?,
    })
}

pub fn config(deps: Deps, _env: Env) -> StdResult<Binary> {
    let config: config::Config = config::read(deps.storage).unwrap();
    let dp_token_str = if config.dp_token == CanonicalAddr::from(vec![]) {
        "".to_string()
    } else {
        deps.api
            .addr_humanize(&config.dp_token)
            .unwrap()
            .to_string()
    };
    let nft_contract = if let Some(contract) = config.nft_contract {
        Some(deps.api.addr_humanize(&contract)?.to_string())
    } else {
        None
    };
    to_binary(&gofund_ust_core::pool_anchor_response::ConfigResponse {
        pool_name: config.pool_name,
        pool_title: config.pool_title,
        pool_description: config.pool_description,
        beneficiary: deps
            .api
            .addr_humanize(&config.beneficiary)
            .unwrap()
            .to_string(),
        fee_collector: deps
            .api
            .addr_humanize(&config.fee_collector)
            .unwrap()
            .to_string(),
        money_market: deps
            .api
            .addr_humanize(&config.money_market)
            .unwrap()
            .to_string(),
        stable_denom: config.stable_denom,
        anchor_token: deps.api.addr_humanize(&config.atoken).unwrap().to_string(),
        dp_token: dp_token_str,
        owner_can_change_config: config.owner_can_change_config,
        owner: deps.api.addr_humanize(&config.owner).unwrap().to_string(),

        nft_contract,
        nft_collection_active: config.nft_collection_active,
        nft_collection_redeemed: config.nft_collection_redeemed,
    })
}
#[allow(dead_code)]
pub fn debug_anchor_epoch_state(deps: Deps, _env: Env) -> StdResult<Binary> {
    let config = config::read(deps.storage).unwrap();

    // assets
    let epoch_state = anchor::epoch_state(deps, &config.money_market)?;
    to_binary(&epoch_state)
}

#[allow(dead_code)]
pub fn debug_atoken_balance(deps: Deps, env: Env) -> StdResult<Binary> {
    let config = config::read(deps.storage).unwrap();

    // assets
    let _epoch_state = anchor::epoch_state(deps, &config.money_market)?;
    let atoken_balance = token::balance_of(
        deps,
        deps.api.addr_humanize(&config.atoken).unwrap().to_string(),
        env.contract.address.to_string(),
    )?;
    to_binary(&atoken_balance)
}

#[allow(dead_code)]
pub fn debug_dp_total_supply(deps: Deps, env: Env) -> StdResult<Binary> {
    let config = config::read(deps.storage).unwrap();

    // assets
    let _epoch_state = anchor::epoch_state(deps, &config.money_market)?;
    let _atoken_balance = token::balance_of(
        deps,
        deps.api.addr_humanize(&config.atoken).unwrap().to_string(),
        env.contract.address.to_string(),
    )?;
    let dp_total_supply = token::total_supply(
        deps,
        deps.api
            .addr_humanize(&config.dp_token)
            .unwrap()
            .to_string(),
    )?;
    to_binary(&dp_total_supply)
}

#[allow(dead_code)]
pub fn debug_pool_value_locked(deps: Deps, env: Env) -> StdResult<Binary> {
    let config = config::read(deps.storage).unwrap();

    // assets
    let epoch_state = anchor::epoch_state(deps, &config.money_market)?;
    let atoken_balance = token::balance_of(
        deps,
        deps.api.addr_humanize(&config.atoken).unwrap().to_string(),
        env.contract.address.to_string(),
    )?;
    let _dp_total_supply = token::total_supply(
        deps,
        deps.api
            .addr_humanize(&config.dp_token)
            .unwrap()
            .to_string(),
    )?;

    let pool_value_locked = Uint256::from(
        deduct_tax(
            deps,
            Coin {
                denom: config.stable_denom,
                amount: (atoken_balance.mul(epoch_state.exchange_rate)).into(),
            },
        )?
        .amount,
    );
    to_binary(&pool_value_locked)
}

#[allow(dead_code)]
pub fn debug_earnable(deps: Deps, env: Env) -> StdResult<Binary> {
    let config = config::read(deps.storage).unwrap();

    // assets
    let epoch_state = anchor::epoch_state(deps, &config.money_market)?;
    let atoken_balance = token::balance_of(
        deps,
        deps.api.addr_humanize(&config.atoken).unwrap().to_string(),
        env.contract.address.to_string(),
    )?;
    let dp_total_supply = token::total_supply(
        deps,
        deps.api
            .addr_humanize(&config.dp_token)
            .unwrap()
            .to_string(),
    )?;

    let pool_value_locked = Uint256::from(
        deduct_tax(
            deps,
            Coin {
                denom: config.stable_denom,
                amount: (atoken_balance.mul(epoch_state.exchange_rate)).into(),
            },
        )?
        .amount,
    );
    let earnable = if dp_total_supply >= pool_value_locked {
        Uint256::zero()
    } else {
        pool_value_locked.sub(dp_total_supply)
    };

    to_binary(&earnable)
}
/*
pub fn debug_redeem(deps: Deps, _env: Env, _sender: String, amount: Uint128) -> StdResult<Binary> {
    let config = config::read(deps.storage).unwrap();

    if amount.is_zero() {
        return to_binary("amount zero");
    }
    let epoch_state = anchor::epoch_state(deps, &config.money_market)?;
    if epoch_state.exchange_rate.is_zero() {
        return to_binary("exchange rate zero");
    }
    let market_redeem_amount = Uint256::from(amount).div(epoch_state.exchange_rate);

    let user_redeem_amount = deduct_tax(
        deps,
        Coin {
            denom: config.stable_denom.clone(),
            amount: deduct_tax(
                deps,
                Coin {
                    denom: config.stable_denom.clone(),
                    amount,
                },
            )
            .unwrap()
            .amount,
        },
    )
    .unwrap();

    to_binary(&RedeemResponse {
        burn_amount: amount,
        user_redeem_amount,
        market_redeem_amount,
    })
}
*/
pub fn last_claimed(deps: Deps, _env: Env) -> StdResult<Binary> {
    let last_claimed = last_claimed_read(deps.storage).unwrap();

    to_binary(&last_claimed)
}

pub fn fee(deps: Deps, env: Env) -> StdResult<Binary> {
    let config = config::read(deps.storage).unwrap();

    // assets
    let epoch_state = anchor::epoch_state(deps, &config.money_market)?;
    let atoken_balance = token::balance_of(
        deps,
        deps.api.addr_humanize(&config.atoken).unwrap().to_string(),
        env.contract.address.to_string(),
    )?;
    let dp_total_supply = token::total_supply(
        deps,
        deps.api
            .addr_humanize(&config.dp_token)
            .unwrap()
            .to_string(),
    )?;

    let pool_value_locked = Uint256::from(
        deduct_tax(
            deps,
            Coin {
                denom: config.stable_denom,
                amount: (atoken_balance.mul(epoch_state.exchange_rate)).into(),
            },
        )?
        .amount,
    );
    // let earnable = pool_value_locked.sub(dp_total_supply);
    let earnable = if dp_total_supply >= pool_value_locked {
        Uint256::zero()
    } else {
        pool_value_locked.sub(dp_total_supply)
    };
    let last_claimed = last_claimed_read(deps.storage).unwrap();

    let (fee, _updated_last_claimed) = calc_fee(
        earnable,
        config.fee_amount,
        config.fee_max,
        config.fee_reset_every_num_blocks,
        env.block.height,
        last_claimed,
    );
    to_binary(&FeeResponse {
        fee_amount: config.fee_amount,
        fee_max: config.fee_max,
        fee_reset_every_num_blocks: config.fee_reset_every_num_blocks,
        fee,
    })
}

pub fn claimable(deps: Deps, env: Env) -> StdResult<Binary> {
    let config = config::read(deps.storage).unwrap();

    // assets
    let epoch_state = anchor::epoch_state(deps, &config.money_market)?;
    let atoken_balance = token::balance_of(
        deps,
        deps.api.addr_humanize(&config.atoken).unwrap().to_string(),
        env.contract.address.to_string(),
    )?;
    let dp_total_supply = token::total_supply(
        deps,
        deps.api
            .addr_humanize(&config.dp_token)
            .unwrap()
            .to_string(),
    )?;

    let pool_value_locked = Uint256::from(
        deduct_tax(
            deps,
            Coin {
                denom: config.stable_denom,
                amount: (atoken_balance.mul(epoch_state.exchange_rate)).into(),
            },
        )?
        .amount,
    );
    let earnable = if dp_total_supply >= pool_value_locked {
        Uint256::zero()
    } else {
        pool_value_locked.sub(dp_total_supply)
    };

    let last_claimed = last_claimed_read(deps.storage).unwrap();

    let (fee, updated_last_claimed) = calc_fee(
        earnable,
        config.fee_amount,
        config.fee_max,
        config.fee_reset_every_num_blocks,
        env.block.height,
        last_claimed,
    );
    let claimable = if fee.is_zero() {
        earnable
    } else if earnable > fee {
        earnable.sub(fee)
    } else {
        Uint256::zero()
    };
    to_binary(&ClaimableRewardResponse {
        total_value: dp_total_supply,
        pool_value: pool_value_locked,
        earned: updated_last_claimed.total_earned_at_last_claimed,
        claimable,
        fee,
    })
}
