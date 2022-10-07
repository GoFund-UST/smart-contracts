use crate::collection::{CollectionExecuteMsg, NewCollectionMsg, Trait};

use cosmwasm_std::*;
use std::str::FromStr;
use yieldpay_core::factory_response::AnchorPool;
use yieldpay_core::pool_msg::{
    NftCallback, NFT_REPLY_COLLECTION_ACTIVE, NFT_REPLY_COLLECTION_REDEEMED,
};
use yieldpay_core::{pool_anchor_msg, pool_msg};

use crate::config;
use crate::config::read;
use crate::contract::INSTANTIATE_REPLY_ID;
use crate::error::ContractError;
use crate::querier::pool_anchor::pool_anchor_config;
use crate::state::anchor_pools;

#[allow(clippy::too_many_arguments)]
pub fn configure(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    fee_collector: Option<String>,
    fee_amount: Option<String>,
    fee_max: Option<Uint128>,
    fee_reset_every_num_blocks: Option<u64>,
    money_market: Option<String>,
    dp_code_id: Option<u64>,
    anchor_pool_code_id: Option<u64>,
    nft_contract: Option<String>,
    homepage: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage).unwrap();
    if config.owner != deps.api.addr_canonicalize(info.sender.as_str()).unwrap() {
        return Err(ContractError::Unauthorized {
            action: "configure".to_string(),
            expected: deps.api.addr_humanize(&config.owner).unwrap().to_string(),
            actual: info.sender.to_string(),
        });
    }

    if let Some(fee_collector) = fee_collector {
        config.fee_collector = deps.api.addr_canonicalize(fee_collector.as_str()).unwrap();
    }
    if let Some(fee_amount) = fee_amount {
        config.fee_amount = Decimal::from_str(&fee_amount)?
    }
    if let Some(fee_max) = fee_max {
        config.fee_max = fee_max
    }
    if let Some(fee_reset_every_num_blocks) = fee_reset_every_num_blocks {
        config.fee_reset_every_num_blocks = fee_reset_every_num_blocks
    }
    if let Some(money_market) = money_market {
        config.money_market = deps.api.addr_canonicalize(money_market.as_str())?
    }
    if let Some(dp_code_id) = dp_code_id {
        config.dp_code_id = dp_code_id
    }
    if let Some(anchor_pool_code_id) = anchor_pool_code_id {
        config.anchor_pool_code_id = anchor_pool_code_id
    }
    if let Some(nft) = nft_contract {
        if nft.to_ascii_lowercase() == "none" {
            config.nft_contract = None;
        } else {
            let nft_addr = deps.api.addr_canonicalize(nft.as_str())?;
            config.nft_contract = Some(nft_addr)
        }
    }
    if let Some(homepage_url) = homepage {
        if homepage_url.to_ascii_lowercase() == "none" {
            config.homepage = None;
        } else {
            config.homepage = Some(homepage_url)
        }
    }

    config::store(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn create_anchor_fund(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    pool_name: String,
    pool_title: String,
    pool_description: String,
    beneficiary: String,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage).unwrap();
    let nft_contract = if let Some(nft_addr) = config.nft_contract {
        Some(deps.api.addr_humanize(&nft_addr)?.to_string())
    } else {
        None
    };

    Ok(Response::new().add_submessage(SubMsg {
        // Create Anchor Pool contract
        msg: WasmMsg::Instantiate {
            admin: Some(_env.contract.address.to_string()), // FIX/TODO switch to None
            //  admin: Some(deps.api.addr_humanize(&config.owner)?.to_string()), // FIX/TODO switch to None
            code_id: config.anchor_pool_code_id,
            funds: vec![],
            label: "".into(),
            msg: to_binary(&pool_anchor_msg::InstantiateMsg {
                pool_name,
                pool_title,
                pool_description,
                beneficiary,
                fee_collector: deps.api.addr_humanize(&config.fee_collector)?.to_string(),
                fee_amount: config.fee_amount.to_string(),
                fee_max: config.fee_max,
                fee_reset_every_num_blocks: config.fee_reset_every_num_blocks,
                money_market: deps.api.addr_humanize(&config.money_market)?.to_string(),
                dp_code_id: config.dp_code_id,
                owner_can_change_config: false, // TODO should this be configurable
                nft_contract,
                nft_collection_active: None,
                nft_collection_redeemed: None,
            })
            .map_err(|_o| ContractError::InstantiateError {
                action: "anchor_pool_code_id".to_string(),
            })?,
        }
        .into(),
        gas_limit: None,
        id: INSTANTIATE_REPLY_ID,
        reply_on: ReplyOn::Success,
    }))
}

