use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Zero amount withdrawal is disallowed")]
    ZeroAmountWithdrawal {},

    #[error("No rewards to claim")]
    ZeroRewardClaim {},

    #[error("Withdraw amount higher than available balance")]
    WithdrawTooMuch {},

    #[error("Caller did not provide requested funds")]
    NoDenomSent {},

    #[error("No user staked")]
    NoUserStake {},
}
