use crate::mock_querier::mock_dependencies;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, StdError};

use crate::contract;
use schemars::_serde_json::json;
use yieldpay_core::factory_msg;
use yieldpay_core::factory_response::FundsCountResponse;
use yieldpay_core::pool_anchor_response;
use yieldpay_core::pool_msg;

const ANCHOR_POOL: &str = "pool-anchor";
const ANCHOR_POOL_2: &str = "pool-anchor-2";
const MONEY_MARKET: &str = "money-market";
const BENEFICIARY: &str = "bene";
const ACTIVE_META:&str = "ewogICAgImNvbGxlY3Rpb25fdXJpIjoiYWN0aXZlL3RlcnJhMWcwOTA4dDR5cTZ2cmRoN24zMnZjZmxqaGc3Z3M1NWQ0Z3k0dnpnIiwKICAgICJjb2xsZWN0aW9uX2ltYWdlIjoiaHR0cDovL2NsaXBhcnQtbGlicmFyeS5jb20vaW1hZ2VzL3JpbktSRWVqVC5wbmciLAogICAgInRva2VuX2ltYWdlIjoiaHR0cDovL2NsaXBhcnQtbGlicmFyeS5jb20vaW1hZ2VzL3lpa0tianI0VC5wbmciLAogICAgImNvbGxlY3Rpb25faW1hZ2VfZGF0YSI6bnVsbCwKICAgICJ0b2tlbl9pbWFnZV9kYXRhIjpudWxsLAogICAgImV4dGVybmFsX3VybCI6Imh0dHBzOi8vZXhhbXBsZS5jb20vYWN0aXZlIiwKICAgICJkZXNjcmlwdGlvbiI6IlBvb2wgQWN0aXZlIiwKICAgICJjb2xsZWN0aW9uX25hbWUiOiJQb29sIHh5eiBhY3RpdmUiLAogICAgInRva2VuX25hbWUiOiJQb29sIEFjdGl2ZSBQYXJ0aWNpcGFudCIsCiAgICAiYXR0cmlidXRlcyI6IFt7ImRpc3BsYXlfdHlwZSI6bnVsbCwidHJhaXRfdHlwZSI6ImV2ZW50LWRhdGUiLCJ2YWx1ZSI6IjIwMjEtMDItMjQifV0sCiAgICAiYmFja2dyb3VuZF9jb2xvciI6bnVsbCwKICAgICJhbmltYXRpb25fdXJsIjpudWxsLCJ5b3V0dWJlX3VybCI6bnVsbCwibWF4X2lzc3VhbmNlIjoyMDAsImVtYmFyZ29fdW50aWwiOjAsImhhc191bmlxdWVfdG9rZW5zIjp0cnVlLAogICAgImNhbl9jaGFuZ2VfbWF4X2lzc3VhbmNlIjpmYWxzZSwidHJhbnNmZXJhYmxlIjpmYWxzZSwicm95YWx0eSI6IjAuMiIKICAgIH0K";
const REDEEM_META:&str = "ewogICAgImNvbGxlY3Rpb25fdXJpIjoicmVkZWVtL3RlcnJhMWcwOTA4dDR5cTZ2cmRoN24zMnZjZmxqaGc3Z3M1NWQ0Z3k0dnpnIiwKICAgICJjb2xsZWN0aW9uX2ltYWdlIjoiaHR0cDovL2NsaXBhcnQtbGlicmFyeS5jb20vaW1hZ2VzL2tjS29nZzdjai5qcGciLAogICAgInRva2VuX2ltYWdlIjoiaHR0cDovL2NsaXBhcnQtbGlicmFyeS5jb20vaW1hZ2VzL2tUS0JYR2JUai5qcGciLAogICAgImNvbGxlY3Rpb25faW1hZ2VfZGF0YSI6bnVsbCwKICAgICJ0b2tlbl9pbWFnZV9kYXRhIjpudWxsLAogICAgImV4dGVybmFsX3VybCI6Imh0dHBzOi8vZXhhbXBsZS5jb20vcmVkZWVtIiwKICAgICJkZXNjcmlwdGlvbiI6IlBvb2wgQWN0aXZlIiwKICAgICJjb2xsZWN0aW9uX25hbWUiOiJQb29sIHh5eiByZWRlZW0iLAogICAgInRva2VuX25hbWUiOiJQb29sIEhpc3RvcmljYWwgUGFydGljaXBhbnQiLAogICAgImF0dHJpYnV0ZXMiOiBbeyJkaXNwbGF5X3R5cGUiOm51bGwsInRyYWl0X3R5cGUiOiJldmVudC1kYXRlIiwidmFsdWUiOiIyMDIxLTAyLTI0In1dLAogICAgImJhY2tncm91bmRfY29sb3IiOm51bGwsCiAgICAiYW5pbWF0aW9uX3VybCI6bnVsbCwKICAgICJ5b3V0dWJlX3VybCI6bnVsbCwKICAgICJtYXhfaXNzdWFuY2UiOjIwMCwKICAgICJlbWJhcmdvX3VudGlsIjowLAogICAgImhhc191bmlxdWVfdG9rZW5zIjp0cnVlLAogICAgImNhbl9jaGFuZ2VfbWF4X2lzc3VhbmNlIjpmYWxzZSwKICAgICJ0cmFuc2ZlcmFibGUiOnRydWUsCiAgICAicm95YWx0eSI6IjAuMiIKICAgIH0K";
#[test]
fn instantiate() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let info = mock_info("owner", &[]);

    deps.querier.register_wasm_smart_query_handler(
        ANCHOR_POOL.to_string(),
        Box::new(|x| match from_binary::<pool_msg::QueryMsg>(x).unwrap() {
            pool_msg::QueryMsg::Config {} => to_binary(&pool_anchor_response::ConfigResponse {
                pool_name: "pool_name".to_string(),
                pool_title: "pool_title".to_string(),
                pool_description: "pool_description".to_string(),
                beneficiary: BENEFICIARY.to_string(),
                fee_collector: "fee_addr".to_string(),
                owner: "owner_addr".to_string(),
                money_market: "money_addr".to_string(),
                stable_denom: "stable".to_string(),
                anchor_token: "anchor_token".to_string(),
                dp_token: "1234".to_string(),
                owner_can_change_config: false,
                nft_contract: None,
                nft_collection_active: None,
                nft_collection_redeemed: None,
            }),
            _ => {
                //  assert!(false, "unexpected message");
                Err(StdError::GenericErr {
                    msg: "wrong".to_string(),
                })
            }
        }),
    );

    let msg = factory_msg::InstantiateMsg {
        fee_collector: "test-fee-collector".to_string(),
        fee_amount: "0.05".to_string(),
        fee_max: Default::default(),
        fee_reset_every_num_blocks: 0,
        money_market: MONEY_MARKET.to_string(),
        dp_code_id: 666,
        anchor_pool_code_id: 12345,
        nft_code_id: None,
        // nft_instantiate: None,
        homepage: None,
    };
    let json = json!(msg).to_string();
    println!("{}", json);
    let resp = contract::instantiate(deps.as_mut(), env, info, msg)
        .expect("testing: should init contract");
    println!("{:?}", resp);
}

