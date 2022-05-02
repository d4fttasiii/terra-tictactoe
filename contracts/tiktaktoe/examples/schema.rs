use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use cosmwasm_std::{Coin, Uint128};
use cw_controllers::AdminResponse;
use tiktaktoe::msg::{ExecuteMsg, GameResponse, InstantiateMsg, LockedResponse, QueryMsg, GamesResponse, LeaderboardResponse};
use tiktaktoe::state::{Config, Game};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Config), &out_dir);
    export_schema(&schema_for!(LockedResponse), &out_dir);
    export_schema(&schema_for!(GameResponse), &out_dir);
    export_schema(&schema_for!(GamesResponse), &out_dir);
    export_schema(&schema_for!(AdminResponse), &out_dir);
    export_schema(&schema_for!(LeaderboardResponse), &out_dir);
    export_schema(&schema_for!(Game), &out_dir);
    export_schema(&schema_for!(Coin), &out_dir);
    export_schema(&schema_for!(Uint128), &out_dir);
}
