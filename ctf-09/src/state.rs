use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct State {
    pub owner: Addr,
    pub total_staked: Uint128,
    pub global_index: Decimal,
}

#[cw_serde]
pub struct UserRewardInfo {
    pub staked_amount: Uint128,
    pub user_index: Decimal,
    pub pending_rewards: Uint128,
}

pub const STATE: Item<State> = Item::new("state");

pub const USERS: Map<&Addr, UserRewardInfo> = Map::new("users");
