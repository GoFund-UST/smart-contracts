use crate::config::LastClaimed;
use crate::contract;
use crate::error::ContractError;
use crate::handler::core::calc_fee;
use crate::mock_querier::mock_dependencies;
use crate::querier::anchor::{ConfigResponse, EpochStateResponse, QueryMsg as AnchorQueryMsg};
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary};
use gofund_ust_core::pool_anchor_msg::InstantiateMsg;
use gofund_ust_core::pool_anchor_response;
use gofund_ust_core::pool_msg::{ExecuteMsg, QueryMsg};
use schemars::_serde_json::json;
use std::str::FromStr;

const MONEY_MARKET: &str = "money-market";
const ATOKEN_CONTRACT: &str = "terra1m0rjzm27qetjj8fx89knnhl8frvlrmjcfultav";

#[test]
fn instantiate() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let info = mock_info("owner", &[]);

    deps.querier.register_wasm_smart_query_handler(
        MONEY_MARKET.to_string(),
        Box::new(|x| match from_binary::<AnchorQueryMsg>(x).unwrap() {
            AnchorQueryMsg::Config {} => to_binary(&ConfigResponse {
                owner_addr: "".to_string(),
                aterra_contract: ATOKEN_CONTRACT.to_string(),
                interest_model: "".to_string(),
                distribution_model: "".to_string(),
                overseer_contract: "".to_string(),
                collector_contract: "".to_string(),
                distributor_contract: "".to_string(),
                stable_denom: "uusd".to_string(),
                max_borrow_factor: Default::default(),
            }),
            AnchorQueryMsg::EpochState { .. } => to_binary(&EpochStateResponse {
                exchange_rate: Default::default(),
                aterra_supply: Default::default(),
            }),
        }),
    );

    let msg = InstantiateMsg {
        pool_name: "test-pool".to_string(),
        pool_title: "Snappy oneliner designed to be catch the person's attention 🚀".to_string(),
        pool_description: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string(),
        beneficiary: "test-beneficiary".to_string(),
        fee_collector: "test-fee-collector".to_string(),
        fee_amount: "0.05".to_string(),
        fee_max: Default::default(),
        fee_reset_every_num_blocks: 0,
        money_market: MONEY_MARKET.to_string(),
        dp_code_id: 123456,
        owner_can_change_config: false,
        nft_contract: None,
        nft_collection_active: None,
        nft_collection_redeemed: None
    };
    let json = json!(msg).to_string();
    println!("{}", json);
    let resp = contract::instantiate(deps.as_mut(), env, info, msg)
        .expect("testing: should init contract");
    println!("{:?}", resp);
}

#[test]
fn test_calc_fee() {
    let last_claimed = LastClaimed {
        last_claimed_at_block_height: 100,
        fees_collected: Default::default(),
        total_earned_at_last_claimed: Default::default(),
    };
    let (fee, _new_last) = calc_fee(
        Uint256::zero(),
        Decimal256::from_str("0.05").unwrap(),
        Uint256::from(1_000_000_000u128),
        1000u64,
        2000u64,
        last_claimed.clone(),
    );
    assert_eq!(fee, Uint256::zero());
    let (fee, new_last) = calc_fee(
        Uint256::from(1_000_000u64),
        Decimal256::from_str("0.05").unwrap(),
        Uint256::from(1_000_000_000u128),
        1000u64,
        2000u64,
        last_claimed,
    );

    assert_eq!(
        new_last.total_earned_at_last_claimed,
        Uint256::from(950_000u64)
    );
    assert_eq!(new_last.fees_collected, Uint256::from(50_000u64));
    assert_eq!(fee, Uint256::from(50_000u64));
    let (fee, new_last2) = calc_fee(
        Uint256::from(1_000_000u64),
        Decimal256::from_str("0.05").unwrap(),
        Uint256::from(1_000_000_000u128),
        1000u64,
        3000u64,
        new_last,
    );
    assert_eq!(
        new_last2.total_earned_at_last_claimed,
        Uint256::from(1_900_000u64)
    );
    assert_eq!(new_last2.fees_collected, Uint256::from(100_000u64));

    assert_eq!(fee, Uint256::from(50_000u64));
}

