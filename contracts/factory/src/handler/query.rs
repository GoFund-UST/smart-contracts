use cosmwasm_std::*;
use cw_storage_plus::Bound;
use yieldpay_core::factory_response::{
    AnchorPool, ConfigResponse, FundsCountResponse, FundsResponse,
};
use yieldpay_core::pool_anchor_response;
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

use crate::config;
use crate::querier::pool_anchor::pool_anchor_config;
use crate::state::anchor_pools;

pub fn config(deps: Deps, _env: Env) -> StdResult<Binary> {
    let config: config::Config = config::read(deps.storage).unwrap();
    let nft_contract = if let Some(nft_canonical_addr) = config.nft_contract {
        let addr = deps.api.addr_humanize(&nft_canonical_addr).unwrap();
        Some(addr.to_string())
    } else {
        None
    };
    to_binary(&ConfigResponse {
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
        dp_code_id: config.dp_code_id,
        fee_amount: config.fee_amount.to_string(),
        fee_max: config.fee_max.to_string(),
        fee_reset_every_num_blocks: config.fee_reset_every_num_blocks,
        anchor_pool_code_id: config.anchor_pool_code_id,
        nft_code_id: config.nft_code_id,
        nft_instantiate: config.nft_instantiate,
        nft_contract,
        homepage: config.homepage,
    })
}
pub fn anchor_fund(deps: Deps, _env: Env, contract: &str) -> StdResult<Option<AnchorPool>> {
    let addr = deps.api.addr_validate(contract)?;
    anchor_pools().may_load(deps.storage, addr.to_string())
}

pub fn anchor_fund_ex(
    deps: Deps,
    _env: Env,
    contract: &str,
) -> StdResult<pool_anchor_response::ConfigResponse> {
    let addr = deps.api.addr_validate(contract)?;
    let pool_config = pool_anchor_config(deps, &addr)?;
    Ok(pool_config)
}

pub fn all_anchor_funds(
    deps: Deps,
    _env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<FundsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    //  let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_after.map(Bound::exclusive);
    Ok(FundsResponse {
        funds: anchor_pools()
            .range(deps.storage, start, None, Order::Ascending)
            .filter(|f| match f {
                Ok((_, ap)) => ap.open,
                Err(_) => false,
            })
            .take(limit)
            .map(|item| item.map(|(_, v)| v))
            .collect::<StdResult<Vec<AnchorPool>>>()?,
    })
}
pub fn all_anchor_fund_count(deps: Deps, _env: Env) -> StdResult<FundsCountResponse> {
    let i_map = anchor_pools();
    let len = i_map
        .range(deps.storage, None, None, Order::Ascending)
        .count();
    Ok(FundsCountResponse { count: len })
}
pub fn anchor_funds_by_beneficiary(
    deps: Deps,
    _env: Env,
    beneficiary: &str,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<FundsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let beneficiary_addr = deps.api.addr_validate(beneficiary)?.to_string();
    //let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_after.map(Bound::exclusive);
    Ok(FundsResponse {
        funds: anchor_pools()
            .idx
            .beneficiary
            .prefix(beneficiary_addr)
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| item.map(|(_, v)| v))
            .collect::<StdResult<Vec<AnchorPool>>>()?,
    })
}
pub fn anchor_funds_by_owner(
    deps: Deps,
    _env: Env,
    owner: &str,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<FundsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let owner_addr = deps.api.addr_validate(owner)?.to_string();
    let start = start_after.map(Bound::exclusive);
    Ok(FundsResponse {
        funds: anchor_pools()
            .idx
            .owner
            .prefix(owner_addr)
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| item.map(|(_, v)| v))
            .collect::<StdResult<Vec<AnchorPool>>>()?,
    })
}

pub fn anchor_funds_by_pool_name(
    deps: Deps,
    _env: Env,
    pool_name: &str,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<FundsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let start = start_after.map(Bound::exclusive);
    Ok(FundsResponse {
        funds: anchor_pools()
            .idx
            .pool
            .prefix(pool_name.parse().unwrap())
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| item.map(|(_, v)| v))
            .collect::<StdResult<Vec<AnchorPool>>>()?,
    })
}