pub fn add_anchor_fund(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract: String,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;
    if deps.api.addr_humanize(&config.owner)? != info.sender {
        return Err(ContractError::Unauthorized {
            action: "add_anchor_fund".to_string(),
            expected: deps.api.addr_humanize(&config.owner)?.to_string(),
            actual: info.sender.to_string(),
        });
    }
    let address = deps.api.addr_validate(&contract)?;

    let pool_config = pool_anchor_config(deps.as_ref(), &address)?;

    let ap = anchor_pools();
    if ap.may_load(deps.storage, address.to_string())?.is_some() {
        return Err(ContractError::AnchorPoolAlreadyRegistered(
            address.to_string(),
        ));
    }
    let anchor_config = AnchorPool {
        contract,
        owner: pool_config.owner,
        beneficiary: pool_config.beneficiary,
        pool_name: pool_config.pool_name.clone(),
        open: true,
        active_collection: None,
        redeemed_collection: None,
    };

    ap.save(deps.storage, address.to_string(), &anchor_config)?;

    Ok(Response::new()
        .add_attribute("anchor_pool_token", address.to_string())
        .add_attribute("pool_name", pool_config.pool_name))
}
pub fn migrate_anchor_fund(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract: String,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;
    if deps.api.addr_humanize(&config.owner)? != info.sender {
        return Err(ContractError::Unauthorized {
            action: "migrate_anchor_fund".to_string(),
            expected: deps.api.addr_humanize(&config.owner)?.to_string(),
            actual: info.sender.to_string(),
        });
    }
    let address = deps.api.addr_validate(&contract)?;

    let ap = anchor_pools();
    if ap.may_load(deps.storage, address.to_string())?.is_none() {
        return Err(ContractError::AnchorPoolNotFound(address.to_string()));
    }
    let migrate_message = pool_anchor_msg::MigrateMsg {};

    let migrate_msg = CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr: contract,
        new_code_id: config.anchor_pool_code_id,
        msg: to_binary(&migrate_message)?,
    });

    Ok(Response::new()
        .add_attribute("action", "migrate_anchor_fund")
        .add_attribute("anchor_pool_token", address.to_string())
        .add_attribute("code_id", &format!("{}", config.anchor_pool_code_id))
        .add_message(migrate_msg))
}

pub fn hide_anchor_fund(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract: String,
    visible: bool,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;
    let address = deps.api.addr_validate(&contract)?;

    let ap = anchor_pools();
    if let Some(mut anchor_pool) = ap.may_load(deps.storage, address.to_string())? {
        if (deps.api.addr_validate(&anchor_pool.beneficiary)? == info.sender && !visible)
            || (deps.api.addr_humanize(&config.owner)? == info.sender)
        {
            anchor_pool.open = visible;
            ap.save(deps.storage, address.to_string(), &anchor_pool)?;
            Ok(Response::new()
                .add_attribute("anchor_pool_token", address.to_string())
                .add_attribute("visible", visible.to_string()))
        } else {
            Err(ContractError::Unauthorized {
                action: "hide_anchor_fund".to_string(),
                expected: deps.api.addr_humanize(&config.owner)?.to_string(),
                actual: info.sender.to_string(),
            })
        }
    } else {
        Err(ContractError::AnchorPoolNotFound(contract))
    }
}
// mainly for bug fixing
pub fn remove_nft_from_fund(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract: String,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;
    let address = deps.api.addr_validate(&contract)?;

    let ap = anchor_pools();
    if let Some(anchor_pool) = ap.may_load(deps.storage, address.to_string())? {
        if (deps.api.addr_validate(&anchor_pool.beneficiary)? == info.sender)
            || (deps.api.addr_humanize(&config.owner)? == info.sender)
        {
            let clear_nft_message = pool_msg::ExecuteMsg::ClearNftContract {};

            Ok(Response::new()
                .add_attribute("remove_nft_from_fund", address.to_string())
                .add_message(clear_nft_message.into_cosmos_msg(address)?))
        } else {
            Err(ContractError::Unauthorized {
                action: "remove_nft_from_fund".to_string(),
                expected: deps.api.addr_humanize(&config.owner)?.to_string(),
                actual: info.sender.to_string(),
            })
        }
    } else {
        Err(ContractError::AnchorPoolNotFound(contract))
    }
}

