#[cfg(test)]
pub mod tests {
    use crate::{
        contract::{DENOM, REWARD_DENOM},
        msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
        state::{State, UserRewardInfo},
    };
    use cosmwasm_std::{coin, Addr, Decimal, Empty, Uint128};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    pub fn challenge_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    pub const OWNER: &str = "owner";
    pub const USER: &str = "user";
    pub const USER2: &str = "user2";

    pub fn proper_instantiate() -> (App, Addr) {
        let mut app = App::default();
        let cw_template_id = app.store_code(challenge_contract());

        // init contract
        let msg = InstantiateMsg {};
        let contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(OWNER),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        // query state
        let state: State = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::State {})
            .unwrap();

        assert_eq!(
            state,
            State {
                owner: Addr::unchecked(OWNER),
                total_staked: Uint128::zero(),
                global_index: Decimal::zero()
            }
        );

        // mint funds to owner
        app = mint_reward_tokens(app, OWNER.to_owned(), Uint128::new(10_000));
        app = mint_tokens(app, OWNER.to_owned(), Uint128::new(10_000));

        // mint funds to user
        app = mint_tokens(app, USER.to_owned(), Uint128::new(10_000));

        // only owner can increase global index
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &ExecuteMsg::IncreaseReward {},
            &[coin(10_000, DENOM)],
        )
        .unwrap_err();

        // only reward denom is accepted
        app.execute_contract(
            Addr::unchecked(OWNER),
            contract_addr.clone(),
            &ExecuteMsg::IncreaseReward {},
            &[coin(10_000, DENOM)],
        )
        .unwrap_err();

        // cannot increase reward if no one staked
        app.execute_contract(
            Addr::unchecked(OWNER),
            contract_addr.clone(),
            &ExecuteMsg::IncreaseReward {},
            &[coin(10_000, REWARD_DENOM)],
        )
        .unwrap_err();

        // cannot deposit reward denom
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &ExecuteMsg::Deposit {},
            &[coin(10_000, REWARD_DENOM)],
        )
        .unwrap_err();

        // user deposit funds
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &ExecuteMsg::Deposit {},
            &[coin(10_000, DENOM)],
        )
        .unwrap();

        // owner increases global index
        app.execute_contract(
            Addr::unchecked(OWNER),
            contract_addr.clone(),
            &ExecuteMsg::IncreaseReward {},
            &[coin(10_000, REWARD_DENOM)],
        )
        .unwrap();

        // query state
        let state: State = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::State {})
            .unwrap();

        assert_eq!(
            state,
            State {
                owner: Addr::unchecked(OWNER),
                total_staked: Uint128::new(10_000),
                global_index: Decimal::one()
            }
        );

        // query user info
        let user_info: UserRewardInfo = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::User {
                    user: USER.to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            user_info,
            UserRewardInfo {
                staked_amount: Uint128::new(10_000),
                user_index: state.global_index,
                pending_rewards: Uint128::new(10_000)
            }
        );

        (app, contract_addr)
    }

    pub fn mint_tokens(mut app: App, recipient: String, amount: Uint128) -> App {
        app.sudo(cw_multi_test::SudoMsg::Bank(
            cw_multi_test::BankSudo::Mint {
                to_address: recipient,
                amount: vec![coin(amount.u128(), DENOM)],
            },
        ))
        .unwrap();
        app
    }

    pub fn mint_reward_tokens(mut app: App, recipient: String, amount: Uint128) -> App {
        app.sudo(cw_multi_test::SudoMsg::Bank(
            cw_multi_test::BankSudo::Mint {
                to_address: recipient,
                amount: vec![coin(amount.u128(), REWARD_DENOM)],
            },
        ))
        .unwrap();
        app
    }

    #[test]
    fn basic_flow() {
        let (mut app, contract_addr) = proper_instantiate();

        // new user2 join
        app = mint_tokens(app, USER2.to_owned(), Uint128::new(10_000));
        app.execute_contract(
            Addr::unchecked(USER2),
            contract_addr.clone(),
            &ExecuteMsg::Deposit {},
            &[coin(10_000, DENOM)],
        )
        .unwrap();

        // owner increases reward
        app = mint_reward_tokens(app, OWNER.to_owned(), Uint128::new(10_000));
        app.execute_contract(
            Addr::unchecked(OWNER),
            contract_addr.clone(),
            &ExecuteMsg::IncreaseReward {},
            &[coin(10_000, REWARD_DENOM)],
        )
        .unwrap();

        // query user1 info
        let user_info: UserRewardInfo = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::User {
                    user: USER.to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_info.pending_rewards, Uint128::new(15_000));

        // query user2 info
        let user_info: UserRewardInfo = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::User {
                    user: USER2.to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_info.pending_rewards, Uint128::new(5_000));

        // user1 claim rewards
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &ExecuteMsg::ClaimRewards {},
            &[],
        )
        .unwrap();

        // cannot claim second time
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &ExecuteMsg::ClaimRewards {},
            &[],
        )
        .unwrap_err();

        // query user1 info
        let user_info: UserRewardInfo = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::User {
                    user: USER.to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_info.pending_rewards, Uint128::zero());

        // query user1 reward balance
        let balance = app
            .wrap()
            .query_balance(USER.to_string(), REWARD_DENOM)
            .unwrap()
            .amount;

        assert_eq!(balance, Uint128::new(15_000));

        // user1 withdraw all funds
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &ExecuteMsg::Withdraw {
                amount: Uint128::new(10_000),
            },
            &[],
        )
        .unwrap();

        // query user1 balance
        let balance = app
            .wrap()
            .query_balance(USER.to_string(), DENOM)
            .unwrap()
            .amount;

        assert_eq!(balance, Uint128::new(10_000));

        // query user info
        let user_info: UserRewardInfo = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::User {
                    user: USER.to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_info.staked_amount, Uint128::zero());

        // query state
        let state: State = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::State {})
            .unwrap();

        assert_eq!(state.total_staked, Uint128::new(10_000));

        // owner increase reward
        app = mint_reward_tokens(app, OWNER.to_owned(), Uint128::new(10_000));
        app.execute_contract(
            Addr::unchecked(OWNER),
            contract_addr.clone(),
            &ExecuteMsg::IncreaseReward {},
            &[coin(10_000, REWARD_DENOM)],
        )
        .unwrap();

        // user1 will not receive any rewards anymore
        let user_info: UserRewardInfo = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::User {
                    user: USER.to_string(),
                },
            )
            .unwrap();
        assert_eq!(user_info.pending_rewards, Uint128::zero());
        assert_eq!(user_info.staked_amount, Uint128::zero());

        // user2 gets all reward
        let user_info: UserRewardInfo = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::User {
                    user: USER2.to_string(),
                },
            )
            .unwrap();
        assert_eq!(user_info.pending_rewards, Uint128::new(15_000));

        // user2 perform full withdrawal
        app.execute_contract(
            Addr::unchecked(USER2),
            contract_addr.clone(),
            &ExecuteMsg::Withdraw {
                amount: user_info.staked_amount,
            },
            &[],
        )
        .unwrap();

        // user2 claim rewards
        app.execute_contract(
            Addr::unchecked(USER2),
            contract_addr.clone(),
            &ExecuteMsg::ClaimRewards {},
            &[],
        )
        .unwrap();

        // contract should have zero funds
        let balance = app
            .wrap()
            .query_balance(contract_addr.to_string(), DENOM)
            .unwrap()
            .amount;

        assert_eq!(balance, Uint128::zero());

        // contract should have zero reward denom
        let balance = app
            .wrap()
            .query_balance(contract_addr.to_string(), REWARD_DENOM)
            .unwrap()
            .amount;

        assert_eq!(balance, Uint128::zero());

        // user2 receives reward denom
        let balance = app
            .wrap()
            .query_balance(USER2.to_string(), REWARD_DENOM)
            .unwrap()
            .amount;

        assert_eq!(balance, user_info.pending_rewards);

        // user2 receives funds
        let balance = app
            .wrap()
            .query_balance(USER2.to_string(), DENOM)
            .unwrap()
            .amount;

        assert_eq!(balance, user_info.staked_amount);

        // query user2 info
        let user_info: UserRewardInfo = app
            .wrap()
            .query_wasm_smart(
                contract_addr,
                &QueryMsg::User {
                    user: USER2.to_string(),
                },
            )
            .unwrap();

        assert_eq!(user_info.staked_amount, Uint128::zero());
        assert_eq!(user_info.pending_rewards, Uint128::zero());
    }
}
