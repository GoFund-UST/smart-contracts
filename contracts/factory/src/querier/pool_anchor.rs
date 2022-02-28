use cosmwasm_std::*;
use gofund_ust_core::pool_anchor_response::ConfigResponse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
}

pub fn pool_anchor_config(deps: Deps, pool_anchor_contract: &Addr) -> StdResult<ConfigResponse> {
    let pool_config: ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: pool_anchor_contract.to_string(),
            msg: to_binary(&QueryMsg::Config {})?,
        }))?;

    Ok(pool_config)
}