pub fn create_collections_for_fund(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    active_meta: String,
    redeemed_meta: String,
) -> Result<Response, ContractError> {
    let config = read(deps.storage)?;
    if config.nft_contract.is_none() {
        return Err(ContractError::NFTContractNotSet);
    }
    let nft_contract = deps
        .api
        .addr_humanize(&config.nft_contract.unwrap())?
        .to_string();
    let address = deps.api.addr_validate(&contract)?;

    let ap = anchor_pools();
    let anchor_pool = ap.load(deps.storage, address.to_string())?;
    let beneficiary = deps.api.addr_validate(&anchor_pool.beneficiary)?;
    if info.sender != beneficiary {
        return Err(ContractError::Unauthorized {
            action: "create_collections_for_fund".to_string(),
            expected: anchor_pool.beneficiary,
            actual: info.sender.to_string(),
        });
    }
    //println!("Active ={}", active_meta);
    let active_decoded = base64::decode(&active_meta)?;
    let mut active: NewCollectionMsg = serde_json_wasm::from_slice(&active_decoded)?;
    active.minter = Some(contract.clone());

    let mut new_attributes: Vec<Trait> = if let Some(attributes) = active.attributes {
        attributes
            .into_iter()
            .filter(|t| {
                t.trait_type != "homepage"
                    && t.trait_type != "fund"
                    && t.trait_type != "active"
                    && t.trait_type != "fund_url"
            })
            //     .map(|t| t.clone())
            .collect::<Vec<Trait>>()
    } else {
        Vec::new()
    };

    let homepage = if let Some(home) = config.homepage {
        home
    } else {
        "-not set-".into()
    };
    new_attributes.push(Trait {
        display_type: None,
        trait_type: "homepage".to_string(),
        value: homepage.clone(),
    });
    new_attributes.push(Trait {
        display_type: None,
        trait_type: "fund".to_string(),
        value: contract.clone(),
    });
    new_attributes.push(Trait {
        display_type: None,
        trait_type: "fund_url".to_string(),
        value: format!("{}/fund/{}", homepage, contract),
    });
    new_attributes.push(Trait {
        display_type: None,
        trait_type: "active".to_string(),
        value: "true".to_string(),
    });

    active.attributes = Some(new_attributes);
    let active_callback = NftCallback {
        contract_address: env.contract.address.to_string(), // this isn't really needed I think.
        active_redeemed: NFT_REPLY_COLLECTION_ACTIVE,
    };

    let active_str = serde_json_wasm::to_string(&active)?;
    let active_b64 = base64::encode(active_str);

    // Send a message to the NFT to create a collection, and notify the fund when this occurs
    let active_new_collection = CollectionExecuteMsg::NewCollectionWithNotify {
        contract: contract.clone(),
        new_collection: active_b64,
        msg: to_binary(&active_callback)?,
    };
    //xxx
    let redeem_decoded = base64::decode(&redeemed_meta)?;
    let mut redeemed: NewCollectionMsg = serde_json_wasm::from_slice(&redeem_decoded)?;
    redeemed.minter = Some(contract.clone());

    let mut redeeemed_attributes: Vec<Trait> = if let Some(attributes) = redeemed.attributes {
        attributes
            .into_iter()
            .filter(|t| {
                t.trait_type != "homepage"
                    && t.trait_type != "fund"
                    && t.trait_type != "active"
                    && t.trait_type != "fund_url"
            })
            .collect::<Vec<Trait>>()
    } else {
        Vec::new()
    };

    redeeemed_attributes.push(Trait {
        display_type: None,
        trait_type: "homepage".to_string(),
        value: homepage.clone(),
    });

    redeeemed_attributes.push(Trait {
        display_type: None,
        trait_type: "fund".to_string(),
        value: contract.clone(),
    });
    redeeemed_attributes.push(Trait {
        display_type: None,
        trait_type: "fund_url".to_string(),
        value: format!("{}/fund/{}", homepage, contract),
    });
    redeeemed_attributes.push(Trait {
        display_type: None,
        trait_type: "active".to_string(),
        value: "false".to_string(),
    });

    redeemed.attributes = Some(redeeemed_attributes);
    let redeemed_callback = NftCallback {
        contract_address: env.contract.address.to_string(), // this isn't really needed I think.
        active_redeemed: NFT_REPLY_COLLECTION_REDEEMED,
    };

    let redeemed_str = serde_json_wasm::to_string(&redeemed)?;
    let redeemed_b64 = base64::encode(redeemed_str);

    // Send a message to the NFT to create a collection, and notify the fund when this occurs
    let redeem_new_collection = CollectionExecuteMsg::NewCollectionWithNotify {
        contract: contract.clone(),
        new_collection: redeemed_b64,
        msg: to_binary(&redeemed_callback)?,
    };

    //TODO Set NFT on fund.
    Ok(Response::new()
        .add_attribute("create_collections_for_fund", address.to_string())
        .add_attribute("contract", contract)
        .add_message(active_new_collection.into_cosmos_msg(nft_contract.clone())?)
        .add_message(redeem_new_collection.into_cosmos_msg(nft_contract)?))
}

