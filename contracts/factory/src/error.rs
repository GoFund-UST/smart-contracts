use cosmwasm_std::{OverflowError, StdError};
use protobuf::ProtobufError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("StdError:{0}")]
    Std(#[from] StdError),

    #[error("Overflow:{0}")]
    Overflow(#[from] OverflowError),

    #[error(
        "Factory: Unauthorized (action: {action:?}, expected: {expected:?}, actual: {actual:?})"
    )]
    Unauthorized {
        action: String,
        expected: String,
        actual: String,
    },
    #[error(transparent)]
    JsonDe(#[from] serde_json_wasm::de::Error),
    #[error(transparent)]
    JsonSer(#[from] serde_json_wasm::ser::Error),
    #[error(transparent)]
    DecodeError(#[from] base64::DecodeError),
    #[error(transparent)]
    ProtobufError(#[from] ProtobufError),

    #[error("Factory: Invalid reply ID (ID: {id:?}")]
    InvalidReplyId { id: u64 },
    #[error("Factory: InstantiateError Failed - {action:?} ")]
    InstantiateError { action: String },
    #[error("Factory: ExecuteError Failed - {action:?} ")]
    ExecuteError { action: String },

    #[error("Factory: Contract can't be migrated! {current_name:?} {current_version:?}")]
    MigrationError {
        current_name: String,
        current_version: String,
    },
    #[error("Factory: invalid? Anchor pool contract already registered {0}")]
    AnchorPoolAlreadyRegistered(String),
    #[error("Factory: pool contract {0} not found")]
    AnchorPoolNotFound(String),
    #[error("Factory:NFT Contract is not set")]
    NFTContractNotSet,
}
