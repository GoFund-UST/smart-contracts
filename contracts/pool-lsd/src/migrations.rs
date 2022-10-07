use crate::config::Config;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::singleton_read;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const CONFIG_V100: &[u8] = b"config";
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigV100 {
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
    pub nft_contract: Option<String>,
    pub nft_collection_active: Option<u64>,
    pub nft_collection_redeemed: Option<u64>,
}
impl ConfigV100 {
    pub fn load(storage: &dyn Storage) -> StdResult<Self> {
        singleton_read(storage, CONFIG_V100).load()
    }

    pub fn migrate_from(&self) -> Config {
        Config {
            this: self.this.clone(),
            owner: self.owner.clone(),
            beneficiary: self.beneficiary.clone(),
            fee_collector: self.fee_collector.clone(),
            fee_amount: self.fee_amount,
            fee_max: self.fee_max,
            fee_reset_every_num_blocks: self.fee_reset_every_num_blocks,
            money_market: self.money_market.clone(),
            atoken: self.atoken.clone(),
            stable_denom: self.stable_denom.clone(),
            dp_token: self.dp_token.clone(),
            pool_name: self.pool_name.clone(),
            pool_title: "".to_string(),
            pool_description: "".to_string(),
            owner_can_change_config: false,
            nft_contract: None,
            nft_collection_active: None,
            nft_collection_redeemed: None,
        }
    }
}