#[test]
fn add_pool() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let info = mock_info("owner", &[]);

    deps.querier.register_wasm_smart_query_handler(
        ANCHOR_POOL.to_string(),
        Box::new(|x| match from_binary::<pool_msg::QueryMsg>(x).unwrap() {
            pool_msg::QueryMsg::Config {} => to_binary(&pool_anchor_response::ConfigResponse {
                pool_name: "pool_name".to_string(),
                pool_title: "pool_title".to_string(),
                pool_description: "pool_description".to_string(),
                beneficiary: BENEFICIARY.to_string(),
                fee_collector: "fee_addr".to_string(),
                owner: "owner_addr".to_string(),
                money_market: "money_addr".to_string(),
                stable_denom: "stable".to_string(),
                anchor_token: "anchor_token".to_string(),
                dp_token: "1234".to_string(),
                owner_can_change_config: false,
                nft_contract: None,
                nft_collection_active: None,
                nft_collection_redeemed: None,
            }),
            _ => {
                //  assert!(false, "unexpected message");
                Err(StdError::GenericErr {
                    msg: "wrong".to_string(),
                })
            }
        }),
    );
    deps.querier.register_wasm_smart_query_handler(
        ANCHOR_POOL_2.to_string(),
        Box::new(|x| match from_binary::<pool_msg::QueryMsg>(x).unwrap() {
            pool_msg::QueryMsg::Config {} => to_binary(&pool_anchor_response::ConfigResponse {
                pool_name: "pool_name_2".to_string(),
                pool_title: "pool_title_2".to_string(),
                pool_description: "pool_description_2".to_string(),
                beneficiary: "bene_addr_2".to_string(),
                fee_collector: "fee_addr".to_string(),
                owner: "owner_addr".to_string(),
                money_market: "money_addr".to_string(),
                stable_denom: "stable".to_string(),
                anchor_token: "anchor_token".to_string(),
                dp_token: "1234".to_string(),
                owner_can_change_config: false,
                nft_contract: None,
                nft_collection_active: None,
                nft_collection_redeemed: None,
            }),
            _ => {
                //  assert!(false, "unexpected message");
                Err(StdError::GenericErr {
                    msg: "wrong".to_string(),
                })
            }
        }),
    );
    let msg = factory_msg::InstantiateMsg {
        fee_collector: "test-fee-collector".to_string(),
        fee_amount: "0.05".to_string(),
        fee_max: Default::default(),
        fee_reset_every_num_blocks: 0,
        money_market: MONEY_MARKET.to_string(),
        dp_code_id: 666,
        anchor_pool_code_id: 12345,
        nft_code_id: None,
        //nft_instantiate: None,
        homepage: None,
    };
    //   let json = json!(msg).to_string();
    //  println!("{}", json);
    let _resp = contract::instantiate(deps.as_mut(), env.clone(), info.clone(), msg)
        .expect("testing: should init contract");

    let funds = from_binary::<FundsCountResponse>(
        &contract::query(
            deps.as_ref(),
            env.clone(),
            factory_msg::QueryMsg::AllAnchorFundsCount,
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(0, funds.count);
    let msg = factory_msg::ExecuteMsg::AddAnchorFund {
        contract: ANCHOR_POOL.into(),
    };
    let _resp = contract::execute(deps.as_mut(), env.clone(), info.clone(), msg)
        .expect("testing: should add pool");
    // println!("{:?}", resp);
    let funds = from_binary::<FundsCountResponse>(
        &contract::query(
            deps.as_ref(),
            env.clone(),
            factory_msg::QueryMsg::AllAnchorFundsCount,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(1, funds.count);
    let msg = factory_msg::ExecuteMsg::AddAnchorFund {
        contract: ANCHOR_POOL_2.into(),
    };
    let _resp =
        contract::execute(deps.as_mut(), env.clone(), info, msg).expect("testing: should add pool");
    // println!("{:?}", resp);
    let funds = from_binary::<FundsCountResponse>(
        &contract::query(
            deps.as_ref(),
            env,
            factory_msg::QueryMsg::AllAnchorFundsCount,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(2, funds.count);
}

#[test]
fn create_anchor_fund() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let info = mock_info("owner", &[]);
    let bene = mock_info(BENEFICIARY, &[]);

    deps.querier.register_wasm_smart_query_handler(
        ANCHOR_POOL.to_string(),
        Box::new(|x| match from_binary::<pool_msg::QueryMsg>(x).unwrap() {
            pool_msg::QueryMsg::Config {} => to_binary(&pool_anchor_response::ConfigResponse {
                pool_name: "pool_name".to_string(),
                pool_title: "pool_title".to_string(),
                pool_description: "pool_description".to_string(),
                beneficiary: BENEFICIARY.to_string(),
                fee_collector: "fee_addr".to_string(),
                owner: "owner_addr".to_string(),
                money_market: "money_addr".to_string(),
                stable_denom: "stable".to_string(),
                anchor_token: "anchor_token".to_string(),
                dp_token: "1234".to_string(),
                owner_can_change_config: false,
                nft_contract: None,
                nft_collection_active: None,
                nft_collection_redeemed: None,
            }),
            _ => {
                //  assert!(false, "unexpected message");
                Err(StdError::GenericErr {
                    msg: "wrong".to_string(),
                })
            }
        }),
    );

    let msg = factory_msg::InstantiateMsg {
        fee_collector: "test-fee-collector".to_string(),
        fee_amount: "0.05".to_string(),
        fee_max: Default::default(),
        fee_reset_every_num_blocks: 0,
        money_market: MONEY_MARKET.to_string(),
        dp_code_id: 666,
        anchor_pool_code_id: 12345,
        nft_code_id: None,
        //nft_instantiate: None,
        homepage: None,
    };
    //   let json = json!(msg).to_string();
    //  println!("{}", json);
    let _resp = contract::instantiate(deps.as_mut(), env.clone(), info.clone(), msg)
        .expect("testing: should init contract");

    let msg = factory_msg::ExecuteMsg::AddAnchorFund {
        contract: ANCHOR_POOL.into(),
    };

    let _resp = contract::execute(deps.as_mut(), env.clone(), info.clone(), msg)
        .expect("testing: should add pool");

    let msg = factory_msg::ExecuteMsg::Configure {
        fee_collector: None,
        fee_amount: None,
        fee_max: None,
        fee_reset_every_num_blocks: None,
        money_market: None,
        dp_code_id: None,
        anchor_pool_code_id: None,
        nft_contract: Some("NFT-Contract".to_string()),
        homepage: None,
    };

    let _resp = contract::execute(deps.as_mut(), env.clone(), info.clone(), msg)
        .expect("testing: should ste NFT contract");

    let collections_msg = factory_msg::ExecuteMsg::CreateCollectionsForFund {
        contract: ANCHOR_POOL.to_string(),
        active_meta: ACTIVE_META.to_string(),
        redeemed_meta: REDEEM_META.to_string(),
    };
    let _resp = contract::execute(deps.as_mut(), env.clone(), bene.clone(), collections_msg)
        .expect("testing: should add collections");
    // println!("{:?}", resp);
    // assert!(false, "see prints")
}
