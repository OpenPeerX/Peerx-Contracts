#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Events as _},
    Address, Env, Symbol,
};

use crate::errors::PeerXError;
use crate::observability::{get_log_level, log, set_log_level, LogLevel};
use crate::storage::ADMIN_KEY;

fn setup_with_admin() -> (Env, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    env.storage().persistent().set(&ADMIN_KEY, &admin);
    (env, admin)
}

fn emit_one_of_each_level(env: &Env) {
    log(env, LogLevel::Debug, (Symbol::new(env, "DebugEvt"),), 1u32);
    log(env, LogLevel::Info, (Symbol::new(env, "InfoEvt"),), 1u32);
    log(env, LogLevel::Warn, (Symbol::new(env, "WarnEvt"),), 1u32);
    log(env, LogLevel::Error, (Symbol::new(env, "ErrorEvt"),), 1u32);
}

#[test]
fn defaults_to_dev_level_when_unset() {
    // No mainnet/testnet feature compiled in during tests, so the compiled
    // default is the dev level (Debug) until an admin overrides it.
    let env = Env::default();
    assert_eq!(get_log_level(&env), LogLevel::Debug);
}

#[test]
fn set_log_level_rejects_non_admin() {
    let (env, _admin) = setup_with_admin();
    let not_admin = Address::generate(&env);

    let err = set_log_level(&env, not_admin, LogLevel::Warn).unwrap_err();

    assert_eq!(err, PeerXError::NotAdmin);
}

#[test]
fn set_log_level_persists_durably_for_admin() {
    let (env, admin) = setup_with_admin();

    set_log_level(&env, admin, LogLevel::Warn).unwrap();

    assert_eq!(get_log_level(&env), LogLevel::Warn);
}

#[test]
fn debug_level_allows_every_event_through() {
    let env = Env::default();
    let baseline = env.events().all().len();

    emit_one_of_each_level(&env);

    assert_eq!(env.events().all().len() - baseline, 4);
}

#[test]
fn error_level_silences_everything_except_error_events() {
    let (env, admin) = setup_with_admin();
    set_log_level(&env, admin, LogLevel::Error).unwrap();

    let baseline = env.events().all().len();
    emit_one_of_each_level(&env);

    // Only the single Error-level event should have been published; Debug,
    // Info, and Warn are silenced by the Error threshold.
    assert_eq!(env.events().all().len() - baseline, 1);
}

#[test]
fn warn_level_allows_warn_and_error_only() {
    let (env, admin) = setup_with_admin();
    set_log_level(&env, admin, LogLevel::Warn).unwrap();

    let baseline = env.events().all().len();
    emit_one_of_each_level(&env);

    assert_eq!(env.events().all().len() - baseline, 2);
}
