use cosmwasm_std::{Coin, Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct DepositAmountResponse {
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct TotalDepositAmountResponse {
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct ClaimableRewardResponse {
    pub total_value: Uint128,
    pub pool_value: Uint128,
    pub earned: Uint128,
    pub claimable: Uint128,
    pub fee: Uint128,
}
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct FeeResponse {
    pub fee_amount: Decimal,
    pub fee_max: Uint128,
    pub fee_reset_every_num_blocks: u64,
    pub fee: Uint128,
}
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct RedeemResponse {
    pub burn_amount: Uint128,
    pub market_redeem_amount: Uint128,
    pub user_redeem_amount: Coin,
}
