#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use cw_utils::must_pay;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Balance, Config, BALANCES, CONFIG};

const DECIMAL_OFFSET: u32 = 1;
pub const DENOM: &str = "uawesome";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        total_supply: Uint128::zero(),
    };

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint {} => mint(deps, env, info),
        ExecuteMsg::Burn { shares } => burn(deps, env, info, shares),
    }
}

/// Entry point for users to mint shares
pub fn mint(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let amount = must_pay(&info, DENOM).unwrap();

    let mut config = CONFIG.load(deps.storage).unwrap();

    let contract_balance = deps
        .querier
        .query_balance(env.contract.address.to_string(), DENOM)
        .unwrap();

    let total_assets = contract_balance.amount - amount;
    let total_supply = config.total_supply;

    // share = asset * (total supply + 10^offset) / (total assets + 1)
    // ref: https://github.com/OpenZeppelin/openzeppelin-contracts/blob/f347b410cf6aeeaaf5197e1fece139c793c03b2b/contracts/token/ERC20/extensions/ERC4626.sol#L226-L231
    let mint_amount = amount.multiply_ratio(
        total_supply + Uint128::new(10_u32.pow(DECIMAL_OFFSET).into()),
        total_assets + Uint128::one(),
    );

    if mint_amount.is_zero() {
        return Err(ContractError::ZeroAmountNotAllowed {});
    }

    // increase total supply
    config.total_supply += mint_amount;
    CONFIG.save(deps.storage, &config)?;

    // increase user balance
    let mut user = BALANCES
        .load(deps.storage, &info.sender)
        .unwrap_or_default();
    user.amount += mint_amount;
    BALANCES.save(deps.storage, &info.sender, &user)?;

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("user", info.sender.to_string())
        .add_attribute("asset", amount.to_string())
        .add_attribute("shares", mint_amount.to_string()))
}

/// Entry point for users to burn shares
pub fn burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    shares: Uint128,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage).unwrap();

    let contract_balance = deps
        .querier
        .query_balance(env.contract.address.to_string(), DENOM)
        .unwrap();

    let total_assets = contract_balance.amount;
    let total_supply = config.total_supply;

    // asset = share * (total assets + 1) / (total supply * 10^3)
    // ref: https://github.com/OpenZeppelin/openzeppelin-contracts/blob/f347b410cf6aeeaaf5197e1fece139c793c03b2b/contracts/token/ERC20/extensions/ERC4626.sol#L233-L238

    let asset_to_return = shares.multiply_ratio(
        total_assets + Uint128::one(),
        total_supply + Uint128::new(10_u32.pow(DECIMAL_OFFSET).into()),
    );

    if asset_to_return.is_zero() {
        return Err(ContractError::ZeroAmountNotAllowed {});
    }

    // decrease total supply
    config.total_supply -= shares;
    CONFIG.save(deps.storage, &config)?;

    // decrease user balance
    let mut user = BALANCES.load(deps.storage, &info.sender)?;
    user.amount -= shares;
    BALANCES.save(deps.storage, &info.sender, &user)?;

    let msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: coins(asset_to_return.u128(), DENOM),
    };

    Ok(Response::new()
        .add_attribute("action", "burn")
        .add_attribute("user", info.sender.to_string())
        .add_attribute("asset", asset_to_return.to_string())
        .add_attribute("shares", shares.to_string())
        .add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::UserBalance { address } => to_binary(&query_user(deps, address)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage).unwrap();
    Ok(config)
}

pub fn query_user(deps: Deps, address: String) -> StdResult<Balance> {
    let user = deps.api.addr_validate(&address).unwrap();
    let balance = BALANCES.load(deps.storage, &user).unwrap_or_default();
    Ok(balance)
}
