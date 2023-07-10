use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub nft_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    BuyNFT {
        id: String,
    },
    NewSale {
        id: String,
        price: Uint128,
        tradable: bool,
    },
    CancelSale {
        id: String,
    },
    NewTrade {
        target: String,
        offered: String,
    },
    AcceptTrade {
        id: String,
        trader: String,
    },
    CancelTrade {
        id: String,
    },
}

#[cw_serde]
pub enum QueryMsg {
    GetSale {
        id: String,
    },
    GetSalesBySeller {
        seller: String,
        from_index: Option<u64>,
        limit: Option<u64>,
    },
    GetTrade {
        id: String,
        trader: String,
    },
    GetTradesByTrader {
        trader: String,
        from_index: Option<u64>,
        limit: Option<u64>,
    },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetCountResponse {
    pub count: i32,
}
