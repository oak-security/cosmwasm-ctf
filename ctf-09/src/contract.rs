#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, BankMsg, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128,
};
use cw0::must_pay;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, UserRewardInfo, STATE, USERS};

pub const DENOM: &str = "uawesome";
pub const REWARD_DENOM: &str = "uoak";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        global_index: Decimal::zero(),
        total_staked: Uint128::zero(),
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::IncreaseReward {} => increase_reward(deps, env, info),
        ExecuteMsg::Deposit {} => deposit(deps, info),
        ExecuteMsg::Withdraw { amount } => withdraw(deps, info, amount),
        ExecuteMsg::ClaimRewards {} => claim_rewards(deps, info),
    }
}

/// Entry point for owner to increase reward
pub fn increase_reward(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;

    let amount = must_pay(&info, REWARD_DENOM).map_err(|_| ContractError::NoDenomSent {})?;

    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    let total_stake = state.total_staked;

    if total_stake.is_zero() {
        // No need to distribute rewards if no one staked
        return Err(ContractError::NoUserStake {});
    }

    state.global_index += Decimal::from_ratio(amount, total_stake);

    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("action", "increase_reward"))
}

/// Entry point for users to deposit funds
pub fn deposit(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let amount = must_pay(&info, DENOM).map_err(|_| ContractError::NoDenomSent {})?;

    let mut state = STATE.load(deps.storage)?;

    let mut user = USERS
        .load(deps.storage, &info.sender)
        .unwrap_or(UserRewardInfo {
            staked_amount: Uint128::zero(),
            user_index: state.global_index,
            pending_rewards: Uint128::zero(),
        });

    // update rewards
    update_rewards(&mut user, &state);

    // increase user amount
    user.staked_amount += amount;

    // increase total staked amount
    state.total_staked += amount;

    USERS.save(deps.storage, &info.sender, &user)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("action", "deposit"))
}

/// Entry point for users to withdraw funds
pub fn withdraw(
    deps: DepsMut,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;

    let mut user = USERS.load(deps.storage, &info.sender)?;

    if amount.is_zero() {
        return Err(ContractError::ZeroAmountWithdrawal {});
    }

    if user.staked_amount < amount {
        return Err(ContractError::WithdrawTooMuch {});
    }

    // update rewards
    update_rewards(&mut user, &state);

    // decrease user amount
    user.staked_amount -= amount;

    // decrease total staked amount
    state.total_staked -= amount;

    USERS.save(deps.storage, &info.sender, &user)?;
    STATE.save(deps.storage, &state)?;

    let msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![coin(amount.u128(), DENOM)],
    };

    Ok(Response::new()
        .add_attribute("action", "withdraw")
        .add_message(msg))
}

/// Entry point for user to claim rewards
pub fn claim_rewards(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let mut user = USERS.load(deps.storage, &info.sender)?;

    let state = STATE.load(deps.storage)?;

    // update rewards
    update_rewards(&mut user, &state);

    let amount = user.pending_rewards;

    // disallow claiming zero rewards
    if amount.is_zero() {
        return Err(ContractError::ZeroRewardClaim {});
    }

    // set pending rewards to zero
    user.pending_rewards = Uint128::zero();

    USERS.save(deps.storage, &info.sender, &user)?;

    let msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![coin(amount.u128(), REWARD_DENOM)],
    };

    Ok(Response::new()
        .add_attribute("action", "claim_reward")
        .add_message(msg))
}

pub fn update_rewards(user: &mut UserRewardInfo, state: &State) {
    // no need update amount if zero
    if user.staked_amount.is_zero() {
        return;
    }

    // calculate pending rewards
    let reward = (state.global_index - user.user_index) * user.staked_amount;
    user.pending_rewards += reward;

    user.user_index = state.global_index;
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::User { user } => to_binary(&query_user(deps, user)?),
    }
}

/// Query contract state
pub fn query_state(deps: Deps) -> StdResult<State> {
    let state = STATE.load(deps.storage)?;
    Ok(state)
}

/// Query user information
pub fn query_user(deps: Deps, user: String) -> StdResult<UserRewardInfo> {
    let user = deps.api.addr_validate(&user).unwrap();
    let state = STATE.load(deps.storage)?;
    let mut user_info = USERS.load(deps.storage, &user)?;
    update_rewards(&mut user_info, &state);
    Ok(user_info)
}
