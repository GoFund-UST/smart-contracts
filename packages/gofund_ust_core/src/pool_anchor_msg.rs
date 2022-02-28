use cosmwasm_bignumber::Uint256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub pool_name: String,
    pub pool_title: String,
    pub pool_description: String,
    pub beneficiary: String,
    pub fee_collector: String,
    pub fee_amount: String,
    pub fee_max: Uint256,
    pub fee_reset_every_num_blocks: u64,
    pub money_market: String,
    pub dp_code_id: u64,
    pub owner_can_change_config: bool,
    pub nft_contract: Option<String>,
    pub nft_collection_active: Option<u64>,
    pub nft_collection_redeemed: Option<u64>,
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
