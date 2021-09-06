use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use licium_cw721::msg::{ExecuteMsg, InstantiateMsg, MintMsg, QueryMsg, TokenResponse};
use licium_cw721::state::{ IsccData, Licensing, License };

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(MintMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(TokenResponse), &out_dir);
    export_schema(&schema_for!(IsccData), &out_dir);
    export_schema(&schema_for!(Licensing), &out_dir);
    export_schema(&schema_for!(License), &out_dir);
}
