#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal, Symbol, TryFromVal, Val, Vec};

use crate::errors::PeerXError;
use crate::storage::ADMIN_KEY;
use crate::{CounterContract, CounterContractClient, Metrics};

fn setup() -> (Env, Address, Address, CounterContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CounterContract);
    let client = CounterContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.as_contract(&contract_id, || {
        env.storage().persistent().set(&ADMIN_KEY, &admin);
    });

    let role = Address::generate(&env);
    client.set_read_only_role(&admin, &role);

    (env, admin, role, client)
}

#[test]
fn read_only_role_can_call_allowlisted_read_function() {
    let (env, _admin, role, client) = setup();

    let result: Val = client.invoke_read(&role, &Symbol::new(&env, "get_metrics"), &Vec::new(&env));

    // Decodes without panicking - proves the dispatch path actually wired
    // through to the real `get_metrics` entry point and back.
    let _metrics = Metrics::try_from_val(&env, &result).unwrap();
}

#[test]
fn non_role_address_cannot_invoke_read() {
    let (env, _admin, _role, client) = setup();
    let impostor = Address::generate(&env);

    let result = client.try_invoke_read(&impostor, &Symbol::new(&env, "get_metrics"), &Vec::new(&env));

    assert_eq!(result, Err(Ok(PeerXError::NotReadOnlyRole)));
}

#[test]
fn read_only_role_cannot_mutate_state() {
    let (env, _admin, role, client) = setup();

    let user = Address::generate(&env);
    let token = Symbol::short("XLM");

    let balance_before = client.balance_of(&token, &user);

    // Attempt to reach a real mutating entry point (`mint`) through the
    // read-only dispatch path.
    let mut mint_args: Vec<Val> = Vec::new(&env);
    mint_args.push_back(token.clone().into_val(&env));
    mint_args.push_back(user.clone().into_val(&env));
    mint_args.push_back(1_000_i128.into_val(&env));

    let result = client.try_invoke_read(&role, &Symbol::new(&env, "mint"), &mint_args);

    assert_eq!(result, Err(Ok(PeerXError::UnsupportedReadOnlyFunction)));

    // State must be provably unchanged - `mint` never ran.
    let balance_after = client.balance_of(&token, &user);
    assert_eq!(balance_before, balance_after);
}

#[test]
fn read_only_role_cannot_reach_add_liquidity() {
    let (env, _admin, role, client) = setup();

    let result = client.try_invoke_read(&role, &Symbol::new(&env, "add_liquidity"), &Vec::new(&env));

    assert_eq!(result, Err(Ok(PeerXError::UnsupportedReadOnlyFunction)));
}
