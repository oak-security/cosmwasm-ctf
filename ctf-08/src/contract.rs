#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order, Reply,
    Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw721::{ApprovalResponse, Cw721ExecuteMsg, Cw721QueryMsg, OwnerOfResponse};
use cw_utils::must_pay;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, Sale, Trade, CONFIG, OPERATIONS, SALES, TRADES};

pub const DENOM: &str = "uawesome";
pub const TRADE_REPLY: u64 = 1;
pub const SALE_REPLY: u64 = 2;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        nft_contract: deps.api.addr_validate(&msg.nft_address)?,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("NFT", config.nft_contract))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::BuyNFT { id } => exec_buy(deps, env, info, id),
        ExecuteMsg::NewSale {
            id,
            price,
            tradable,
        } => exec_new_sale(deps, env, info, id, price, tradable),
        ExecuteMsg::CancelSale { id } => exec_cancel_sale(deps, info, id),
        ExecuteMsg::NewTrade { target, offered } => {
            exec_new_trade(deps, env, info, target, offered)
        }
        ExecuteMsg::AcceptTrade { id, trader } => exec_accept_trade(deps, info, id, trader),
        ExecuteMsg::CancelTrade { id } => exec_cancel_trade(deps, info, id),
    }
}

/// Creates a new sale. This requires the contract to be approved as token operator.
pub fn exec_new_sale(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
    price: Uint128,
    tradable: bool,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let nft_owner: OwnerOfResponse = deps.querier.query_wasm_smart(
        config.nft_contract.to_string(),
        &Cw721QueryMsg::OwnerOf {
            token_id: id.clone(),
            include_expired: Some(false),
        },
    )?;

    if nft_owner.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let new_sale = Sale {
        nft_id: id.clone(),
        price,
        owner: info.sender,
        tradable,
    };
    SALES.save(deps.storage, id.clone(), &new_sale)?;

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.nft_contract.to_string(),
        msg: to_binary(&Cw721ExecuteMsg::TransferNft {
            recipient: env.contract.address.to_string(),
            token_id: id.clone(),
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_attribute("action", "new sale")
        .add_attribute("NFT", id)
        .add_message(msg))
}

/// Purchase an NFT from existing sales.
pub fn exec_buy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: String,
) -> Result<Response, ContractError> {
    let amount = must_pay(&info, DENOM).unwrap();
    let target = SALES.load(deps.storage, id.clone())?;

    if amount != target.price {
        return Err(ContractError::IncorrectPayment {
            price: target.price,
        });
    }

    let config = CONFIG.load(deps.storage)?;

    let submsg = SubMsg::reply_on_success(
        WasmMsg::Execute {
            contract_addr: config.nft_contract.to_string(),
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: info.sender.to_string(),
                token_id: target.nft_id.clone(),
            })?,
            funds: vec![],
        },
        SALE_REPLY,
    );

    let payment = BankMsg::Send {
        to_address: target.owner.to_string(),
        amount: vec![Coin {
            denom: DENOM.to_string(),
            amount: target.price,
        }],
    };

    SALES.remove(deps.storage, id);

    Ok(Response::new()
        .add_attribute("action", "NFT bought")
        .add_attribute("NFT", target.nft_id)
        .add_message(payment)
        .add_submessage(submsg))
}

/// Cancel an existing sale.
pub fn exec_cancel_sale(
    deps: DepsMut,
    info: MessageInfo,
    id: String,
) -> Result<Response, ContractError> {
    let target = SALES.load(deps.storage, id.clone())?;

    if target.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let config = CONFIG.load(deps.storage)?;

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.nft_contract.to_string(),
        msg: to_binary(&Cw721ExecuteMsg::TransferNft {
            recipient: target.owner.to_string(),
            token_id: id.clone(),
        })?,
        funds: vec![],
    });

    SALES.remove(deps.storage, id.clone());

    Ok(Response::new()
        .add_attribute("action", "cancel sale")
        .add_attribute("NFT", id)
        .add_message(msg))
}

/// Create a new trade. This requires the contract to be approved as token operator to succeed at the later accept phase.
pub fn exec_new_trade(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asked_id: String,
    offered_id: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let nft_owner: OwnerOfResponse = deps.querier.query_wasm_smart(
        config.nft_contract.to_string(),
        &Cw721QueryMsg::OwnerOf {
            token_id: offered_id.clone(),
            include_expired: Some(false),
        },
    )?;

    if nft_owner.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // ensure contract have approval
    let _: ApprovalResponse = deps
        .querier
        .query_wasm_smart(
            config.nft_contract.to_string(),
            &Cw721QueryMsg::Approval {
                token_id: offered_id.clone(),
                spender: env.contract.address.to_string(),
                include_expired: None,
            },
        )
        .unwrap();

    let sale = SALES.load(deps.storage, asked_id.clone())?;

    if !sale.tradable {
        return Err(ContractError::NonTradeable {});
    }

    let new_trade = Trade {
        asked_id: asked_id.clone(),
        to_trade_id: offered_id,
        trader: info.sender,
    };

    TRADES.save(
        deps.storage,
        (asked_id.clone(), new_trade.trader.to_string()),
        &new_trade,
    )?;

    Ok(Response::new()
        .add_attribute("action", "new trade")
        .add_attribute("Asked NFT", asked_id))
}

