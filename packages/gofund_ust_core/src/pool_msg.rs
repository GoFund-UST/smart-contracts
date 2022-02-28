use cosmwasm_std::{to_binary, Binary, CosmosMsg, StdResult, WasmMsg};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    Deposit {}, // UST -> DP (user)
    Earn {},    // x -> UST (beneficiary)
    Configure {
        beneficiary: Option<String>,
        fee_collector: Option<String>,
    },
    ConfigDetails {
        title: Option<String>,
        description: Option<String>,
    },
    /// can be used to set/change details. beneficiary can set this. owner can set if it is blank.
    SetNftContract {
        nft_contract: String,
        nft_collection_active: Option<u64>,
        nft_collection_redeemed: Option<u64>,
    },
    /// can be used to clear the NFT details. intent is if there is an error in NFT, it can be wiped out
    /// beneficiary/owner can exec this
    ClearNftContract,
    // message sent by NFT contract
    CollectablesNew {
        sender: String,
        collection_id: String,
        msg: Binary,
    },
}
impl ExecuteMsg {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        //  let msg = CollectablesExecuteMsg(self);
        to_binary(&self)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg<T: Into<String>, C>(self, contract_addr: T) -> StdResult<CosmosMsg<C>>
    where
        C: Clone + std::fmt::Debug + PartialEq + JsonSchema,
    {
        let msg = self.into_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr: contract_addr.into(),
            msg,
            funds: vec![],
        };
        Ok(execute.into())
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Redeem {},
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    DepositAmountOf { owner: String }, // -> Uint128
    TotalDepositAmount {},             // -> Uint128
    Config {},                         // -> Config
    Claimable {},                      // -> Uint256
    LastClaimed {},                    // -> LastClaimed
    Fee {},                            // -> Uint256

                                       // DebugRedeem { owner: String, amount: u64 }, // -> Uint256
                                       /*
                                       DebugAnchorEpoch {},     // -> Uint128
                                       DebugATokenBalance {},   // -> Uint128
                                       DebugDPTotalSupply {},   // -> Uint128
                                       DebugPoolValueLocked {}, // -> Uint128
                                       DebugEarnable {},        // -> Uint128

                                                               */
}

pub const NFT_REPLY_COLLECTION_ACTIVE: u64 = 2;
pub const NFT_REPLY_COLLECTION_REDEEMED: u64 = 3;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NftCallback {
    pub contract_address: String,
    pub active_redeemed: u64,
}
