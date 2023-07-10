#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, wasm_instantiate, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo,
    Reply, Response, StdResult, SubMsg, WasmMsg,
};
use cw721::TokensResponse;
use cw721_base::{
    ExecuteMsg as Cw721ExecuteMsg, InstantiateMsg as Cw721InstantiateMsg, QueryMsg as Cw721QueryMsg,
};
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, Whitelist, CONFIG, WHITELIST};

pub const DENOM: &str = "uawesome";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // nft contract init msg
    let cw721_init_msg = Cw721InstantiateMsg {
        name: "Awesome Wasm".to_owned(),
        symbol: "AWESOME".to_owned(),
        minter: env.contract.address.to_string(),
    };

    let submsg = SubMsg::reply_on_success(
        wasm_instantiate(
            msg.cw721_code_id,
            &cw721_init_msg,
            vec![],
            "awesome nft contract".to_owned(),
        )
        .unwrap(),
        1,
    );

    // store config
    let config = Config {
        nft_contract: Addr::unchecked(""),
        mint_per_user: msg.mint_per_user,
        total_tokens: 0,
    };

    CONFIG.save(deps.storage, &config)?;

    // validate and store whitelisted users
    let _ = msg
        .whitelisted_users
        .iter()
        .map(|user| deps.api.addr_validate(user).unwrap());

    let whitelist = Whitelist {
        users: msg.whitelisted_users,
    };

    WHITELIST.save(deps.storage, &whitelist)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("mint_per_user", msg.mint_per_user.to_string())
        .add_attribute("total_whitelisted_users", whitelist.users.len().to_string())
        .add_submessage(submsg))
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
    }
}

/// Mint NFT to recipient
pub fn mint(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // check user is in whitelist
    let users = WHITELIST.load(deps.storage)?.users;
    let is_whitelisted = users.iter().any(|user| user == &info.sender.to_string());
    if !is_whitelisted {
        return Err(ContractError::NotWhitelisted {});
    }

    let tokens_response: TokensResponse = deps.querier.query_wasm_smart(
        config.nft_contract.to_string(),
        &Cw721QueryMsg::Tokens::<Empty> {
            owner: info.sender.to_string(),
            start_after: None,
            limit: None,
        },
    )?;

    // ensure mint per user limit is not exceeded
    if tokens_response.tokens.len() >= config.mint_per_user as usize {
        return Err(ContractError::MaxLimitExceeded {});
    }

    let token_id = config.total_tokens;

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.nft_contract.to_string(),
        msg: to_binary(&Cw721ExecuteMsg::Mint::<Empty, Empty> {
            token_id: token_id.to_string(),
            owner: info.sender.to_string(),
            token_uri: None,
            extension: Empty {},
        })?,
        funds: vec![],
    });

    // increment total tokens
    config.total_tokens += 1;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("recipient", info.sender.to_string())
        .add_attribute("token_id", token_id.to_string())
        .add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        1 => {
            let res = parse_reply_instantiate_data(reply).unwrap();
            let mut config = CONFIG.load(deps.storage)?;
            let nft_contract = deps.api.addr_validate(&res.contract_address).unwrap();
            config.nft_contract = nft_contract;
            CONFIG.save(deps.storage, &config)?;
            Ok(Response::default())
        }
        _ => Ok(Response::default()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Whitelist {} => to_binary(&query_whitelist(deps)?),
    }
}

/// Returns contract configuration
fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

/// Returns whitelisted users
fn query_whitelist(deps: Deps) -> StdResult<Whitelist> {
    let whitelist = WHITELIST.load(deps.storage)?;
    Ok(whitelist)
}
