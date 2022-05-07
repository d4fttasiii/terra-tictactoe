#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order, QueryRequest,
    Response, StdResult, SubMsg, WasmQuery,
};
use cw0::maybe_addr;
use cw2::set_contract_version;
use cw_storage_plus::U64Key;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, GameResponse, GamesResponse, InstantiateMsg, LeaderBoardEntry, LeaderboardResponse,
    LockedResponse, QueryMsg,
};
use crate::state::{
    games, next_id, Config, Game, GameState, ADMIN, CONFIG, GAMES_COUNT, LEADERBOARD,
};
use crate::terrand::{LatestRandomResponse, QueryMsg as TerrandQueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:terra_tictactoe";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = Config {
        fee_percentage: 2,
        locked: false,
        dimension: 6,
        threshold: 4,
        min_bet: msg.min_bet,
        terrand_address: deps.api.addr_validate(&msg.terrand_address)?,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &state)?;
    GAMES_COUNT.save(deps.storage, &0)?;
    ADMIN.set(deps, Some(info.sender.clone()))?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateAdmin { new_admin } => try_update_admin(deps, info, new_admin),
        ExecuteMsg::UpdateConfig {
            threshold,
            dimension,
            fee_percentage,
        } => try_update_config(deps, info, threshold, dimension, fee_percentage),
        ExecuteMsg::CreateGame {} => try_create_game(_env, deps, info),
        ExecuteMsg::CancelGame { game_id } => try_cancel_game(deps, info, game_id),
        ExecuteMsg::JoinGame { game_id } => try_join_game(_env, deps, info, game_id),
        ExecuteMsg::MakeMove { game_id, x, y } => try_make_move(_env, deps, info, game_id, x, y),
        ExecuteMsg::WithdrawPrice { game_id } => try_withdraw_price(_env, deps, info, game_id),
    }
}

pub fn try_update_admin(
    deps: DepsMut,
    info: MessageInfo,
    new_admin: String,
) -> Result<Response, ContractError> {
    let api = deps.api;
    Ok(ADMIN.execute_update_admin(deps, info, maybe_addr(api, Some(new_admin))?)?)
}

pub fn try_update_config(
    deps: DepsMut,
    info: MessageInfo,
    threshold: u8,
    dimension: u8,
    fee_percentage: u8,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    CONFIG.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.fee_percentage = fee_percentage;
        state.threshold = threshold;
        state.dimension = dimension;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_update_config"))
}

pub fn try_create_game(
    env: Env,
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    assert_is_locked(deps.as_ref(), &info.sender)?;
    assert_host_bet(deps.as_ref(), &info.funds[0])?;

    let config = CONFIG.load(deps.storage)?;
    let id = next_id(deps.storage)?;
    let mut grid: Vec<Vec<i8>> = Vec::new();
    let mut disabled_cells: Vec<(u8, u8)> = Vec::new();
    let mut round: usize = 0;
    for _ in 0..config.threshold {
        let x = get_random(deps.as_ref(), round, config.dimension)?;
        round += 1;
        let y = get_random(deps.as_ref(), round, config.dimension)?;
        round += 1;
        disabled_cells.push((x, y));
    }
    for i in 0..config.dimension {
        let mut row: Vec<i8> = Vec::new();
        for j in 0..config.dimension {
            if disabled_cells.contains(&(i, j)) {
                row.push(-1)
            } else {
                row.push(0);
            }
        }
        grid.push(row);
    }
    let amount = info.funds[0]
        .amount
        .multiply_ratio(u128::from(100 - config.fee_percentage), 100u128);
    let game = Game {
        game_id: id,
        bet: Coin {
            denom: config.min_bet.denom,
            amount,
        },
        host: info.sender.clone(),
        state: GameState::New,
        next_player: info.sender.clone(),
        grid: grid,
        opponent: Addr::unchecked(""),
        updated_at: env.block.time,
        winner: Addr::unchecked(""),
    };
    games().save(deps.storage, U64Key::new(id), &game)?;

    Ok(Response::new()
        .add_attribute("method", "try_create_game")
        .add_attribute("id", id.to_string()))
}

