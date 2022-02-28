use cosmwasm_bignumber::{Decimal256, Uint256};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};

pub static CONFIG_KEY: &[u8] = b"config_003";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub this: CanonicalAddr,
    pub owner: CanonicalAddr,
    pub fee_collector: CanonicalAddr,
    pub fee_amount: Decimal256,
    pub fee_max: Uint256,
    pub fee_reset_every_num_blocks: u64,
    pub money_market: CanonicalAddr,
    pub dp_code_id: u64,
    pub anchor_pool_code_id: u64,
    pub nft_code_id: Option<u64>,
    pub nft_instantiate: Option<String>,
    pub nft_contract: Option<CanonicalAddr>,
    pub homepage: Option<String>,
}

pub fn store(storage: &mut dyn Storage, data: &Config) -> StdResult<()> {
    singleton(storage, CONFIG_KEY).save(data)
}

pub fn read(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, CONFIG_KEY).load()
}
/*
//pub static TEMP_FUND_CONTRACT_ACTIVE: &[u8] = b"temp_fund_contract_active";
//pub static TEMP_FUND_CONTRACT_REDEEM: &[u8] = b"temp_fund_contract_redeem";

pub fn store_contract_active(storage: &mut dyn Storage, data: &String) -> StdResult<()> {
    singleton(storage, TEMP_FUND_CONTRACT_ACTIVE).save(data)
}

pub fn read_contract_active(storage: &dyn Storage) -> StdResult<String> {
    singleton_read(storage, TEMP_FUND_CONTRACT_ACTIVE).load()
}

pub fn store_contract_redeem(storage: &mut dyn Storage, data: &String) -> StdResult<()> {
    singleton(storage, TEMP_FUND_CONTRACT_REDEEM).save(data)
}

pub fn read_contract_redeem(storage: &dyn Storage) -> StdResult<String> {
    singleton_read(storage, TEMP_FUND_CONTRACT_REDEEM).load()
}
*/
