//! Centralized, admin-tunable event logging.
//!
//! `events.rs` used to publish every structured event unconditionally (or,
//! for a handful of ad-hoc call sites, behind a compile-time-only
//! `#[cfg(feature = "logging")]` flag). Neither approach lets a deployed
//! mainnet contract turn noise down without a redeploy. This module gives
//! every event a [`LogLevel`] and routes it through [`log`], which checks a
//! single admin-settable, durably-stored threshold before publishing.
use soroban_sdk::{contracttype, Address, Env, IntoVal, Symbol, Topics, Val};

use crate::admin;
use crate::errors::PeerXError;
use crate::storage::LOG_LEVEL_KEY;

/// Event severity, ordered from most to least verbose.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    /// Numeric severity used to compare levels; higher is more severe.
    fn rank(self) -> u32 {
        match self {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warn => 2,
            LogLevel::Error => 3,
        }
    }
}

// Per-network default, picked at compile time via the `mainnet` / `testnet`
// cargo features (see counter/Cargo.toml). Neither feature enabled means a
// dev build, which defaults to the most verbose level.
#[cfg(feature = "mainnet")]
fn default_log_level() -> LogLevel {
    LogLevel::Warn
}

#[cfg(all(feature = "testnet", not(feature = "mainnet")))]
fn default_log_level() -> LogLevel {
    LogLevel::Info
}

#[cfg(not(any(feature = "mainnet", feature = "testnet")))]
fn default_log_level() -> LogLevel {
    LogLevel::Debug
}

/// Returns the currently configured minimum log level. Falls back to the
/// compiled per-network default until an admin calls [`set_log_level`].
pub fn get_log_level(env: &Env) -> LogLevel {
    env.storage()
        .persistent()
        .get(&LOG_LEVEL_KEY)
        .unwrap_or_else(default_log_level)
}

/// Admin-only. Durably persists the minimum log level so it survives across
/// calls (and upgrades), overriding the compiled per-network default.
pub fn set_log_level(env: &Env, caller: Address, level: LogLevel) -> Result<(), PeerXError> {
    caller.require_auth();
    admin::require_admin(env, &caller)?;

    env.storage().persistent().set(&LOG_LEVEL_KEY, &level);

    env.events().publish(
        (Symbol::new(env, "LogLevelChanged"), caller),
        (level, env.ledger().timestamp()),
    );

    Ok(())
}

/// Central logging entry point. Every helper in `events.rs` routes its
/// publish call through here instead of calling `env.events().publish`
/// directly. `topics` and `data` are exactly what would otherwise be passed
/// to `env.events().publish`, so existing event names and payloads are
/// unchanged - only whether they are published at all depends on `level`
/// versus the configured threshold (from [`get_log_level`]).
pub fn log<T, D>(env: &Env, level: LogLevel, topics: T, data: D)
where
    T: Topics,
    D: IntoVal<Env, Val>,
{
    if level.rank() >= get_log_level(env).rank() {
        env.events().publish(topics, data);
    }
}
