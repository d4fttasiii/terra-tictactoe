use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

use crate::state::GameState;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Locked")]
    Locked {},

    #[error("InvalidDimension")]
    InvalidDimension {},

    #[error("GameNotFound")]
    GameNotFound {},

    #[error("GameCannotBeCancelled")]
    GameCannotBeCancelled {},

    #[error("NotAllowedToJoin")]
    NotAllowedToJoin { },

    #[error("NotAllowedInCurrentState")]
    NotAllowedInCurrentState { state: GameState },
    
    #[error("NotYourTurn")]
    NotYourTurn {},

    #[error("MoveNotAllow")]
    MoveNotAllow {},

    #[error("BetDenomInvalid")]
    BetDenomInvalid {},
    
    #[error("BetAmounTooLow")]
    BetAmounTooLow {},
    
    #[error("PriceCannotBeWithdrawn")]
    PriceCannotBeWithdrawn {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
