use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub pool_name: String,
    pub pool_title: String,
    pub pool_description: String,
    pub beneficiary: String,
    pub fee_collector: String,
    pub owner: String,
    pub money_market: String,
    pub stable_denom: String,
    pub anchor_token: String,
    pub dp_token: String,
    pub owner_can_change_config: bool,
    pub nft_contract: Option<String>,
    pub nft_collection_active: Option<u64>,
    pub nft_collection_redeemed: Option<u64>,
}