fn get_random(deps: Deps, round: usize, to: u8) -> StdResult<u8> {
    let config = CONFIG.load(deps.storage)?;
    let response: LatestRandomResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.terrand_address.to_string(),
            msg: to_binary(&TerrandQueryMsg::LatestDrand {})?,
        }))?;
    let randomness = vector_as_u8_array(response.randomness.to_vec(), round);
    let random_big_number = u16::from_be_bytes(randomness);
    let random_ranged_number = random_big_number.wrapping_rem_euclid(to.into()) as u8;

    Ok(random_ranged_number)
}

fn vector_as_u8_array(vector: Vec<u8>, start: usize) -> [u8; 2] {
    let mut arr = [0u8; 2];
    arr[0] = vector[start];
    arr[1] = vector[start + 1];

    arr
}

pub fn try_cancel_game(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    assert_is_locked(deps.as_ref(), &info.sender)?;
    games().update(deps.storage, U64Key::new(id), |g| match g {
        None => Err(ContractError::GameNotFound {}),
        Some(mut game) => {
            if game.host != info.sender {
                Err(ContractError::Unauthorized {})
            } else if game.state != GameState::New {
                Err(ContractError::GameCannotBeCancelled {})
            } else {
                game.state = GameState::Cancelled;
                Ok(game)
            }
        }
    })?;

    Ok(Response::new()
        .add_attribute("method", "try_cancel_game")
        .add_attribute("id", id.to_string()))
}

pub fn try_join_game(
    env: Env,
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    assert_is_locked(deps.as_ref(), &info.sender)?;
    let config = CONFIG.load(deps.storage)?;
    let amount = info.funds[0]
        .amount
        .multiply_ratio(u128::from(100 - config.fee_percentage), 100u128);
    games().update(deps.storage, U64Key::new(id), |g| match g {
        None => Err(ContractError::GameNotFound {}),
        Some(game) if game.host == info.sender => Err(ContractError::NotAllowedToJoin {}),
        Some(game) if game.state != GameState::New => {
            Err(ContractError::NotAllowedInCurrentState { state: game.state })
        }
        Some(game) if game.bet.amount != amount => Err(ContractError::BetAmounTooLow {}),
        Some(game) if game.bet.denom != info.funds[0].denom => {
            Err(ContractError::BetDenomInvalid {})
        }
        Some(mut game) => {
            game.opponent = info.sender.clone();
            game.state = GameState::InProgress;
            game.bet.amount = game.bet.amount + amount;
            game.updated_at = env.block.time;
            Ok(game)
        }
    })?;

    Ok(Response::new()
        .add_attribute("method", "try_join_game")
        .add_attribute("id", id.to_string())
        .add_attribute("opponent", info.sender.to_string()))
}

pub fn try_make_move(
    env: Env,
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
    x: u8,
    y: u8,
) -> Result<Response, ContractError> {
    assert_is_locked(deps.as_ref(), &info.sender)?;
    let mut maybe_winner: Option<Addr> = None;
    let config = CONFIG.load(deps.storage)?;
    games().update(deps.storage, U64Key::new(id), |g| match g {
        None => Err(ContractError::GameNotFound {}),
        Some(game) if game.next_player != info.sender.to_string() => {
            Err(ContractError::NotYourTurn {})
        }
        Some(game) if game.state != GameState::InProgress => {
            Err(ContractError::NotAllowedInCurrentState { state: game.state })
        }
        Some(mut game) => {
            let pos_x = usize::from(x);
            let pos_y = usize::from(y);
            game.grid[pos_x][pos_y] = get_mark_for_cell(&game, pos_x, pos_y)?;
            if is_game_completed(&game, config.dimension as u16, config.threshold as u16)? {
                game.state = GameState::Completed;
                game.winner = game.next_player.clone();
                maybe_winner = Some(game.next_player.clone());
            } else {
                game.next_player = get_next_player(&game)?;
            }
            game.updated_at = env.block.time;

            Ok(game)
        }
    })?;

    if maybe_winner != None {
        try_update_leaderboard(deps, maybe_winner.unwrap())?;
    }

    return Ok(Response::new()
        .add_attribute("method", "try_join_game")
        .add_attribute("id", id.to_string())
        .add_attribute("opponent", info.sender.to_string()));
}

