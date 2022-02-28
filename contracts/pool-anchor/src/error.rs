use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("StdError: {0}")]
    Std(#[from] StdError),

    #[error("Overflow:{0}")]
    Overflow(#[from] OverflowError),

    #[error(
        "Core/Pool: Unauthorized (action: {action:?}, expected: {expected:?}, actual: {actual:?})"
    )]
    Unauthorized {
        action: String,
        expected: String,
        actual: String,
    },
    #[error(transparent)]
    ParseIntError(#[from] ::std::num::ParseIntError),
    #[error("Core/Pool: Invalid reply ID (ID: {id:?}")]
    InvalidReplyId { id: u64 },

    #[error("Core/Pool: Zero amount not allowed")]
    NotAllowZeroAmount {},

    #[error("Core/Pool: other denom except {denom:?} is not allowed")]
    NotAllowOtherDenoms { denom: String },

    #[error("Core/Pool: other action except {action:?} is not allowed")]
    NotAllowOtherCw20ReceiveAction { action: String },
    #[error("Core/Pool: InstantiateError Failed - {action:?} ")]
    InstantiateError { action: String },
    #[error("Core/Pool: Redeem amount requested is zero ")]
    RedeemZero {},
    #[error("Core/Pool: Redeem epoch exchange is zero ")]
    RedeemEpochIsZero {},
    #[error("Core/Pool: Redeem Tax error: {msg:?}")]
    RedeemTaxError { msg: String },
    #[error("Core/Pool: pool name must be a maximum of 9 characters with spaces removed")]
    PoolNameTooLarge,
    #[error("Core/Pool: NFT contract invalid")]
    NftContractInvalid,
    #[error("Core/Pool: NFT attempted to set collection id {0} to an invalid option {1}")]
    NftCollectionInvalidOption(u64, u64),
    #[error("Core/Pool: Contract can't be migrated! {current_name:?} {current_version:?}")]
    MigrationError {
        current_name: String,
        current_version: String,
    },
}
