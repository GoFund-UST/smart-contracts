use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Coin, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositAmountResponse {
    pub amount: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TotalDepositAmountResponse {
    pub amount: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimableRewardResponse {
    pub total_value: Uint256,
    pub pool_value: Uint256,
    pub earned: Uint256,
    pub claimable: Uint256,
    pub fee: Uint256,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FeeResponse {
    pub fee_amount: Decimal256,
    pub fee_max: Uint256,
    pub fee_reset_every_num_blocks: u64,
    pub fee: Uint256,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RedeemResponse {
    pub burn_amount: Uint128,
    pub market_redeem_amount: Uint256,
    pub user_redeem_amount: Coin,
}
