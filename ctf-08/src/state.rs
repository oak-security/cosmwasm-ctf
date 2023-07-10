use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub nft_contract: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Sale {
    pub nft_id: String,
    pub price: Uint128,
    pub owner: Addr,
    pub tradable: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Trade {
    pub asked_id: String,
    pub to_trade_id: String,
    pub trader: Addr,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Operations {
    pub n_trades: Uint128,
    pub n_sales: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const SALES: Map<String, Sale> = Map::new("sales");
pub const TRADES: Map<(String, String), Trade> = Map::new("trades");
pub const OPERATIONS: Item<Operations> = Item::new("operations");
