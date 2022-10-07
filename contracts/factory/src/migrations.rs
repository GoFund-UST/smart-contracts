use crate::config::Config;
use cosmwasm_std::{CanonicalAddr, Decimal, StdResult, Storage, Uint128};
use cosmwasm_storage::singleton_read;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const CONFIG_V100: &[u8] = b"config_001";

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct ConfigV100 {
    pub this: CanonicalAddr,
    pub owner: CanonicalAddr,
    pub fee_collector: CanonicalAddr,
    pub fee_amount: Decimal,
    pub fee_max: Uint128,
    pub fee_reset_every_num_blocks: u64,
    pub money_market: CanonicalAddr,
    pub dp_code_id: u64,
    pub anchor_pool_code_id: u64,
}
impl ConfigV100 {
    pub fn load(storage: &dyn Storage) -> StdResult<Self> {
        singleton_read(storage, CONFIG_V100).load()
    }

    pub fn migrate_from(&self) -> Config {
        Config {
            this: self.this.clone(),
            owner: self.owner.clone(),
            fee_collector: self.fee_collector.clone(),
            fee_amount: self.fee_amount,
            fee_max: self.fee_max,
            fee_reset_every_num_blocks: self.fee_reset_every_num_blocks,
            money_market: self.money_market.clone(),
            dp_code_id: self.dp_code_id,
            anchor_pool_code_id: self.anchor_pool_code_id,
            nft_code_id: None,
            nft_instantiate: None,
            homepage: None,
            nft_contract: None,
        }
    }
}
