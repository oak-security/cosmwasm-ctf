use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("User is not whitelisted")]
    NotWhitelisted {},

    #[error("Max mint limit exceeded")]
    MaxLimitExceeded {},
}
