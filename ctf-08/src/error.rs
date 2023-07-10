use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Payment is not the same as the price {price}")]
    IncorrectPayment { price: Uint128 },

    #[error("Selected NFT's offer is not tradeable")]
    NonTradeable {},

    #[error("The reply ID is unrecognized")]
    UnrecognizedReply {},
}
