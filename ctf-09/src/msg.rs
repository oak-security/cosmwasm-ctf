use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

use crate::state::{State, UserRewardInfo};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    /// Owner increase global index reward
    IncreaseReward {},
    /// User deposits
    Deposit {},
    /// User withdraws
    Withdraw { amount: Uint128 },
    /// User claim rewards
    ClaimRewards {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Query contract state
    #[returns(State)]
    State {},

    /// Query user reward information
    #[returns(UserRewardInfo)]
    User { user: String },
}
