use cosmwasm_bignumber::Decimal256;
use cw2::{get_contract_version, set_contract_version};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "gofundust-factory";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use std::str::FromStr;

use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdError,
    StdResult, SubMsg, WasmMsg,
};
use gofund_ust_core::factory_msg::{ExecuteMsg, QueryMsg};
use gofund_ust_core::factory_msg::{InstantiateMsg, MigrateMsg};
//use protobuf::reflect::ReflectValueRef::Message;
//use gofund_ust_core::pool_msg;
use protobuf::Message;

use crate::config;
#[allow(unused_imports)]
use crate::config::read;
use crate::error::ContractError;
use crate::handler::core as CoreHandler;
use crate::handler::query as QueryHandler;
use crate::migrations::ConfigV100;
use crate::querier::nft::NFTInstantiateMsg;
use crate::response::MsgInstantiateContractResponse;

// this is used to create the anchor fund
pub const INSTANTIATE_REPLY_ID: u64 = 22;
// this one is to build the NFT
pub const INSTANTIATE_NFT_REPLY_ID: u64 = 21;
//pub const INSTANTIATE_REPLY_NFT_REDEEMED: u64 = 3;

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = config::Config {
        this: deps.api.addr_canonicalize(env.contract.address.as_str())?,
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        fee_collector: deps.api.addr_canonicalize(msg.fee_collector.as_str())?,
        fee_amount: Decimal256::from_str(&msg.fee_amount)?,
        fee_max: msg.fee_max,
        fee_reset_every_num_blocks: msg.fee_reset_every_num_blocks,
        money_market: deps.api.addr_canonicalize(msg.money_market.as_str())?,
        dp_code_id: msg.dp_code_id,
        anchor_pool_code_id: msg.anchor_pool_code_id,
        nft_code_id: msg.nft_code_id,
        nft_instantiate: None,
        homepage: msg.homepage,
        nft_contract: None,
    };

    config::store(deps.storage, &config)?;
    if let Some(nft_code_id) = msg.nft_code_id {
        Ok(Response::new().add_submessage(SubMsg {
            // Create NFT token
            msg: WasmMsg::Instantiate {
                admin: Some(env.contract.address.to_string()),
                code_id: nft_code_id,
                funds: vec![],
                label: "".to_string(),
                msg: to_binary(&NFTInstantiateMsg::gen_default(
                    "Go Fund US(T) NFT",
                    "GoFundNFT",
                    env.contract.address.as_str(),
                ))
                .map_err(|_o| ContractError::InstantiateError {
                    action: "nft_code_id".to_string(),
                })?,
            }
            .into(),
            gas_limit: None,
            id: INSTANTIATE_NFT_REPLY_ID,
            reply_on: ReplyOn::Success,
        }))
    } else {
        Ok(Response::default())
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateAnchorFund {
            pool_name,
            pool_title: pool_oneliner,
            pool_description,
            beneficiary,
        } => CoreHandler::create_anchor_fund(
            deps,
            env,
            info,
            pool_name,
            pool_oneliner,
            pool_description,
            beneficiary,
        ),
        ExecuteMsg::Configure {
            fee_collector,
            fee_amount,
            fee_max,
            fee_reset_every_num_blocks,
            money_market,
            dp_code_id,
            anchor_pool_code_id,
            nft_contract,
            homepage,
        } => CoreHandler::configure(
            deps,
            env,
            info,
            fee_collector,
            fee_amount,
            fee_max,
            fee_reset_every_num_blocks,
            money_market,
            dp_code_id,
            anchor_pool_code_id,
            nft_contract,
            homepage,
        ),
        ExecuteMsg::AddAnchorFund { contract } => {
            CoreHandler::add_anchor_fund(deps, env, info, contract)
        }
        ExecuteMsg::MigrateAnchorFund { contract } => {
            CoreHandler::migrate_anchor_fund(deps, env, info, contract)
        }
        ExecuteMsg::HideAnchorFund { contract, visible } => {
            CoreHandler::hide_anchor_fund(deps, env, info, contract, visible)
        }
        ExecuteMsg::CreateCollectionsForFund {
            contract,
            active_meta,
            redeemed_meta,
        } => CoreHandler::create_collections_for_fund(
            deps,
            env,
            info,
            contract,
            active_meta,
            redeemed_meta,
        ),
        ExecuteMsg::RevertNftAdmin {} => CoreHandler::revert_nft_admin(deps, env, info),
        ExecuteMsg::RemoveNftFromFund { contract } => {
            CoreHandler::remove_nft_from_fund(deps, env, info, contract)
        }
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        INSTANTIATE_REPLY_ID => {
            // get new token's contract address
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(
                msg.result.unwrap().data.unwrap().as_slice(),
            )
            .map_err(|_| {
                ContractError::Std(StdError::parse_err(
                    "MsgInstantiateContractResponse",
                    "failed to parse data",
                ))
            })?;
            let token_addr = Addr::unchecked(res.get_contract_address());

            CoreHandler::register_anchor_pool_token(deps, env, &token_addr)
        }
        /*
        pool_msg::NFT_REPLY_COLLECTION_ACTIVE => Ok(Response::new().add_attribute(
            "NFT_replied-active",
            format!("{}", pool_msg::NFT_REPLY_COLLECTION_ACTIVE),
        )),
        pool_msg::NFT_REPLY_COLLECTION_REDEEMED => Ok(Response::new().add_attribute(
            "NFT_replied-redeemed",
            format!("{}", pool_msg::NFT_REPLY_COLLECTION_REDEEMED),
        )),

         */
        INSTANTIATE_NFT_REPLY_ID => {
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(
                msg.result.unwrap().data.unwrap().as_slice(),
            )
            .map_err(|_| {
                ContractError::Std(StdError::parse_err(
                    "MsgInstantiateContractResponse",
                    "failed to parse data/NFT",
                ))
            })?;

            CoreHandler::register_nft_token(deps, env, res.get_contract_address())
        }
        _ => Err(ContractError::InvalidReplyId { id: msg.id }),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => QueryHandler::config(deps, env),
        QueryMsg::AllAnchorFunds { start_after, limit } => to_binary(
            &QueryHandler::all_anchor_funds(deps, env, start_after, limit)?,
        ),
        QueryMsg::AnchorFundsByBeneficiary {
            beneficiary,
            start_after,
            limit,
        } => to_binary(&QueryHandler::anchor_funds_by_beneficiary(
            deps,
            env,
            &beneficiary,
            start_after,
            limit,
        )?),

        QueryMsg::AnchorFundsByOwner {
            owner,
            start_after,
            limit,
        } => to_binary(&QueryHandler::anchor_funds_by_owner(
            deps,
            env,
            &owner,
            start_after,
            limit,
        )?),

        QueryMsg::AnchorFundsByPoolName {
            pool_name,
            start_after,
            limit,
        } => to_binary(&QueryHandler::anchor_funds_by_pool_name(
            deps,
            env,
            &pool_name,
            start_after,
            limit,
        )?),

        QueryMsg::AnchorFund { contract } => {
            to_binary(&QueryHandler::anchor_fund(deps, env, &contract)?)
        }
        QueryMsg::AnchorFundEx { contract } => {
            to_binary(&QueryHandler::anchor_fund_ex(deps, env, &contract)?)
        }
        QueryMsg::AllAnchorFundsCount => {
            to_binary(&QueryHandler::all_anchor_fund_count(deps, env)?)
        }
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let contract_version = get_contract_version(deps.storage)?;

    match contract_version.contract.as_ref() {
        #[allow(clippy::single_match)]
        "gofundust-factory" => match contract_version.version.as_ref() {
            "0.1.1" => {
                let config_v100 = ConfigV100::load(deps.storage)?;

                config::store(deps.storage, &config_v100.migrate_from())?;
            }

            _ => {}
        },
        _ => {
            return Err(ContractError::MigrationError {
                current_name: contract_version.contract,
                current_version: contract_version.version,
            })
        }
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("previous_contract_name", &contract_version.contract)
        .add_attribute("previous_contract_version", &contract_version.version)
        .add_attribute("new_contract_name", CONTRACT_NAME)
        .add_attribute("new_contract_version", CONTRACT_VERSION))
}
