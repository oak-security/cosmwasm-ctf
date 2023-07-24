use cosmwasm_schema::cw_serde;

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    /// NFT contract address
    pub nft_contract: Addr,
    /// Mint per user
    pub mint_per_user: u64,
    /// Total minted tokens
    pub total_tokens: u128,
}

#[cw_serde]
pub struct Whitelist {
    /// whitelisted users to receive NFTs
    pub users: Vec<String>,
}

#[cw_serde]
pub struct MintedNFT {
    /// NFT identifier value
    pub nft_id: u128,
    /// Timestamp
    pub timestamp: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const WHITELIST: Item<Whitelist> = Item::new("whitelist");
pub const MINT_PER_USER: Map<&Addr, Vec<MintedNFT>> = Map::new("mint_per_user");
