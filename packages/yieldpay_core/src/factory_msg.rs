use cosmwasm_bignumber::Uint256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub fee_collector: String,
    pub fee_amount: String,
    pub fee_max: Uint256,
    pub fee_reset_every_num_blocks: u64,
    pub money_market: String,
    pub dp_code_id: u64,
    pub anchor_pool_code_id: u64,
    pub nft_code_id: Option<u64>,
    pub homepage: Option<String>,
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Configure {
        fee_collector: Option<String>,
        fee_amount: Option<String>,
        fee_max: Option<Uint256>,
        fee_reset_every_num_blocks: Option<u64>,
        money_market: Option<String>,
        dp_code_id: Option<u64>,
        anchor_pool_code_id: Option<u64>,
        nft_contract: Option<String>,
        homepage: Option<String>,
    },
    CreateAnchorFund {
        pool_name: String,
        pool_title: String,
        pool_description: String,
        beneficiary: String,
    },

    AddAnchorFund {
        contract: String,
    },
    /// owner only.
    /// migrate fund to current code-id configured
    MigrateAnchorFund {
        contract: String,
    },
    /// remove anchor fund from listings. It doesn't close the fund. it just removes it from listings. visible = false to hide
    /// beneficiary can 'hide' fund from listing.
    HideAnchorFund {
        contract: String,
        visible: bool,
    },
    /// removes NFTs from the fund.
    RemoveNftFromFund {
        contract: String,
    },
    CreateCollectionsForFund {
        contract: String,
        active_meta: String,
        redeemed_meta: String,
    },
    /// switches NFT administrator to the admin of this contract
    RevertNftAdmin {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    AllAnchorFunds {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    AllAnchorFundsCount,
    AnchorFundsByBeneficiary {
        beneficiary: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    AnchorFundsByOwner {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    AnchorFundsByPoolName {
        pool_name: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    AnchorFund {
        contract: String,
    },
    AnchorFundEx {
        contract: String,
    },
}
