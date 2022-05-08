use cosmwasm_std::Addr;

use crate::state::Game;
use crate::ContractError;

pub fn get_mark_for_cell(game: &Game, x: usize, y: usize) -> Result<i8, ContractError> {
    if game.grid[x][y] != 0 || game.grid[x][y] == -1 {
        return Err(ContractError::MoveNotAllow {});
    } else if game.host.to_string() == game.next_player {
        Ok(1)
    } else {
        Ok(100)
    }
}

pub fn get_next_player(game: &Game) -> Result<Addr, ContractError> {
    if game.next_player == game.host {
        Ok(game.opponent.clone())
    } else {
        Ok(game.host.clone())
    }
}

pub fn is_game_completed(
    game: &Game,
    dimension: u16,
    threshold: u16,
) -> Result<bool, ContractError> {
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
