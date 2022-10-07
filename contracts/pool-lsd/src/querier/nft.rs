use cosmwasm_std::{to_binary, Addr, CanonicalAddr, Deps, QueryRequest, StdResult, WasmQuery};
use cw721::TokensResponse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecMsg {
    QuickMint {
        /// token prefix for generated ids
        token_prefix: String,
        /// The owner of the newly minter NFT
        owner: String,
        /// Universal resource identifier for this NFT
        /// Should point to a JSON file that conforms to the ERC721
        /// Collection the NFT belongs to
        collection_id: u64,
        /// number of tokens to mint in collection
        num_to_mint: u64,
    },
    /*
    Mint {
        token_id: String,
        /// The owner of the newly minter NFT
        owner: String,
        /// Universal resource identifier for this NFT
        /// Should point to a JSON file that conforms to the ERC721
        /// Metadata JSON Schema
        token_uri: Option<String>,
        /// Collection the NFT belongs to
        collection_id: u64,
        extension: Metadata,
    },

     */
    SwitchCollection {
        token_id: String,
        new_collection_id: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// With Enumerable extension.
    /// Returns all tokens owned by the given address in a given collection, [] if unset.
    /// Return type: TokensResponse.
    TokensInCollection {
        owner: String,
        collection_id: u64,
        start_after: Option<String>,
        limit: Option<u32>,
    },
}
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, JsonSchema, Debug, Default)]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, JsonSchema, Debug, Default)]
pub struct Metadata {
    pub token_uri: String,
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
    pub current_status: Option<String>,
}
pub fn nft_exists(
    deps: Deps,
    nft_address: &CanonicalAddr,
    owner: &Addr,
    collection_id: u64,
) -> StdResult<TokensResponse> {
    let msg = QueryMsg::TokensInCollection {
        owner: owner.to_string(),
        collection_id,
        start_after: None,
        limit: None,
    };
    let token_response: TokensResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.addr_humanize(nft_address).unwrap().to_string(),
            msg: to_binary(&msg)?,
        }))?;

    Ok(token_response)
}

pub fn quick_mint_msg(prefix: &str, owner: &Addr, collection: u64) -> ExecMsg {
    ExecMsg::QuickMint {
        token_prefix: prefix.to_string(),
        owner: owner.to_string(),
        collection_id: collection,
        num_to_mint: 1,
    }
}
/*
pub fn mint_msg(prefix: &str, owner: &Addr, collection: u64, env: Env) -> ExecMsg {
    let t = Trait {
        display_type: None,
        trait_type: "created".to_string(),
        value: format!("{}", env.block.time),
    };
    let m = Metadata {
        token_uri: "".to_string(),
        image: None,
        image_data: None,
        external_url: None,
        description: None,
        name: None,
        attributes: Some(vec![t]),
        background_color: None,
        animation_url: None,
        youtube_url: None,
        current_status: None,
    };
    ExecMsg::Mint {
        token_id: format!("{}-{}-{}-{}", prefix, owner, collection, env.block.height),
        owner: owner.to_string(),
        token_uri: None,
        collection_id: collection,
        extension: Default::default(),
    }
}
*/
pub fn switch_collection_msg(token_id: &str, new_collection: u64) -> ExecMsg {
    ExecMsg::SwitchCollection {
        token_id: token_id.to_string(),
        new_collection_id: new_collection,
    }
}
