use cosmwasm_std::{Addr, Coin, StdResult, Storage, Timestamp};

use cw_controllers::Admin;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex, U64Key};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub locked: bool,
    pub fee_percentage: u8,
    pub threshold: u8,
    pub dimension: u8,
    pub min_bet: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Game {
    pub game_id: u64,
    pub host: Addr,
    pub bet: Coin,
    pub state: GameState,
    pub grid: Vec<Vec<u8>>,
    pub opponent: Addr,
    pub next_player: Addr,
    pub updated_at: Timestamp,
    pub winner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GameState {
    New,
    InProgress,
    Completed,
    PriceWithdrawn,
    Cancelled,
}

pub struct GamesIndexes<'a> {
    pub host: MultiIndex<'a, (Addr, U64Key), Game>,
    pub opponent: MultiIndex<'a, (Addr, U64Key), Game>,
}

impl IndexList<Game> for GamesIndexes<'_> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Game>> + '_> {
        let v: Vec<&dyn Index<Game>> = vec![&self.host, &self.opponent];
        Box::new(v.into_iter())
    }
}

pub fn games<'a>() -> IndexedMap<'a, U64Key, Game, GamesIndexes<'a>> {
    let indexes = GamesIndexes {
        host: MultiIndex::new(
            |game: &Game, _key| (game.host.clone(), U64Key::new(game.game_id)),
            "games",
            "games__host",
        ),
        opponent: MultiIndex::new(
            |game: &Game, _key| (game.opponent.clone(), U64Key::new(game.game_id)),
            "games",
            "game__opponent",
        ),
    };
    IndexedMap::new("games", indexes)
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const ADMIN: Admin = Admin::new("admin");

pub const GAMES_COUNT: Item<u64> = Item::new("game_count");
pub const LEADERBOARD: Map<Addr, u64> = Map::new("leaderboard");

pub fn next_id(store: &mut dyn Storage) -> StdResult<u64> {
    let id: u64 = GAMES_COUNT.may_load(store)?.unwrap_or_default() + 1;
    GAMES_COUNT.save(store, &id)?;
    Ok(id)
}
