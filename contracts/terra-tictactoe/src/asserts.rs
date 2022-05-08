use cosmwasm_std::{Coin, Deps};

use crate::state::CONFIG;
use crate::ContractError;

pub fn assert_is_locked(deps: Deps) -> Result<(), ContractError> {
    let state = CONFIG.load(deps.storage)?;
    if state.locked {
        return Err(ContractError::Locked {});
    }

    Ok(())
}

pub fn assert_host_bet(deps: Deps, bet: &Coin) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.min_bet.denom != bet.denom {
        return Err(ContractError::BetDenomInvalid {});
    }
    if config.min_bet.amount > bet.amount {
        return Err(ContractError::BetAmounTooLow {});
    }

    Ok(())
}
