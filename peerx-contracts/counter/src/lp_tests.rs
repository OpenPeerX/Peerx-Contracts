use super::*;
use crate::portfolio::{Asset, LPPosition};
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, Symbol, Vec};

// ===== LEGACY LP TESTS (XLM/USDC) =====

#[test]
fn test_add_liquidity_first_provider() {
    let env = Env::default();
    let contract_id = env.register(CounterContract, ());
    let client = CounterContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    client.mint(&symbol_short!("XLM"), &user, &1000);
    client.mint(&symbol_short!("USDCSIM"), &user, &1000);

    let lp_tokens = client.add_liquidity(&100, &100, &user);

    assert!(lp_tokens > 0, "LP tokens should be minted");
    assert!(
        lp_tokens >= 99 && lp_tokens <= 101,
        "LP tokens should be approximately 100"
    );

    let positions = client.get_lp_positions(&user);
    assert_eq!(positions.len(), 1, "User should have one LP position");
    let position = positions.get(0).unwrap();
    assert_eq!(position.lp_address, user);
    assert_eq!(position.xlm_deposited, 100);
    assert_eq!(position.usdc_deposited, 100);
    assert_eq!(position.lp_tokens_minted, lp_tokens);

    let user_xlm = client.balance_of(&symbol_short!("XLM"), &user);
    assert_eq!(user_xlm, 900, "User should have 900 XLM remaining");
    let user_usdc = client.balance_of(&symbol_short!("USDCSIM"), &user);
    assert_eq!(user_usdc, 900, "User should have 900 USDC remaining");
}

// ===== MULTI-TOKEN POOL TESTS =====

#[test]
fn test_register_pool() {
    let env = Env::default();
    let contract_id = env.register(CounterContract, ());
    let client = CounterContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    let btc = symbol_short!("BTC");
    let eth = symbol_short!("ETH");

    let pool_id = client.register_pool(&admin, &btc, &eth, &1000, &2000, &30);
    assert_eq!(pool_id, 1);

    let pool = client.get_pool(&pool_id).unwrap();
    assert_eq!(pool.reserve_a, 1000);
    assert_eq!(pool.reserve_b, 2000);
    assert_eq!(pool.fee_tier, 30);
}

#[test]
fn test_pool_add_liquidity() {
    let env = Env::default();
    let contract_id = env.register(CounterContract, ());
    let client = CounterContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let provider = Address::generate(&env);

    let usdt = symbol_short!("USDT");
    let dai = symbol_short!("DAI");

    let pool_id = client.register_pool(&admin, &usdt, &dai, &1000, &1000, &5);
    let lp_tokens = client.pool_add_liquidity(&pool_id, &500, &500, &provider);

    assert!(lp_tokens > 0);

    let pool = client.get_pool(&pool_id).unwrap();
    assert_eq!(pool.reserve_a, 1500);
    assert_eq!(pool.reserve_b, 1500);
}

#[test]
fn test_pool_swap() {
    let env = Env::default();
    let contract_id = env.register(CounterContract, ());
    let client = CounterContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    let token_a = symbol_short!("TOKA");
    let token_b = symbol_short!("TOKB");

    let pool_id = client.register_pool(&admin, &token_a, &token_b, &10000, &10000, &30);

    let trader = Address::generate(&env);
    let amount_out = client.pool_swap(&pool_id, &token_a, &100, &90, &trader);

    assert!(amount_out >= 90);
    assert!(amount_out < 100);
}

#[test]
fn test_pool_remove_liquidity() {
    let env = Env::default();
    let contract_id = env.register(CounterContract, ());
    let client = CounterContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let provider = Address::generate(&env);

    let token_a = symbol_short!("TOKA");
    let token_b = symbol_short!("TOKB");

    let pool_id = client.register_pool(&admin, &token_a, &token_b, &1000, &2000, &30);
    let lp_tokens = client.pool_add_liquidity(&pool_id, &1000, &2000, &provider);

    let half_lp_tokens = lp_tokens / 2;
    let (amount_a, amount_b) = client.pool_remove_liquidity(&pool_id, &half_lp_tokens, &provider);

    assert!(amount_a > 0);
    assert!(amount_b > 0);
}

#[test]
fn test_find_best_route_direct() {
    let env = Env::default();
    let contract_id = env.register(CounterContract, ());
    let client = CounterContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    let xlm = symbol_short!("XLM");
    let usdc = symbol_short!("USDC");

    client.register_pool(&admin, &xlm, &usdc, &10000, &10000, &30);

    let route = client.find_best_route(&xlm, &usdc, &100);
    assert!(route.is_some());

    let r = route.unwrap();
    assert_eq!(r.pools.len(), 1);
    assert_eq!(r.tokens.len(), 2);
    assert!(r.total_price_impact_bps > 0);
}