fn try_update_leaderboard(deps: DepsMut, winner: Addr) -> Result<(), ContractError> {
    LEADERBOARD.update(
        deps.storage,
        winner,
        |maybe_win_count: Option<u64>| -> StdResult<u64> {
            match maybe_win_count {
                Some(win_count) => Ok(win_count + 1),
                None => Ok(1),
            }
        },
    )?;

    Ok(())
}

pub fn try_withdraw_price(
    env: Env,
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    assert_is_locked(deps.as_ref(), &info.sender)?;
    let game = query_game_by_id(deps.as_ref(), id)?.game;
    let msg = match game.state {
        GameState::Completed => {
            let message = SubMsg::new(BankMsg::Send {
                to_address: game.winner.to_string(),
                amount: vec![game.bet],
            });
            Ok(message)
        }
        GameState::InProgress if game.updated_at.seconds() + 3600 < env.block.time.seconds() => {
            let message = SubMsg::new(BankMsg::Send {
                to_address: get_next_player(&game)?.to_string(),
                amount: vec![game.bet],
            });
            Ok(message)
        }
        _ => Err(ContractError::PriceCannotBeWithdrawn {}),
    }?;

    games().update(deps.storage, U64Key::new(game.game_id), |g| match g {
        None => Err(ContractError::GameNotFound {}),
        Some(mut game_found) => {
            game_found.state = GameState::PriceWithdrawn;
            Ok(game_found)
        }
    })?;

    return Ok(Response::new()
        .add_submessage(msg)
        .add_attribute("method", "try_withdraw_price"));
}

fn get_mark_for_cell(game: &Game, x: usize, y: usize) -> Result<i8, ContractError> {
    if game.grid[x][y] != 0 || game.grid[x][y] == -1 {
        return Err(ContractError::MoveNotAllow {});
    } else if game.host.to_string() == game.next_player {
        Ok(1)
    } else {
        Ok(100)
    }
}

fn get_next_player(game: &Game) -> Result<Addr, ContractError> {
    if game.next_player == game.host {
        Ok(game.opponent.clone())
    } else {
        Ok(game.host.clone())
    }
}

fn is_game_completed(game: &Game, dimension: u16, threshold: u16) -> Result<bool, ContractError> {
    let mut sum_vertically: u16 = 0;
    let mut sum_horizontally: u16 = 0;
    let mut sum_diagonally_x: u16 = 0;
    let mut sum_diagonally_y: u16 = 0;
    let opponent_threshold = 100u16 * threshold;

    for x in 0..dimension {
        let pos_x = usize::from(x);
        for y in 0..dimension {
            let pos_y = usize::from(y);
            sum_vertically += if game.grid[pos_x][pos_y].is_negative() {
                0
            } else {
                game.grid[pos_x][pos_y] as u16
            };
            sum_horizontally += if game.grid[pos_y][pos_x].is_negative() {
                0
            } else {
                game.grid[pos_y][pos_x] as u16
            };
        }
        if (sum_vertically == threshold)
            || (sum_vertically == opponent_threshold)
            || (sum_horizontally == threshold)
            || (sum_horizontally == opponent_threshold)
        {
            return Ok(true);
        }
        sum_vertically = 0;
        sum_horizontally = 0;

        let diag_y = usize::from(dimension - 1 - x);
        sum_diagonally_x += if game.grid[pos_x][pos_x].is_negative() {
            0
        } else {
            game.grid[pos_x][pos_x] as u16
        };
        sum_diagonally_y += if game.grid[diag_y][pos_x].is_negative() {
            0
        } else {
            game.grid[diag_y][pos_x] as u16
        };
    }

    if (sum_diagonally_x == threshold)
        || (sum_diagonally_x == opponent_threshold)
        || (sum_diagonally_y == threshold)
        || (sum_diagonally_y == opponent_threshold)
    {
        return Ok(true);
    }

    Ok(false)
}