#[test]
fn test_nft_set_reset() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let info = mock_info("owner", &[]);

    deps.querier.register_wasm_smart_query_handler(
        MONEY_MARKET.to_string(),
        Box::new(|x| match from_binary::<AnchorQueryMsg>(x).unwrap() {
            AnchorQueryMsg::Config {} => to_binary(&ConfigResponse {
                owner_addr: "".to_string(),
                aterra_contract: ATOKEN_CONTRACT.to_string(),
                interest_model: "".to_string(),
                distribution_model: "".to_string(),
                overseer_contract: "".to_string(),
                collector_contract: "".to_string(),
                distributor_contract: "".to_string(),
                stable_denom: "uusd".to_string(),
                max_borrow_factor: Default::default(),
            }),
            AnchorQueryMsg::EpochState { .. } => to_binary(&EpochStateResponse {
                exchange_rate: Default::default(),
                aterra_supply: Default::default(),
            }),
        }),
    );

    let msg = InstantiateMsg {
        pool_name: "test-pool".to_string(),
        pool_title: "Snappy oneliner designed to be catch the person's attention 🚀".to_string(),
        pool_description: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string(),
        beneficiary: "test-beneficiary".to_string(),
        fee_collector: "test-fee-collector".to_string(),
        fee_amount: "0.05".to_string(),
        fee_max: Default::default(),
        fee_reset_every_num_blocks: 0,
        money_market: MONEY_MARKET.to_string(),
        dp_code_id: 123456,
        owner_can_change_config: false,
        nft_contract: None,
        nft_collection_active: None,
        nft_collection_redeemed: None
    };

    let _ = contract::instantiate(deps.as_mut(), env.clone(), info.clone(), msg)
        .expect("testing: should init contract");
    let qry_config = QueryMsg::Config {};
    let qry = from_binary::<pool_anchor_response::ConfigResponse>(
        &contract::query(deps.as_ref(), mock_env(), qry_config).expect("query config fail?"),
    )
    .expect("from_binary_fail");
    assert!(qry.nft_collection_active.is_none());
    assert!(qry.nft_collection_redeemed.is_none());
    assert!(qry.nft_contract.is_none());

    let msg = InstantiateMsg {
        pool_name: "test-pool".to_string(),
        pool_title: "Snappy oneliner designed to be catch the person's attention 🚀".to_string(),
        pool_description: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string(),
        beneficiary: "test-beneficiary".to_string(),
        fee_collector: "test-fee-collector".to_string(),
        fee_amount: "0.05".to_string(),
        fee_max: Default::default(),
        fee_reset_every_num_blocks: 0,
        money_market: MONEY_MARKET.to_string(),
        dp_code_id: 123456,
        owner_can_change_config: false,
        nft_contract: Some("nft-minter".to_string()),
        nft_collection_active: Some(2u64),
        nft_collection_redeemed: Some(17u64)
    };
    let _ = contract::instantiate(deps.as_mut(), env.clone(), info.clone(), msg)
        .expect("testing: should init contract");
    let qry_config = QueryMsg::Config {};
    let qry = from_binary::<pool_anchor_response::ConfigResponse>(
        &contract::query(deps.as_ref(), mock_env(), qry_config).expect("query config fail?"),
    )
    .expect("from_binary_fail");
    assert_eq!(qry.nft_collection_active.unwrap(), 2u64);
    assert_eq!(qry.nft_collection_redeemed.unwrap(), 17u64);
    assert_eq!(qry.nft_contract.unwrap(), "nft-minter");

    let random = mock_info("random", &[]);
    let beneficiary = mock_info("test-beneficiary", &[]);
    let fee_collector = mock_info("test-fee-collector", &[]);
    let clear_msg = ExecuteMsg::ClearNftContract;
    let set_msg = ExecuteMsg::SetNftContract {
        nft_contract: "nft-minter-2".to_string(),
        nft_collection_active: Some(33u64),
        nft_collection_redeemed: Some(47u64),
    };
    let err = contract::execute(
        deps.as_mut(),
        env.clone(),
        random.clone(),
        clear_msg.clone(),
    )
    .unwrap_err();
    match err {
        ContractError::Unauthorized { .. } => {}
        _ => {
            assert!(false, "Unexpected error {:?}", err)
        }
    }
    let err =
        contract::execute(deps.as_mut(), env.clone(), random.clone(), set_msg.clone()).unwrap_err();
    match err {
        ContractError::Unauthorized { .. } => {}
        _ => {
            assert!(false, "Unexpected error {:?}", err)
        }
    }
    // these should work
    let _resp = contract::execute(
        deps.as_mut(),
        env.clone(),
        beneficiary.clone(),
        clear_msg.clone(),
    )
    .unwrap();
    let _resp = contract::execute(
        deps.as_mut(),
        env.clone(),
        beneficiary.clone(),
        set_msg.clone(),
    )
    .unwrap();

    // these should work
    let _resp = contract::execute(
        deps.as_mut(),
        env.clone(),
        fee_collector.clone(),
        clear_msg.clone(),
    )
    .unwrap();
    let _resp =
        contract::execute(deps.as_mut(), env.clone(), info.clone(), set_msg.clone()).unwrap();

    // owner can only set if it is empty. this should fail
    let err =
        contract::execute(deps.as_mut(), env.clone(), info.clone(), set_msg.clone()).unwrap_err();

    match err {
        ContractError::Unauthorized { .. } => {}
        _ => {
            assert!(false, "Unexpected error {:?}", err)
        }
    }
}
