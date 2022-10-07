use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct NFTInstantiateMsg {
    /// Name of the NFT contract
    pub name: String,
    /// Symbol of the NFT contract
    pub symbol: String,

    /// The minter is the only one who can create new NFTs.
    /// This is designed for a base NFT that is controlled by an external program
    /// or contract. You will likely replace this with custom logic in custom NFTs
    /// This field can be set at the collection level which will override this.
    pub minter: String,
    /// The admin is updateable and can do NewCollection - maybe additional things
    /// in the future
    pub admin: Option<String>,
    /// public key that can sign buy messages
    pub public_key: String,
    /// minimum amount of uluna to buy via BUY message
    pub mint_amount: u64,
    /// minimum amount of uusd to execute a change message
    pub change_amount: u64,
    /// price change multiplier
    pub change_multiplier: u64,
    /// max amount of tokens to issue
    pub max_issuance: u64,
    /// max amount of collections to create
    pub max_collections: u64,
    /// transferable flag
    pub transferable: bool,
    // default royalty to be charged for tokens. This can also be set at the collection level
    pub default_royalty: String,
    /// can minter change collections (usually false)
    pub minter_can_switch_collections: bool,
}
impl NFTInstantiateMsg {
    pub fn gen_default(name: &str, symbol: &str, contract: &str) -> Self {
        NFTInstantiateMsg {
            name: name.to_string(),
            symbol: symbol.to_string(),
            minter: contract.to_string(),
            admin: Some(contract.to_string()),
            public_key: "A8O7tqWAvsKW9XA7p2W8YZdIZmmadf9qoQmRiZq8xpvl".to_string(),
            mint_amount: 2000000,
            change_amount: 1000000,
            change_multiplier: 0,
            max_issuance: 0,
            max_collections: 0,
            transferable: false,
            default_royalty: "0.01".to_string(),
            minter_can_switch_collections: true,
        }
    }
}
