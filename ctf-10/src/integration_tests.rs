#[cfg(test)]
pub mod tests {
    use crate::{
        msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
        state::{Config, Whitelist},
    };
    use cosmwasm_std::{Addr, Empty};

    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    pub fn challenge_code() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply);
        Box::new(contract)
    }

    fn cw721_code() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw721_base::entry::execute,
            cw721_base::entry::instantiate,
            cw721_base::entry::query,
        );
        Box::new(contract)
    }

    pub const ADMIN: &str = "admin";
    pub const USER1: &str = "user1";
    pub const USER2: &str = "user2";
    pub const USER3: &str = "user3";

    pub fn proper_instantiate() -> (App, Addr) {
        let mut app = App::default();
        let challenge_id = app.store_code(challenge_code());
        let cw_721_id = app.store_code(cw721_code());

        // Init challenge
        let challenge_inst = InstantiateMsg {
            cw721_code_id: cw_721_id,
            mint_per_user: 3,
            whitelisted_users: vec![USER1.to_owned(), USER2.to_owned(), USER3.to_owned()],
        };

        let contract_addr = app
            .instantiate_contract(
                challenge_id,
                Addr::unchecked(ADMIN),
                &challenge_inst,
                &[],
                "test",
                None,
            )
            .unwrap();

        (app, contract_addr)
    }

    #[test]
    fn basic_flow() {
        let (mut app, contract_addr) = proper_instantiate();

        // query config
        let config: Config = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::Config {})
            .unwrap();

        // query whitelisted users
        let whitelist: Whitelist = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::Whitelist {})
            .unwrap();

        assert!(whitelist.users.contains(&USER1.to_owned()));
        assert!(whitelist.users.contains(&USER2.to_owned()));
        assert!(whitelist.users.contains(&USER3.to_owned()));

        let user4 = "user4";

        // mint to non-whitelisted user
        app.execute_contract(
            Addr::unchecked(user4),
            contract_addr.clone(),
            &ExecuteMsg::Mint {},
            &[],
        )
        .unwrap_err();

        // mint to whitelisted user until max limit
        assert_eq!(config.mint_per_user, 3);

        app.execute_contract(
            Addr::unchecked(USER1),
            contract_addr.clone(),
            &ExecuteMsg::Mint {},
            &[],
        )
        .unwrap();
        app.execute_contract(
            Addr::unchecked(USER1),
            contract_addr.clone(),
            &ExecuteMsg::Mint {},
            &[],
        )
        .unwrap();
        app.execute_contract(
            Addr::unchecked(USER1),
            contract_addr.clone(),
            &ExecuteMsg::Mint {},
            &[],
        )
        .unwrap();

        // exceed max limit fails
        app.execute_contract(
            Addr::unchecked(USER1),
            contract_addr.clone(),
            &ExecuteMsg::Mint {},
            &[],
        )
        .unwrap_err();

        // other users can mint freely
        app.execute_contract(
            Addr::unchecked(USER2),
            contract_addr.clone(),
            &ExecuteMsg::Mint {},
            &[],
        )
        .unwrap();

        // ensure total tokens increases
        let config: Config = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Config {})
            .unwrap();

        assert_eq!(config.total_tokens, 4);
    }
}