#[test]
fn test_find_best_route_multihop() {
    let env = Env::default();
    let contract_id = env.register(CounterContract, ());
    let client = CounterContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    let xlm = symbol_short!("XLM");
    let usdc = symbol_short!("USDC");
    let btc = symbol_short!("BTC");

    client.register_pool(&admin, &xlm, &usdc, &10000, &10000, &30);
    client.register_pool(&admin, &usdc, &btc, &10000, &5000, &30);

    let route = client.find_best_route(&xlm, &btc, &100);
    assert!(route.is_some());

    let r = route.unwrap();
    assert_eq!(r.pools.len(), 2);
    assert_eq!(r.tokens.len(), 3);
    assert!(r.total_price_impact_bps > 0);
}

#[test]
fn test_multiple_fee_tiers() {
    let env = Env::default();
    let contract_id = env.register(CounterContract, ());
    let client = CounterContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    let token_a = symbol_short!("TOKA");
    let token_b = symbol_short!("TOKB");

    let pool1 = client.register_pool(&admin, &token_a, &token_b, &10000, &10000, &1);
    let pool2 = client.register_pool(
        &admin,
        &symbol_short!("TOKC"),
        &symbol_short!("TOKD"),
        &10000,
        &10000,
        &5,
    );
    let pool3 = client.register_pool(
        &admin,
        &symbol_short!("TOKE"),
        &symbol_short!("TOKF"),
        &10000,
        &10000,
        &30,
    );

    let p1 = client.get_pool(&pool1).unwrap();
    let p2 = client.get_pool(&pool2).unwrap();
    let p3 = client.get_pool(&pool3).unwrap();

    assert_eq!(p1.fee_tier, 1);
    assert_eq!(p2.fee_tier, 5);
    assert_eq!(p3.fee_tier, 30);
}

#[test]
fn test_pool_lp_balance() {
    let env = Env::default();
    let contract_id = env.register(CounterContract, ());
    let client = CounterContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let provider = Address::generate(&env);

    let token_a = symbol_short!("TOKA");
    let token_b = symbol_short!("TOKB");

    let pool_id = client.register_pool(&admin, &token_a, &token_b, &1000, &1000, &30);
    let lp_tokens = client.pool_add_liquidity(&pool_id, &500, &500, &provider);

    let balance = client.get_pool_lp_balance(&pool_id, &provider);
    assert_eq!(balance, lp_tokens);
}

#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_invalid_fee_tier() {
    let env = Env::default();
    let contract_id = env.register(CounterContract, ());
    let client = CounterContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    let token_a = symbol_short!("TOKA");
    let token_b = symbol_short!("TOKB");

    client.register_pool(&admin, &token_a, &token_b, &1000, &1000, &100);
}

// ===== MULTI-POOL PER PAIR TESTS (Issue #1) =====

/// Register two XLM/USDC pools at different depths; `find_best_route`
/// should choose the pool that returns a higher output for the given
/// `amount_in`.
#[test]
fn test_two_pools_same_pair_route_chooses_optimal() {
    let env = Env::default();
    let contract_id = env.register(CounterContract, ());
    let client = CounterContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    let xlm = symbol_short!("XLM");
    let usdc = symbol_short!("USDC");

    // Pool A: shallow — 1 000 XLM / 1 000 USDC (fee tier 1 bps)
    let pool_a = client.register_pool(&admin, &xlm, &usdc, &1000, &1000, &1);
    // Pool B: deep   — 100 000 XLM / 100 000 USDC (fee tier 30 bps)
    // A deep pool produces more output per unit because price impact is lower.
    let pool_b = client.register_pool(&admin, &xlm, &usdc, &100000, &100000, &30);

    assert_ne!(pool_a, pool_b, "two distinct pool IDs must be assigned");

    let amount_in: i128 = 100;
    let route = client.find_best_route(&xlm, &usdc, &amount_in);
    assert!(route.is_some(), "a direct route must be found");

    let r = route.unwrap();
    assert_eq!(r.pools.len(), 1, "direct route uses exactly one pool");

    // The optimal pool for 100 XLM is the deep pool (pool_b) because the
    // shallow pool suffers a much larger price impact.
    let chosen_pool = r.pools.get(0).unwrap();
    assert_eq!(
        chosen_pool, pool_b,
        "find_best_route should pick the deep pool (pool_b) for lower price impact"
    );
}

/// Registering a second pool for the same pair must succeed (no duplicate rejection).
#[test]
fn test_register_second_pool_same_pair_succeeds() {
    let env = Env::default();
    let contract_id = env.register(CounterContract, ());
    let client = CounterContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    let tok_a = symbol_short!("TOKA");
    let tok_b = symbol_short!("TOKB");

    let id1 = client.register_pool(&admin, &tok_a, &tok_b, &500, &500, &1);
    let id2 = client.register_pool(&admin, &tok_a, &tok_b, &5000, &5000, &5);

    assert_ne!(id1, id2);
    assert!(client.get_pool(&id1).is_some());
    assert!(client.get_pool(&id2).is_some());
}