/// Entry point for the sale owner to accept trades.
pub fn exec_accept_trade(
    deps: DepsMut,
    info: MessageInfo,
    asked_id: String,
    trader: String,
) -> Result<Response, ContractError> {
    let trade = TRADES.load(deps.storage, (asked_id.clone(), trader))?;
    let sale = SALES.load(deps.storage, asked_id)?;

    if sale.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let config = CONFIG.load(deps.storage)?;

    // Asked
    let mut submsgs = vec![SubMsg::reply_always(
        WasmMsg::Execute {
            contract_addr: config.nft_contract.to_string(),
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: trade.trader.to_string(),
                token_id: trade.asked_id.clone(),
            })?,
            funds: vec![],
        },
        TRADE_REPLY,
    )];

    // Offered
    submsgs.push(SubMsg::reply_always(
        WasmMsg::Execute {
            contract_addr: config.nft_contract.to_string(),
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: sale.owner.to_string(),
                token_id: trade.to_trade_id.clone(),
            })?,
            funds: vec![],
        },
        TRADE_REPLY,
    ));

    TRADES.remove(
        deps.storage,
        (trade.asked_id.clone(), trade.trader.to_string()),
    );

    Ok(Response::new()
        .add_attribute("action", "NFT traded")
        .add_attribute("NFT asked", trade.asked_id)
        .add_attribute("NFT offered", trade.to_trade_id)
        .add_submessages(submsgs))
}

/// Entrypoint for trader to cancel existing trades.
pub fn exec_cancel_trade(
    deps: DepsMut,
    info: MessageInfo,
    asked_id: String,
) -> Result<Response, ContractError> {
    let target = TRADES.load(deps.storage, (asked_id.clone(), info.sender.to_string()))?;

    if target.trader != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let config = CONFIG.load(deps.storage)?;

    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.nft_contract.to_string(),
        msg: to_binary(&Cw721ExecuteMsg::TransferNft {
            recipient: target.trader.to_string(),
            token_id: target.to_trade_id,
        })?,
        funds: vec![],
    });

    TRADES.remove(deps.storage, (target.asked_id, target.trader.to_string()));

    Ok(Response::new()
        .add_attribute("action", "cancel sale")
        .add_attribute("NFT", asked_id)
        .add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    let mut ops = OPERATIONS.load(deps.storage).unwrap_or_default();
    match reply.id {
        SALE_REPLY => {
            ops.n_sales += Uint128::one();
            OPERATIONS.save(deps.storage, &ops)?;

            Ok(Response::new().add_attribute("Operation", "sale"))
        }
        TRADE_REPLY => {
            ops.n_trades += Uint128::one();
            OPERATIONS.save(deps.storage, &ops)?;

            Ok(Response::new().add_attribute("Operation", "trade"))
        }
        _ => Err(ContractError::UnrecognizedReply {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetSale { id } => to_binary(&get_sale(deps, id)?),
        QueryMsg::GetSalesBySeller {
            seller,
            from_index,
            limit,
        } => to_binary(&get_sales_seller(deps, seller, from_index, limit)?),
        QueryMsg::GetTrade { id, trader } => to_binary(&get_trade(deps, id, trader)?),
        QueryMsg::GetTradesByTrader {
            trader,
            from_index,
            limit,
        } => to_binary(&get_trades_trader(deps, trader, from_index, limit)?),
    }
}

/// Returns sale information for specified id.
pub fn get_sale(deps: Deps, id: String) -> StdResult<Sale> {
    let sale = SALES.load(deps.storage, id)?;
    Ok(sale)
}

/// Returns paginated sales from a seller.
pub fn get_sales_seller(
    deps: Deps,
    seller: String,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> StdResult<Vec<Sale>> {
    let from_index = from_index.unwrap_or(0);
    let limit = limit.unwrap_or(10);

    let sales: StdResult<Vec<Sale>> = SALES
        .range(deps.storage, None, None, Order::Ascending)
        .skip(from_index as usize)
        .take(limit as usize)
        .filter(|item| item.as_ref().unwrap().1.owner == seller)
        .map(|item| item.map(|(_, sale)| sale))
        .collect();
    sales
}

/// Returns trade information for specified id.
pub fn get_trade(deps: Deps, id: String, trader: String) -> StdResult<Trade> {
    let trade = TRADES.load(deps.storage, (id, trader))?;

    Ok(trade)
}

/// Returns paginated trades from a trader.
pub fn get_trades_trader(
    deps: Deps,
    trader: String,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> StdResult<Vec<Trade>> {
    let from_index = from_index.unwrap_or(0);
    let limit = limit.unwrap_or(10);

    let trades: StdResult<Vec<Trade>> = TRADES
        .prefix(trader)
        .range(deps.storage, None, None, Order::Ascending)
        .skip(from_index as usize)
        .take(limit as usize)
        .map(|item| item.map(|(_, trade)| trade))
        .collect();

    trades
}