pub fn register_anchor_pool_token(
    deps: DepsMut,
    _env: Env,

    address: &Addr,
) -> Result<Response, ContractError> {
    //  let config = config::read(deps.storage).unwrap();

    let pool_config = pool_anchor_config(deps.as_ref(), address)?;

    if let Some(_existing) = anchor_pools().may_load(deps.storage, address.to_string())? {
        return Err(ContractError::AnchorPoolAlreadyRegistered(
            address.to_string(),
        ));
    }
    let anchor_config = AnchorPool {
        contract: address.to_string(),
        owner: pool_config.owner,
        beneficiary: pool_config.beneficiary,
        pool_name: pool_config.pool_name.clone(),
        open: true,
        active_collection: None,
        redeemed_collection: None,
    };
    anchor_pools().save(deps.storage, address.to_string(), &anchor_config)?;

    Ok(Response::new()
        .add_attribute("anchor_pool_contract", address.to_string())
        .add_attribute("pool_name", pool_config.pool_name))
}
pub fn register_nft_token(
    deps: DepsMut,
    _env: Env,
    address: &str,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage).unwrap();
    config.nft_contract = Some(deps.api.addr_canonicalize(address)?);

    config::store(deps.storage, &config)?;
    Ok(Response::new().add_attribute("nft_contract", address.to_string()))
}

pub fn revert_nft_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = read(deps.storage)?;
    let address = deps.api.addr_humanize(&config.owner)?;
    if info.sender != address {
        return Err(ContractError::Unauthorized {
            action: "revert_nft_admin".to_string(),
            expected: address.to_string(),
            actual: info.sender.to_string(),
        });
    }
    if let Some(nft_canonical_addr) = config.nft_contract {
        let nft_addr = deps.api.addr_humanize(&nft_canonical_addr)?;
        let updateadmin = CollectionExecuteMsg::UpdateAdmin {
            admin: Some(address.to_string()),
        }
        .into_cosmos_msg(nft_addr.to_string())?;

        Ok(Response::new()
            .add_message(updateadmin)
            .add_attribute("action", "revert_nft_admin")
            .add_attribute("admin", address.to_string()))
    } else {
        Err(ContractError::NFTContractNotSet)
    }
}
