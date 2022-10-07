use cosmwasm_std::{to_binary, Binary, CosmosMsg, StdResult, WasmMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, JsonSchema, Debug, Default)]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct NewCollectionMsg {
    pub collection_uri: String,
    pub collection_image: Option<String>,
    pub token_image: Option<String>,
    pub collection_image_data: Option<String>,
    pub token_image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub collection_name: Option<String>,
    pub token_name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
    pub max_issuance: u64,
    pub embargo_until: u64,
    pub has_unique_tokens: bool,
    pub can_change_max_issuance: bool,
    /// this is not permanent
    pub transferable: bool,
    pub royalty: Option<String>,
    pub minter: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum CollectionExecuteMsg {
    /// Add a new collection
    //  NewCollection(NewCollectionMsg),
    /// NewCollectionWithNotify is a base message to create a collection and trigger an action
    /// on the receiving contract.
    NewCollectionWithNotify {
        contract: String,
        new_collection: String, //base64 Json
        msg: Binary,
    },
    UpdateAdmin {
        admin: Option<String>,
    },
}
impl CollectionExecuteMsg {
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