fn assert_is_locked(deps: Deps, sender: &Addr) -> Result<(), ContractError> {
    if ADMIN.is_admin(deps, sender)? {
        Ok(())
    } else {
        let state = CONFIG.load(deps.storage)?;
        if state.locked {
            Err(ContractError::Locked {})
        } else {
            Ok(())
        }
    }
}

fn assert_host_bet(deps: Deps, bet: &Coin) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.min_bet.denom != bet.denom {
        return Err(ContractError::BetDenomInvalid {});
    }
    if config.min_bet.amount > bet.amount {
        return Err(ContractError::BetAmounTooLow {});
    }

    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsLocked {} => to_binary(&query_locked(deps)?),
        QueryMsg::GetAdmin {} => to_binary(&ADMIN.query_admin(deps)?),
        QueryMsg::GetGames {} => to_binary(&query_games(deps)?),
        QueryMsg::GetGameById { id } => to_binary(&query_game_by_id(deps, id)?),
        QueryMsg::GetGamesByAddress { address } => {
            to_binary(&query_games_by_address(deps, address)?)
        }
        QueryMsg::GetLeaderboard {} => to_binary(&query_leaderboard(deps)?),
    }
}

pub fn query_locked(deps: Deps) -> StdResult<LockedResponse> {
    let state = CONFIG.load(deps.storage)?;
    Ok(LockedResponse {
        locked: state.locked,
    })
}

pub fn query_games(deps: Deps) -> StdResult<GamesResponse> {
    let all_games = games()
        .range(deps.storage, None, None, Order::Ascending)
        .flat_map(|item| match item {
            Ok((_, data)) => Some(data),
            _ => None,
        })
        .collect();

    Ok(GamesResponse { games: all_games })
}

pub fn query_game_by_id(deps: Deps, id: u64) -> StdResult<GameResponse> {
    let game = games().load(deps.storage, U64Key::new(id))?;
    Ok(GameResponse { game })
}

pub fn query_games_by_address(deps: Deps, address: String) -> StdResult<GamesResponse> {
    let addr = deps.api.addr_validate(address.as_str())?;
    let games_by_host: Vec<Game> = games()
        .idx
        .host
        .prefix(addr.clone())
        .range(deps.storage, None, None, Order::Ascending)
        .flat_map(|item| match item {
            Ok((_, data)) => Some(data),
            _ => None,
        })
        .collect();

    let games_by_opponent: Vec<Game> = games()
        .idx
        .opponent
        .prefix(addr.clone())
        .range(deps.storage, None, None, Order::Ascending)
        .flat_map(|item| match item {
            Ok((_, data)) => Some(data),
            _ => None,
        })
        .collect();

    let mut games: Vec<Game> = Vec::new();
    games.extend(games_by_host);
    games.extend(games_by_opponent);

    Ok(GamesResponse { games: games })
}

