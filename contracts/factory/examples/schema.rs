use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use gofund_ust_core::factory_msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use gofund_ust_core::factory_response::{
    AnchorPool, ConfigResponse, FundsCountResponse, FundsResponse,
};
use gofund_ust_factory::config::Config;
use std::env::current_dir;
use std::fs::create_dir_all;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Config), &out_dir);
    export_schema(&schema_for!(AnchorPool), &out_dir);
    export_schema(&schema_for!(FundsResponse), &out_dir);
    export_schema(&schema_for!(FundsCountResponse), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
}
