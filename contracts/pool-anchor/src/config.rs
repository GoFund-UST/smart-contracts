use cosmwasm_bignumber::{Decimal256, Uint256};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};

pub static CONFIG_KEY: &[u8] = b"config_v104";
pub static LAST_CLAIMED_KEY: &[u8] = b"last_claimed";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub this: CanonicalAddr,
    pub owner: CanonicalAddr,
    pub beneficiary: CanonicalAddr,
    pub fee_collector: CanonicalAddr,
    pub fee_amount: Decimal256,
    pub fee_max: Uint256,
    pub fee_reset_every_num_blocks: u64,
    pub money_market: CanonicalAddr,
    pub atoken: CanonicalAddr,
    pub stable_denom: String,
    pub dp_token: CanonicalAddr,
    pub pool_name: String,
    pub pool_title: String,
    pub pool_description: String,
    pub owner_can_change_config: bool,
    pub nft_contract: Option<CanonicalAddr>,
    pub nft_collection_active: Option<u64>,
    pub nft_collection_redeemed: Option<u64>,
}

pub fn store(storage: &mut dyn Storage, data: &Config) -> StdResult<()> {
    singleton(storage, CONFIG_KEY).save(data)
}

pub fn read(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, CONFIG_KEY).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LastClaimed {
    pub last_claimed_at_block_height: u64,
    pub fees_collected: Uint256,
    pub total_earned_at_last_claimed: Uint256,
}

pub fn last_claimed_store(storage: &mut dyn Storage, data: &LastClaimed) -> StdResult<()> {
    singleton(storage, LAST_CLAIMED_KEY).save(data)
}

pub fn last_claimed_read(storage: &dyn Storage) -> StdResult<LastClaimed> {
    singleton_read(storage, LAST_CLAIMED_KEY).load()
}
