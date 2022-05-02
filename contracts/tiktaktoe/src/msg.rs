use cosmwasm_std::{Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::Game;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub min_bet: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateAdmin {
        new_admin: String,
    },
    UpdateConfig {
        dimension: u8,
        threshold: u8,
        fee_percentage: u8,
    },
    CreateGame {},
    CancelGame {
        game_id: u64,
    },
    JoinGame {
        game_id: u64,
    },
    MakeMove {
        game_id: u64,
        x: u8,
        y: u8,
    },
    WithdrawPrice {
        game_id: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    IsLocked {},
    GetAdmin {},
    GetGames {},
    GetGameById { id: u64 },
    GetGamesByAddress { address: String },
    GetLeaderboard { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LockedResponse {
    pub locked: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GameResponse {
    pub game: Game,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GamesResponse {
    pub games: Vec<Game>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LeaderboardResponse {
    pub win_count: u64,
}