pub fn query_leaderboard(deps: Deps) -> StdResult<LeaderboardResponse> {
    let leaderboard_entries = LEADERBOARD
        .range(deps.storage, None, None, Order::Descending)
        .map(|item| {
            let (k, win_count) = item.unwrap();
            let address = match std::str::from_utf8(&k) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let player = Addr::unchecked(address);
            LeaderBoardEntry {
                player: player,
                win_count,
            }
        })
        .collect();

    Ok(LeaderboardResponse {
        entries: leaderboard_entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_querier::mock_dependencies;
    use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{coins, from_binary, Uint128};
    use cw_controllers::AdminResponse;

    #[test]
    fn init_contract() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {
            min_bet: Coin {
                amount: Uint128::new(10_000_000),
                denom: "uust".to_string(),
            },
            terrand_address: MOCK_CONTRACT_ADDR.to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::IsLocked {}).unwrap();
        let value: LockedResponse = from_binary(&res).unwrap();
        assert_eq!(false, value.locked);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAdmin {}).unwrap();
        let value: AdminResponse = from_binary(&res).unwrap();
        assert_eq!("creator", value.admin.unwrap());
    }

    #[test]
    fn create_game() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {
            min_bet: Coin {
                amount: Uint128::new(10_000_000),
                denom: "uust".to_string(),
            },
            terrand_address: MOCK_CONTRACT_ADDR.to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let unauth_info = mock_info("anyone", &coins(100_000_000, "uust"));
        let msg = ExecuteMsg::CreateGame {};
        let _res = execute(deps.as_mut(), mock_env(), unauth_info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGameById { id: 1 }).unwrap();
        let value: GameResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.game.game_id);
        assert_eq!(Uint128::from(98_000_000u128), value.game.bet.amount);
    }

    #[test]
    fn query_games_by_address() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {
            min_bet: Coin {
                amount: Uint128::new(10_000_000),
                denom: "uust".to_string(),
            },
            terrand_address: MOCK_CONTRACT_ADDR.to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let anyone = mock_info("anyone", &coins(100_000_000, "uust"));
        for _ in 0..5 {
            let msg = ExecuteMsg::CreateGame {};
            let _res = execute(deps.as_mut(), mock_env(), anyone.clone(), msg).unwrap();
        }

        // All games
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGames {}).unwrap();
        let value: GamesResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.games.len());

        // Games by address
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetGamesByAddress {
                address: "anyone".to_string(),
            },
        )
        .unwrap();
        let value: GamesResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.games.len());
    }

    #[test]
    fn join_game() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {
            min_bet: Coin {
                amount: Uint128::new(10_000_000),
                denom: "uust".to_string(),
            },
            terrand_address: MOCK_CONTRACT_ADDR.to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let unauth_info = mock_info("anyone", &coins(100_000_000, "uust"));
        let msg = ExecuteMsg::CreateGame {};
        let _res = execute(deps.as_mut(), mock_env(), unauth_info, msg).unwrap();

        let unauth_info = mock_info("anyone_else", &coins(100_000_000, "uust"));
        let msg = ExecuteMsg::JoinGame { game_id: 1 };
        let _res = execute(deps.as_mut(), mock_env(), unauth_info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGameById { id: 1 }).unwrap();
        let value: GameResponse = from_binary(&res).unwrap();
        assert_eq!(GameState::InProgress, value.game.state);
        assert_eq!("anyone_else", value.game.opponent.to_string());
        assert_eq!("anyone", value.game.host.to_string());
        assert_eq!(
            Coin {
                amount: Uint128::new(196_000_000),
                denom: "uust".to_string()
            },
            value.game.bet
        );
    }

    #[test]
    fn new_game_can_be_cancelled() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg {
            min_bet: Coin {
                amount: Uint128::new(10_000_000),
                denom: "uust".to_string(),
            },
            terrand_address: MOCK_CONTRACT_ADDR.to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let unauth_info = mock_info("anyone", &coins(100_000_000, "uust"));
        let msg = ExecuteMsg::CreateGame {};
        let _res = execute(deps.as_mut(), mock_env(), unauth_info, msg).unwrap();

        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::CancelGame { game_id: 1 };
        let _res = execute(deps.as_mut(), mock_env(), unauth_info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGameById { id: 1 }).unwrap();
        let value: GameResponse = from_binary(&res).unwrap();
        assert_eq!(GameState::Cancelled, value.game.state);

        // match res {
        //     Err(ContractError::Unauthorized {}) => {}
        //     _ => panic!("Must return unauthorized error"),
        // }

        // // only the original creator can reset the counter
        // let auth_info = mock_info("creator", &coins(2, "token"));
        // let msg = ExecuteMsg::Reset { count: 5 };
        // let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // // should now be 5
        // let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        // let value: CountResponse = from_binary(&res).unwrap();
        // assert_eq!(5, value.count);
    }

    #[test]
    fn full_game_until_host_wins() {
        let mut deps = mock_dependencies(&coins(2, "token"));
        deps.querier.with_terrand();

        let msg = InstantiateMsg {
            min_bet: Coin {
                amount: Uint128::new(10_000_000),
                denom: "uust".to_string(),
            },
            terrand_address: MOCK_CONTRACT_ADDR.to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let unauth_info = mock_info("anyone", &coins(100_000_000, "uust"));
        let msg = ExecuteMsg::CreateGame {};
        let _res = execute(deps.as_mut(), mock_env(), unauth_info, msg).unwrap();

        let unauth_info = mock_info("anyone_else", &coins(100_000_000, "uust"));
        let msg = ExecuteMsg::JoinGame { game_id: 1 };
        let _res = execute(deps.as_mut(), mock_env(), unauth_info, msg).unwrap();

        // X to [0,0]
        let unauth_info = mock_info("anyone", &coins(100, "uust"));
        let msg = ExecuteMsg::MakeMove {
            game_id: 1,
            x: 0,
            y: 0,
        };
        let _res = execute(deps.as_mut(), mock_env(), unauth_info, msg).unwrap();

        // O to [0,1]
        let unauth_info = mock_info("anyone_else", &coins(100, "uust"));
        let msg = ExecuteMsg::MakeMove {
            game_id: 1,
            x: 0,
            y: 1,
        };
        let _res = execute(deps.as_mut(), mock_env(), unauth_info, msg).unwrap();

        // X to [1,0]
        let unauth_info = mock_info("anyone", &coins(100, "uust"));
        let msg = ExecuteMsg::MakeMove {
            game_id: 1,
            x: 1,
            y: 0,
        };
        let _res = execute(deps.as_mut(), mock_env(), unauth_info, msg).unwrap();

        // O to [1,1]
        let unauth_info = mock_info("anyone_else", &coins(100, "uust"));
        let msg = ExecuteMsg::MakeMove {
            game_id: 1,
            x: 1,
            y: 1,
        };
        let _res = execute(deps.as_mut(), mock_env(), unauth_info, msg).unwrap();

        // X to [2,0]
        let unauth_info = mock_info("anyone", &coins(100, "uust"));
        let msg = ExecuteMsg::MakeMove {
            game_id: 1,
            x: 2,
            y: 0,
        };
        let _res = execute(deps.as_mut(), mock_env(), unauth_info, msg).unwrap();

        // O to [3,3]
        let unauth_info = mock_info("anyone_else", &coins(100, "uust"));
        let msg = ExecuteMsg::MakeMove {
            game_id: 1,
            x: 3,
            y: 3,
        };
        let _res = execute(deps.as_mut(), mock_env(), unauth_info, msg).unwrap();

        // X to [3,0]
        let unauth_info = mock_info("anyone", &coins(100, "uust"));
        let msg = ExecuteMsg::MakeMove {
            game_id: 1,
            x: 3,
            y: 0,
        };
        let _res = execute(deps.as_mut(), mock_env(), unauth_info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGameById { id: 1 }).unwrap();
        let value: GameResponse = from_binary(&res).unwrap();
        assert_eq!(GameState::Completed, value.game.state);
        assert_eq!("anyone", value.game.winner.to_string());

        // Check leaderboard
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetLeaderboard {}).unwrap();
        let value: LeaderboardResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.entries[0].win_count);
        assert_eq!("anyone", value.entries[0].player);
    }
}
