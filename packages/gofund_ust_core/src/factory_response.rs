use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub fee_collector: String,
    pub money_market: String,
    pub dp_code_id: u64,
    pub fee_amount: String,
    pub fee_max: String,
    pub fee_reset_every_num_blocks: u64,
    pub anchor_pool_code_id: u64,
    pub nft_code_id: Option<u64>,
    pub nft_instantiate: Option<String>,
    pub nft_contract: Option<String>,
    pub homepage: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AnchorPool {
    pub contract: String,
    pub owner: String,
    pub beneficiary: String,
    pub pool_name: String,
    pub open: bool,
    pub active_collection: Option<u64>,
    pub redeemed_collection: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct FundsResponse {
    pub funds: Vec<AnchorPool>,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct FundsCountResponse {
    pub count: usize,
}
