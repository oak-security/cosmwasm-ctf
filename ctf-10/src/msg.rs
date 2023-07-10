use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub cw721_code_id: u64,
    pub mint_per_user: u64,
    pub whitelisted_users: Vec<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Mint {},
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Whitelist {},
}
