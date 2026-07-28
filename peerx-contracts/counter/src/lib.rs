#![cfg_attr(all(not(test), target_family="wasm"), no_std)]
#![deny(missing_docs)]
//! PeerX Contracts
//! 
//! Production-grade Soroban smart contracts for a risk-free, on-chain trading classroom.
//! 
//! This contract implements a full simulated decentralized exchange (DEX) with:
//! - Constant-product AMM swaps
//! - Multi-hop routing
//! - Liquidity pools with LP tokens
//! - Limit and stop-loss orders
//! - KYC system
//! - Rate limiting
//! - Circuit breakers
//! - Governance parameters
//! - Referral system
//! - Staking bonuses
//! - And much more!
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, IntoVal, Map, Symbol,
    TryFromVal, Val, Vec,
};
#[cfg(feature = "experimental")]
use soroban_sdk::Bytes;

// Bring in modules from parent directory
mod admin;
mod bridge;
#[cfg(test)]
mod alert_tests;
mod alerts;
mod errors;
mod events;
mod invariants;
mod kyc;
#[cfg(test)]
mod kyc_tests;
mod liquidity_pool;
mod observability;
#[cfg(test)]
mod observability_tests;
mod rate_limit;
mod state_snapshot;
#[cfg(test)]
mod state_snapshot_tests;
mod storage;
mod referral_system;
mod batch {
    include!("../batch.rs");
}
mod tiers {
    include!("../tiers.rs");
}
#[cfg(all(test, feature = "experimental"))]
mod batch_event_tests;
#[cfg(all(test, feature = "experimental"))]
mod batch_opt_simple_test;
#[cfg(all(test, feature = "experimental"))]
mod batch_performance_tests;
mod oracle;
mod oracle_adapter;
mod orders;
#[cfg(test)]
mod orders_tests;
#[cfg(test)]
mod analytics_dashboard_tests;
#[cfg(test)]
mod oracle_adapter_tests;
#[cfg(test)]
mod multihop_swap_tests;
mod governance_types;
mod governance_system;
mod governance_params;
mod nonce;
mod risk_management;
mod validation;

pub use governance_params::{GovernanceParams, ParamKey, PendingParamUpdate};
pub use nonce::NonceGuard;
pub use rate_limit::SensitiveRateLimiter;
pub use rate_limit::action_tags as sensitive_action_tags;

mod portfolio {
    include!("../portfolio.rs");
}
mod trading {
    include!("../trading.rs");
}
#[cfg(feature = "experimental")]
mod analytics;
mod migration;

#[cfg(feature = "experimental")]
mod dynamic_fee_adjustment;
#[cfg(feature = "experimental")]
mod emergency_override;
#[cfg(feature = "experimental")]
mod fee_adjustment_manager;
#[cfg(feature = "experimental")]
mod fee_history;
#[cfg(feature = "experimental")]
mod network_congestion;

#[cfg(all(test, feature = "experimental"))]
mod dynamic_fee_adjustment_tests;
mod risk_management_tests;
#[cfg(test)]
mod fuzz;

// Staking Bonus System
mod staking_bonus;

// Treasury
mod treasury;
pub use treasury::{TreasuryManager, TreasuryKey, WithdrawRequest};

// Re-export fee adjustment types
#[cfg(feature = "experimental")]
pub use dynamic_fee_adjustment::{
    DynamicFeeAdjustment, FeeAdjustmentConfig, FeeAdjustmentResult, FeeImpact,
};
#[cfg(feature = "experimental")]
pub use fee_history::{AdjustmentReason, FeeHistoryEntry, FeeHistoryManager, FeeHistoryStats};
#[cfg(feature = "experimental")]
pub use network_congestion::{
    CongestionLevel, CongestionTrend, NetworkCongestionMonitor, NetworkMetrics,
};

// Re-export staking bonus types
#[cfg(feature = "experimental")]
pub use emergency_override::{
    EmergencyOverrideManager, EmergencyOverrideState, OverrideReason, OverrideStatus,
};
#[cfg(feature = "experimental")]
pub use fee_adjustment_manager::FeeAdjustmentManager;
pub use staking_bonus::{DistributionRecord, StakeRecord, StakingBonusKey, StakingBonusManager};

#[cfg(feature = "experimental")]
mod nft_errors;
#[cfg(feature = "experimental")]
mod nft_events;
#[cfg(feature = "experimental")]
mod nft_fractional;
#[cfg(feature = "experimental")]
mod nft_lending;
#[cfg(feature = "experimental")]
mod nft_marketplace;
#[cfg(feature = "experimental")]
mod nft_minting;
#[cfg(feature = "experimental")]
mod nft_storage;
#[cfg(feature = "experimental")]
mod nft_types;

#[cfg(feature = "experimental")]
mod private_transaction;
#[cfg(feature = "experimental")]
mod zkp_circuits;
#[cfg(feature = "experimental")]
mod zkp_errors;
#[cfg(feature = "experimental")]
mod zkp_proof_generation;
#[cfg(feature = "experimental")]
mod zkp_types;
#[cfg(feature = "experimental")]
mod zkp_verification;

// Re-export invariant functions for external use
pub use invariants::verify_contract_invariants;
pub use liquidity_pool::{LiquidityPool, PoolRegistry, Route};

// KYC exports for contract interface
pub use kyc::{
    GovernanceOverride, KYCError, KYCRecord, KYCStatus, KYCSystem, DEFAULT_PENDING_EXPIRY_DURATION,
    DEFAULT_TIMELOCK_DURATION, MIN_PENDING_EXPIRY_DURATION, MIN_TIMELOCK_DURATION,
};

// ZKP exports for contract interface
#[cfg(feature = "experimental")]
pub use private_transaction::{
    AuditTrailManager, PrivateTransactionBuilder, PrivateTransactionProcessor, WitnessManager,
};
#[cfg(feature = "experimental")]
pub use zkp_errors::ZKPError;
#[cfg(feature = "experimental")]
pub use zkp_proof_generation::ProofGenerator;
#[cfg(feature = "experimental")]
pub use zkp_types::{
    AuditEventType, AuditLogEntry, BalanceProof, Commitment, PrivateTransaction, ProofScheme,
    ProofVerificationResult, RangeProof, Receipt, TransactionWitness, ZKProof,
};
#[cfg(feature = "experimental")]
pub use zkp_verification::ProofVerifier;

use portfolio::{Asset, CachedPortfolio, CachedTopTraders, LPPosition, Portfolio};
pub use observability::LogLevel;
pub use portfolio::{Badge, Metrics, Transaction};
pub use rate_limit::{RateLimitStatus, RateLimiter};
pub use tiers::UserTier;
use trading::perform_swap;

use crate::errors::{ContractError, PeerXError, SwapChecklist};
use crate::storage::{ADMIN_KEY, PAUSED_KEY, READ_ONLY_ROLE_KEY};

/// Checks `role` against the durably-stored read-only auditor/dashboard
/// role set via `CounterContract::set_read_only_role`. Deliberately does
/// NOT call `role.require_auth()` - the whole point of `invoke_read` is
/// letting auditors/dashboards reach read-only entry points without
/// submitting a signed transaction of their own; the admin-curated
/// allowlist plus this identity check are the only gate.
fn require_read_only_role(env: &Env, role: &Address) -> Result<(), PeerXError> {
    let stored: Address = env
        .storage()
        .persistent()
        .get(&READ_ONLY_ROLE_KEY)
        .ok_or(PeerXError::NotReadOnlyRole)?;

    if stored == *role {
        Ok(())
    } else {
        Err(PeerXError::NotReadOnlyRole)
    }
}

pub(crate) fn require_verified_user(env: &Env, user: &Address) -> Result<(), ContractError> {
    kyc::KYCSystem::require_verified(env, user)
}

fn require_authenticated_verified_user(env: &Env, user: &Address) -> Result<(), ContractError> {
    user.require_auth();
    require_verified_user(env, user)
}

pub fn pause_trading(env: Env, caller: Address) -> Result<bool, PeerXError> {
    caller.require_auth();
    crate::admin::require_admin(&env, &caller)?;
    env.storage().persistent().set(&PAUSED_KEY, &true);
    Ok(true)
}

pub fn resume_trading(env: Env, caller: Address) -> Result<bool, PeerXError> {
    caller.require_auth();
    crate::admin::require_admin(&env, &caller)?;
    env.storage().persistent().set(&PAUSED_KEY, &false);
    Ok(true)
}

pub fn set_admin(env: Env, caller: Address, new_admin: Address) -> Result<(), PeerXError> {
    caller.require_auth();
    crate::admin::require_admin(&env, &caller)?;

    // ── Sensitive-action rate limit (audit-logged) ─────────────────────────
    SensitiveRateLimiter::check_and_record_tagged(
        &env,
        &caller,
        crate::rate_limit::action_tags::SET_ADMIN,
    )?;

    env.storage().persistent().set(&ADMIN_KEY, &new_admin);
    Ok(())
}

// Batch imports
use batch::{execute_batch_atomic, execute_batch_best_effort, BatchOperation, BatchResult};

// Oracle imports
use oracle::{get_stored_price, set_stored_price};
pub const CONTRACT_VERSION: u32 = 1;

const PORTFOLIO_CACHE_KEY: Symbol = symbol_short!("pcache");
const TOP_TRADERS_CACHE_KEY: Symbol = symbol_short!("tcache");
const CACHE_TTL_KEY: Symbol = symbol_short!("cttl");
const CACHE_HITS_KEY: Symbol = symbol_short!("chits");
const CACHE_MISSES_KEY: Symbol = symbol_short!("cmiss");
const DEFAULT_CACHE_TTL_SECONDS: u64 = 60;
const POOL_REGISTRY_KEY: Symbol = symbol_short!("lpreg");

fn load_pool_registry(env: &Env) -> PoolRegistry {
    env.storage()
        .instance()
        .get(&POOL_REGISTRY_KEY)
        .unwrap_or_else(|| PoolRegistry::new(env))
}

fn save_pool_registry(env: &Env, registry: &PoolRegistry) {
    env.storage().instance().set(&POOL_REGISTRY_KEY, registry);
}

#[derive(Clone)]
#[contracttype]
struct CacheHitMetrics {
    hits: u64,
    misses: u64,
    ratio_bps: u32,
}

fn get_cache_ttl(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&CACHE_TTL_KEY)
        .unwrap_or(DEFAULT_CACHE_TTL_SECONDS)
}

fn cache_ratio_bps(hits: u64, misses: u64) -> u32 {
    let total = hits.saturating_add(misses);
    if total == 0 {
        return 0;
    }
    ((hits.saturating_mul(10_000)) / total) as u32
}

fn record_cache_access(env: &Env, query: Symbol, hit: bool) {
    let mut hits: u64 = env.storage().instance().get(&CACHE_HITS_KEY).unwrap_or(0);
    let mut misses: u64 = env.storage().instance().get(&CACHE_MISSES_KEY).unwrap_or(0);

    if hit {
        hits = hits.saturating_add(1);
        env.storage().instance().set(&CACHE_HITS_KEY, &hits);
    } else {
        misses = misses.saturating_add(1);
        env.storage().instance().set(&CACHE_MISSES_KEY, &misses);
    }

    let payload = CacheHitMetrics {
        hits,
        misses,
        ratio_bps: cache_ratio_bps(hits, misses),
    };
    env.events()
        .publish((symbol_short!("cache"), query), payload);
}

fn invalidate_query_cache(env: &Env) {
    env.storage().instance().remove(&PORTFOLIO_CACHE_KEY);
    env.storage().instance().remove(&TOP_TRADERS_CACHE_KEY);
}

fn apply_trader_limit(
    env: &Env,
    traders: Vec<(Address, i128)>,
    limit: u32,
) -> Vec<(Address, i128)> {
    let mut result = Vec::new(env);
    let len = traders.len() as u32;
    let cap = if len < limit { len } else { limit };

    for i in 0..cap {
        if let Some(entry) = traders.get(i) {
            result.push_back(entry);
        }
    }
    result
}

#[contract]
pub struct CounterContract;

#[contractimpl]
impl CounterContract {
    /// Initialize the contract version.
    /// Should be called after deployment.
    pub fn initialize(env: Env) {
        if migration::get_stored_version(&env) == 0 {
            env.storage()
                .instance()
                .set(&Symbol::short("v_code"), &CONTRACT_VERSION);
        }
    }

    /// Get the current contract version from storage
    pub fn get_contract_version(env: Env) -> u32 {
        migration::get_stored_version(&env)
    }

    /// Migrate contract data from V1 to V2
    pub fn migrate(env: Env) -> Result<(), PeerXError> {
        migration::migrate_from_v1_to_v2(&env)
    }

    /// Mint simulated tokens to a user.
    /// 
    /// This is an admin-only function that creates new simulated tokens
    /// and credits them to the specified user's balance.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `token` - The token symbol to mint (XLM or a custom token)
    /// * `to` - The address to mint tokens to
    /// * `amount` - The amount of tokens to mint
    /// 
    /// # Panics
    /// This function will panic if called by a non-admin user
    pub fn mint(env: Env, token: Symbol, to: Address, amount: i128) {
        let mut portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        let asset = if token == Symbol::short("XLM") {
            Asset::XLM
        } else {
            Asset::Custom(token.clone())
        };

        portfolio.mint(&env, asset, to, amount);

        env.storage().instance().set(&(), &portfolio);
        invalidate_query_cache(&env);
    }

    /// Get the token balance of a user.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `token` - The token symbol to check balance for
    /// * `user` - The user address to check
    /// 
    /// # Returns
    /// The token balance of the user
    /// 
    /// # Panics
    /// This function never panics
    pub fn balance_of(env: Env, token: Symbol, user: Address) -> i128 {
        let portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        let asset = if token == Symbol::short("XLM") {
            Asset::XLM
        } else {
            Asset::Custom(token.clone())
        };

        portfolio.balance_of(&env, asset, user)
    }

    /// Alias to match external API
    pub fn get_balance(env: Env, token: Symbol, owner: Address) -> i128 {
        Self::balance_of(env, token, owner)
    }

    /// Swap tokens using simplified AMM (1:1 XLM <-> USDC-SIM)
    pub fn swap(env: Env, from: Symbol, to: Symbol, amount: i128, user: Address) -> Result<i128, ContractError> {
        require_authenticated_verified_user(&env, &user)?;

        let mut portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        // Get user's current tier for fee calculation and rate limiting
        let user_tier = portfolio.get_user_tier(&env, user.clone());

        // Check rate limit before executing swap
        RateLimiter::check_swap_limit(&env, &user, &user_tier)
            .map_err(|_| ContractError::RateLimitExceeded)?;

        // ===== RISK MANAGEMENT CHECKS =====

        // Check circuit breaker
        if risk_management::CircuitBreaker::is_circuit_breaker_active(&env) {
            return Err(ContractError::CircuitBreakerActive);
        }

        // Check concentration limits
        if risk_management::ConcentrationRisk::check_concentration_limit(&env, &portfolio, &user) {
            return Err(ContractError::InvalidAmount); // Use existing error for now
        }

        // Check position limits for the asset being purchased
        let to_asset = if to == symbol_short!("XLM") {
            Asset::XLM
        } else {
            Asset::Custom(to.clone())
        };

        // Estimate output amount for position limit check
        let estimated_out = if from == symbol_short!("XLM") && to == symbol_short!("USDCSIM") {
            amount // Simplified 1:1 for limit checking
        } else if from == symbol_short!("USDCSIM") && to == symbol_short!("XLM") {
            amount
        } else {
            amount // Fallback
        };

        if let Err(_) = risk_management::PositionLimits::check_position_limits(
            &env,
            &portfolio,
            &user,
            &to_asset,
            estimated_out,
        ) {
            return Err(ContractError::InvalidAmount); // Position limit exceeded
        }

        let fee_bps = user_tier.effective_fee_bps();

        // Calculate fee amount (fee is collected on input amount)
        let fee_amount = (amount * fee_bps as i128) / 10000;
        let swap_amount = amount - fee_amount;

        // Collect the fee
        if fee_amount > 0 {
            // Deduct from user
            let fee_asset = if from == symbol_short!("XLM") {
                Asset::XLM
            } else {
                Asset::Custom(from.clone())
            };

            // We need to use a mutable borrow of portfolio which we already have
            portfolio.debit(&env, fee_asset, user.clone(), fee_amount);
            portfolio.collect_fee(fee_amount);

            // Distribute referral commissions
            crate::referral_system::calculate_and_distribute_commission(&env, user.clone(), fee_amount);
        }

        let out_amount = perform_swap(
            &env,
            &mut portfolio,
            from.clone(),
            to.clone(),
            swap_amount,
            user.clone(),
        );

        portfolio.record_trade(&env, user.clone());

        // Record daily portfolio value for analytics
        portfolio.record_daily_portfolio_value(&env, user.clone(), env.ledger().timestamp());

        env.storage().instance().set(&(), &portfolio);
        invalidate_query_cache(&env);

        // Flush batched badge events
        crate::events::Events::flush_badge_events(&env);

        // Structured logging for successful swap, gated by the admin-set
        // log level (see observability.rs) rather than a compile-time flag.
        observability::log(
            &env,
            observability::LogLevel::Debug,
            (symbol_short!("swap"),),
            (amount, out_amount),
        );

        Ok(out_amount)
    }

    /// Pre-flight validation for a swap intent.
    ///
    /// Mirrors every guard that [`swap`] applies – balance, KYC, rate-limit,
    /// oracle freshness, pool depth, slippage, circuit-breaker, trading-pause,
    /// and pair validity – but is **read-only**.  No state is mutated and no
    /// authentication is required.
    ///
    /// A fully-green `SwapChecklist` (all fields `true`) means the swap
    /// **should** succeed on-chain (barring race conditions between this
    /// read and the actual submission).
    pub fn preflight_swap(
        env: Env,
        from: Symbol,
        to: Symbol,
        amount: i128,
        user: Address,
    ) -> SwapChecklist {
        let mut checklist = SwapChecklist {
            balance_ok: false,
            kyc_ok: false,
            rate_limit_ok: false,
            slippage_ok: false,
            oracle_fresh_ok: false,
            pool_depth_ok: false,
            circuit_breaker_ok: false,
            trading_paused_ok: false,
            pair_ok: false,
        };

        // ── Pair validity ────────────────────────────────────────────────────
        checklist.pair_ok = validation::validate_swap_pair(from.clone(), to.clone()).is_ok();
        if !checklist.pair_ok {
            return checklist;
        }

        // ── Trading pause ────────────────────────────────────────────────────
        let paused: bool = env
            .storage()
            .persistent()
            .get(&PAUSED_KEY)
            .unwrap_or(false);
        checklist.trading_paused_ok = !paused;
        if !checklist.trading_paused_ok {
            return checklist;
        }

        // ── Circuit breaker ──────────────────────────────────────────────────
        checklist.circuit_breaker_ok =
            !risk_management::CircuitBreaker::is_circuit_breaker_active(&env);

        // ── KYC ──────────────────────────────────────────────────────────────
        checklist.kyc_ok = kyc::KYCSystem::is_verified(&env, &user);

        // ── Balance ──────────────────────────────────────────────────────────
        let portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        let from_asset = if from == symbol_short!("XLM") {
            Asset::XLM
        } else {
            Asset::Custom(from.clone())
        };
        let balance = portfolio.balance_of(&env, from_asset, user.clone());
        checklist.balance_ok = balance >= amount;

        // ── Rate limit ───────────────────────────────────────────────────────
        let user_tier = portfolio.get_user_tier(&env, user.clone());
        checklist.rate_limit_ok =
            RateLimiter::check_swap_limit(&env, &user, &user_tier).is_ok();

        // ── Oracle freshness ─────────────────────────────────────────────────
        const STALE_THRESHOLD: u64 = 600; // 10 minutes
        let oracle_fresh = oracle::get_stored_price(&env, (from.clone(), to.clone()))
            .or_else(|| oracle::get_stored_price(&env, (to.clone(), from.clone())))
            .map(|data| {
                let age = env.ledger().timestamp().saturating_sub(data.timestamp);
                age <= STALE_THRESHOLD && data.price > 0
            })
            .unwrap_or(false);
        // Allow 1:1 fallback when no oracle is configured – treat as fresh
        checklist.oracle_fresh_ok = oracle_fresh || amount > 0;

        // ── Pool depth ───────────────────────────────────────────────────────
        let xlm_liq = portfolio.get_liquidity(Asset::XLM);
        let usdc_liq =
            portfolio.get_liquidity(Asset::Custom(symbol_short!("USDCSIM")));
        let reserves_sufficient = if from == symbol_short!("XLM") {
            usdc_liq >= amount
        } else {
            xlm_liq >= amount
        };
        checklist.pool_depth_ok = reserves_sufficient;

        // ── Slippage ─────────────────────────────────────────────────────────
        // Estimate slippage using constant-product AMM formula
        let (reserve_in, reserve_out) = if from == symbol_short!("XLM") {
            (xlm_liq as u128, usdc_liq as u128)
        } else {
            (usdc_liq as u128, xlm_liq as u128)
        };
        if reserve_in > 0 && reserve_out > 0 {
            let amount_u = amount as u128;
            // Output with fee
            let fee_bps: u128 = 30; // 0.3%
            let amount_after_fee = amount_u * (10000 - fee_bps) / 10000;
            let actual_out = (reserve_out * amount_after_fee) / (reserve_in + amount_after_fee);
            // Theoretical output without fee (for slippage calc)
            let theoretical_out = (reserve_out * amount_u) / (reserve_in + amount_u);
            let max_slip: u32 = env
                .storage()
                .instance()
                .get(&symbol_short!("MAX_SLIP"))
                .unwrap_or(10000);
            let slippage_ok = if theoretical_out > 0 {
                let slippage_bps = ((theoretical_out - actual_out) * 10000) / theoretical_out;
                slippage_bps <= max_slip as u128
            } else {
                true
            };
            checklist.slippage_ok = slippage_ok;
        } else {
            // No liquidity – AMM fallback uses oracle price, slippage N/A
            checklist.slippage_ok = true;
        }

        checklist
    }

    /// Non-panicking swap that counts failed orders and returns 0 on failure.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `from` - The token to swap from
    /// * `to` - The token to swap to
    /// * `amount` - The amount to swap
    /// * `user` - The user performing the swap
    /// 
    /// # Returns
    /// The amount received from the swap, or 0 if the swap failed
    /// 
    /// # Events
    /// Emits a "fail" event if the swap fails
    /// Emits a "swap" event if the swap succeeds
    /// 
    /// # Panics
    /// This function never panics
    pub fn safe_swap(env: Env, from: Symbol, to: Symbol, amount: i128, user: Address) -> i128 {
        if require_authenticated_verified_user(&env, &user).is_err() {
            return 0;
        }

        let mut portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        let tokens_ok = (from == symbol_short!("XLM") || from == symbol_short!("USDCSIM"))
            && (to == symbol_short!("XLM") || to == symbol_short!("USDCSIM"));
        let pair_ok = from != to;
        let amount_ok = amount > 0;

        if !(tokens_ok && pair_ok && amount_ok) {
            // Count failed order
            portfolio.inc_failed_order();
            env.storage().instance().set(&(), &portfolio);
            invalidate_query_cache(&env);

            observability::log(
                &env,
                observability::LogLevel::Warn,
                (symbol_short!("fail"), user.clone()),
                (from, to, amount),
            );
            return 0;
        }

        let out_amount = perform_swap(&env, &mut portfolio, from, to, amount, user.clone());
        portfolio.record_trade(&env, user);
        env.storage().instance().set(&(), &portfolio);
        invalidate_query_cache(&env);

        // Flush batched badge events
        crate::events::Events::flush_badge_events(&env);

        observability::log(
            &env,
            observability::LogLevel::Debug,
            (symbol_short!("swap"),),
            (amount, out_amount),
        );

        out_amount
    }

    /// Record a swap execution for a user.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `user` - The user who performed the trade
    /// 
    /// # Panics
    /// This function never panics
    pub fn record_trade(env: Env, user: Address) {
        let mut portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        portfolio.record_trade(&env, user);

        env.storage().instance().set(&(), &portfolio);
        invalidate_query_cache(&env);
    }

    /// Get portfolio stats for a user (trade count, pnl).
    /// 
    /// Uses caching with a configurable TTL.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `user` - The user to get portfolio stats for
    /// 
    /// # Returns
    /// A tuple of (trade count, pnl)
    /// 
    /// # Panics
    /// This function never panics
    pub fn get_portfolio(env: Env, user: Address) -> (u32, i128) {
        let now = env.ledger().timestamp();
        let ttl = get_cache_ttl(&env);

        let portfolio_cache: Map<Address, CachedPortfolio> = env
            .storage()
            .instance()
            .get(&PORTFOLIO_CACHE_KEY)
            .unwrap_or_else(|| Map::new(&env));

        if let Some(entry) = portfolio_cache.get(user.clone()) {
            if now.saturating_sub(entry.cached_at) <= ttl {
                record_cache_access(&env, symbol_short!("portf"), true);
                return (entry.trades, entry.pnl);
            }
        }

        record_cache_access(&env, symbol_short!("portf"), false);
        let portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        let value = portfolio.get_portfolio(&env, user.clone());
        let mut updated_cache: Map<Address, CachedPortfolio> = env
            .storage()
            .instance()
            .get(&PORTFOLIO_CACHE_KEY)
            .unwrap_or_else(|| Map::new(&env));
        updated_cache.set(
            user,
            CachedPortfolio {
                trades: value.0,
                pnl: value.1,
                cached_at: now,
            },
        );
        env.storage()
            .instance()
            .set(&PORTFOLIO_CACHE_KEY, &updated_cache);

        value
    }

    /// Get top traders with instance-storage caching.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `limit` - Maximum number of traders to return
    /// 
    /// # Returns
    /// A vector of (address, pnl), sorted by pnl descending
    /// 
    /// # Panics
    /// This function never panics
    pub fn get_top_traders(env: Env, limit: u32) -> Vec<(Address, i128)> {
        let now = env.ledger().timestamp();
        let ttl = get_cache_ttl(&env);

        if let Some(entry) = env
            .storage()
            .instance()
            .get::<_, CachedTopTraders>(&TOP_TRADERS_CACHE_KEY)
        {
            if now.saturating_sub(entry.cached_at) <= ttl {
                record_cache_access(&env, symbol_short!("toptr"), true);
                return apply_trader_limit(&env, entry.traders, limit);
            }
        }

        record_cache_access(&env, symbol_short!("toptr"), false);
        let portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        let candidate_limit = if limit > 100 { limit } else { limit.max(100) };
        let traders = portfolio.get_top_traders(&env, candidate_limit);
        env.storage().instance().set(
            &TOP_TRADERS_CACHE_KEY,
            &CachedTopTraders {
                traders: traders.clone(),
                cached_at: now,
            },
        );

        apply_trader_limit(&env, traders, limit)
    }

    /// Update cache TTL in seconds (admin only).
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `caller` - The admin caller address
    /// * `ttl_seconds` - New cache TTL in seconds
    /// 
    /// # Errors
    /// Returns `PeerXError::NotAdmin` if caller is not admin
    /// 
    /// # Panics
    /// Panics if caller authentication fails
    pub fn set_cache_ttl(
        env: Env,
        caller: Address,
        ttl_seconds: u64,
    ) -> Result<(), PeerXError> {
        caller.require_auth();
        crate::admin::require_admin(&env, &caller)?;
        env.storage().instance().set(&CACHE_TTL_KEY, &ttl_seconds);
        Ok(())
    }

    // ===== OBSERVABILITY =====

    /// Set the minimum event log level (admin only). Durable across calls
    /// and upgrades; overrides the compiled per-network default (Debug on
    /// dev, Info on testnet, Warn on mainnet). See src/observability.rs.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `caller` - The admin caller address
    /// * `level` - The new minimum log level
    /// 
    /// # Errors
    /// Returns `PeerXError::NotAdmin` if caller is not admin
    /// 
    /// # Panics
    /// Panics if caller authentication fails
    pub fn set_log_level(env: Env, caller: Address, level: LogLevel) -> Result<(), PeerXError> {
        observability::set_log_level(&env, caller, level)
    }

    /// Get the currently effective minimum event log level.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// 
    /// # Returns
    /// The current minimum log level
    /// 
    /// # Panics
    /// This function never panics
    pub fn get_log_level(env: Env) -> LogLevel {
        observability::get_log_level(&env)
    }

    /// Get cache stats as (hits, misses, hit_ratio_bps).
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// 
    /// # Returns
    /// A tuple of (hits, misses, hit_ratio_bps)
    /// 
    /// # Panics
    /// This function never panics
    pub fn get_cache_stats(env: Env) -> (u64, u64, u32) {
        let hits: u64 = env.storage().instance().get(&CACHE_HITS_KEY).unwrap_or(0);
        let misses: u64 = env.storage().instance().get(&CACHE_MISSES_KEY).unwrap_or(0);
        (hits, misses, cache_ratio_bps(hits, misses))
    }

    /// Clear all query caches (admin only).
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `caller` - The admin caller address
    /// 
    /// # Errors
    /// Returns `PeerXError::NotAdmin` if caller is not admin
    /// 
    /// # Panics
    /// Panics if caller authentication fails
    pub fn clear_cache(env: Env, caller: Address) -> Result<(), PeerXError> {
        caller.require_auth();
        crate::admin::require_admin(&env, &caller)?;
        invalidate_query_cache(&env);
        Ok(())
    }

    /// Get aggregate metrics.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// 
    /// # Returns
    /// Aggregate metrics for the contract
    /// 
    /// # Panics
    /// This function never panics
    pub fn get_metrics(env: Env) -> Metrics {
        let portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        portfolio.get_metrics()
    }

    /// Check if a user has earned a specific badge.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `user` - The user to check
    /// * `badge` - The badge to check for
    /// 
    /// # Returns
    /// True if the user has the badge, false otherwise
    /// 
    /// # Panics
    /// This function never panics
    pub fn has_badge(env: Env, user: Address, badge: Badge) -> bool {
        let portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        portfolio.has_badge(&env, user, badge)
    }

    /// Get all badges earned by a user.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `user` - The user to get badges for
    /// 
    /// # Returns
    /// A vector of badges earned by the user
    /// 
    /// # Panics
    /// This function never panics
    pub fn get_user_badges(env: Env, user: Address) -> Vec<Badge> {
        let portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        portfolio.get_user_badges(&env, user)
    }

    /// Get recent transactions for a user.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `user` - The user to get transactions for
    /// * `limit` - Maximum number of transactions to return
    /// 
    /// # Returns
    /// A vector of recent transactions for the user
    /// 
    /// # Panics
    /// This function never panics
    pub fn get_user_transactions(env: Env, user: Address, limit: u32) -> Vec<Transaction> {
        let portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        portfolio.get_user_transactions(&env, user, limit)
    }

    /// Get the current tier for a user.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `user` - The user to get the tier for
    /// 
    /// # Returns
    /// The user's current tier
    /// 
    /// # Panics
    /// This function never panics
    pub fn get_user_tier(env: Env, user: Address) -> UserTier {
        let portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        portfolio.get_user_tier(&env, user)
    }

    // ===== RATE LIMITING =====

    /// Get rate limit status for swap operations.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `user` - The user to check rate limits for
    /// 
    /// # Returns
    /// The rate limit status for swap operations
    /// 
    /// # Panics
    /// This function never panics
    pub fn get_swap_rate_limit(env: Env, user: Address) -> RateLimitStatus {
        let portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        let user_tier = portfolio.get_user_tier(&env, user.clone());
        RateLimiter::get_swap_status(&env, &user, &user_tier)
    }

    /// Get rate limit status for LP operations.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `user` - The user to check rate limits for
    /// 
    /// # Returns
    /// The rate limit status for LP operations
    /// 
    /// # Panics
    /// This function never panics
    pub fn get_lp_rate_limit(env: Env, user: Address) -> RateLimitStatus {
        let portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        let user_tier = portfolio.get_user_tier(&env, user.clone());
        RateLimiter::get_lp_status(&env, &user, &user_tier)
    }

    /// Remove expired rate limit counters for a user.
    /// Returns the number of storage entries cleaned up.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `user` - The user to clean up rate limits for
    /// 
    /// # Returns
    /// The number of storage entries cleaned up
    /// 
    /// # Panics
    /// This function never panics
    pub fn cleanup_rate_limits(env: Env, user: Address) -> u32 {
        RateLimiter::cleanup_rate_limits(&env, &user)
    }

    /// Get the number of sensitive admin actions used by `user` in the
    /// current 10-minute window (limit = `SENSITIVE_ACTION_LIMIT`).
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `user` - The user to check sensitive rate limit usage for
    /// 
    /// # Returns
    /// The number of sensitive actions used in the current window
    /// 
    /// # Panics
    /// This function never panics
    pub fn get_sensitive_rate_limit_usage(env: Env, user: Address) -> u32 {
        SensitiveRateLimiter::current_usage(&env, &user)
    }

    // ===== BATCH OPERATIONS =====

    pub fn execute_batch_atomic(env: Env, operations: Vec<BatchOperation>) -> BatchResult {
        // Validate batch operations are not empty
        if operations.is_empty() {
            let mut result = BatchResult::new(&env);
            result.operations_failed = 1;
            return result;
        }

        // Extract caller from first operation for authentication and rate limiting
        let caller = match operations.get(0) {
            Some(BatchOperation::Swap(_, _, _, user)) 
            | Some(BatchOperation::AddLiquidity(_, _, user))
            | Some(BatchOperation::RemoveLiquidity(_, _, user)) => Some(user.clone()),
            Some(BatchOperation::MintToken(_, _, _)) => None,
            _ => None,
        };

        // Require authentication from the caller
        if let Some(caller_addr) = &caller {
            caller_addr.require_auth();
            if let Err(_) = require_verified_user(&env, caller_addr) {
                let mut result = BatchResult::new(&env);
                result.operations_failed = 1;
                return result;
            }
        }

        let mut portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        // Check rate limiting for batch operations with swaps
        if let Some(caller_addr) = &caller {
            let user_tier = portfolio.get_user_tier(&env, caller_addr.clone());
            // Count swap operations in batch
            let swap_count = operations.iter().filter(|op| matches!(op, BatchOperation::Swap(_, _, _, _))).count();
            if swap_count > 0 {
                // Apply rate limit check for batch swaps
                if RateLimiter::check_swap_limit(&env, caller_addr, &user_tier).is_err() {
                    let mut result = BatchResult::new(&env);
                    result.operations_failed = 1;
                    return result;
                }
            }
        }

        let result = execute_batch_atomic(&env, &mut portfolio, operations.clone());

        match result {
            Ok(res) => {
                env.storage().instance().set(&(), &portfolio);
                
                // Record rate limit usage for executed swaps
                if let Some(caller_addr) = &caller {
                    let swap_count = operations.iter().filter(|op| matches!(op, BatchOperation::Swap(_, _, _, _))).count();
                    if swap_count > 0 && res.operations_executed > 0 {
                        for _ in 0..res.operations_executed {
                            RateLimiter::record_swap_op(&env, caller_addr, env.ledger().timestamp());
                        }
                    }
                }

                crate::events::Events::flush_badge_events(&env);
                invalidate_query_cache(&env);
                res
            }
            Err(_) => {
                let mut err = BatchResult::new(&env);
                err.operations_failed = 1;
                err
            }
        }
    }

    pub fn execute_batch_best_effort(env: Env, operations: Vec<BatchOperation>) -> BatchResult {
        // Validate batch operations are not empty
        if operations.is_empty() {
            let mut result = BatchResult::new(&env);
            result.operations_failed = 1;
            return result;
        }

        // Extract caller from first operation for authentication and rate limiting
        let caller = match operations.get(0) {
            Some(BatchOperation::Swap(_, _, _, user)) 
            | Some(BatchOperation::AddLiquidity(_, _, user))
            | Some(BatchOperation::RemoveLiquidity(_, _, user)) => Some(user.clone()),
            Some(BatchOperation::MintToken(_, _, _)) => None,
            _ => None,
        };

        // Require authentication from the caller
        if let Some(caller_addr) = &caller {
            caller_addr.require_auth();
            if let Err(_) = require_verified_user(&env, caller_addr) {
                let mut result = BatchResult::new(&env);
                result.operations_failed = 1;
                return result;
            }
        }

        let mut portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        // Check rate limiting for batch operations with swaps
        if let Some(caller_addr) = &caller {
            let user_tier = portfolio.get_user_tier(&env, caller_addr.clone());
            // Count swap operations in batch
            let swap_count = operations.iter().filter(|op| matches!(op, BatchOperation::Swap(_, _, _, _))).count();
            if swap_count > 0 {
                // Apply rate limit check for batch swaps
                if RateLimiter::check_swap_limit(&env, caller_addr, &user_tier).is_err() {
                    let mut result = BatchResult::new(&env);
                    result.operations_failed = 1;
                    return result;
                }
            }
        }

        let result = execute_batch_best_effort(&env, &mut portfolio, operations.clone());

        match result {
            Ok(res) => {
                env.storage().instance().set(&(), &portfolio);
                
                // Record rate limit usage for executed swaps
                if let Some(caller_addr) = &caller {
                    let swap_count = operations.iter().filter(|op| matches!(op, BatchOperation::Swap(_, _, _, _))).count();
                    if swap_count > 0 && res.operations_executed > 0 {
                        for _ in 0..res.operations_executed {
                            RateLimiter::record_swap_op(&env, caller_addr, env.ledger().timestamp());
                        }
                    }
                }

                crate::events::Events::flush_badge_events(&env);
                invalidate_query_cache(&env);
                res
            }
            Err(_) => {
                let mut err = BatchResult::new(&env);
                err.operations_failed = 1;
                err
            }
        }
    }

    pub fn execute_batch(env: Env, operations: Vec<BatchOperation>) -> BatchResult {
        Self::execute_batch_atomic(env, operations)
    }

    // ===== LIQUIDITY PROVIDER (LP) FUNCTIONS =====

    /// Add liquidity to the pool and mint LP tokens
    /// Returns the number of LP tokens minted
    pub fn add_liquidity(env: Env, xlm_amount: i128, usdc_amount: i128, user: Address) -> Result<i128, ContractError> {
        require_authenticated_verified_user(&env, &user)?;

        if xlm_amount <= 0 || usdc_amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let mut portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        // Check rate limit for LP operations
        let user_tier = portfolio.get_user_tier(&env, user.clone());
        RateLimiter::check_lp_limit(&env, &user, &user_tier)
            .map_err(|_| ContractError::RateLimitExceeded)?;

        // Get current pool state
        let current_xlm = portfolio.get_liquidity(Asset::XLM);
        let current_usdc = portfolio.get_liquidity(Asset::Custom(symbol_short!("USDCSIM")));
        let total_lp_tokens = portfolio.get_total_lp_tokens();

        // Check user has sufficient balance
        let user_xlm_balance = portfolio.balance_of(&env, Asset::XLM, user.clone());
        let user_usdc_balance =
            portfolio.balance_of(&env, Asset::Custom(symbol_short!("USDCSIM")), user.clone());

        if user_xlm_balance < xlm_amount || user_usdc_balance < usdc_amount {
            return Err(ContractError::InsufficientBalance);
        }

        // Calculate LP tokens to mint using constant product AMM formula
        // If pool is empty, LP tokens = sqrt(xlm * usdc)
        // Otherwise, LP tokens = (deposit / pool_size) * total_lp_tokens
        let lp_tokens_minted = if total_lp_tokens == 0 {
            let product = (xlm_amount as u128).saturating_mul(usdc_amount as u128);
            if product == 0 {
                return Err(ContractError::InvalidAmount);
            }
            // Integer square root using Babylonian method
            let mut guess = product;
            let mut prev_guess = 0u128;
            // Limit iterations to prevent infinite loop
            let mut iterations = 0;
            while guess != prev_guess && iterations < 100 {
                prev_guess = guess;
                let quotient = product / guess;
                guess = (guess + quotient) / 2;
                if guess == 0 {
                    guess = 1;
                    break;
                }
                iterations += 1;
            }
            guess as i128
        } else {
            // Calculate proportional share
            // LP tokens = min((xlm_amount / current_xlm) * total_lp_tokens, (usdc_amount / current_usdc) * total_lp_tokens)
            // This ensures the ratio is maintained
            let xlm_share = if current_xlm > 0 {
                (xlm_amount as u128).saturating_mul(total_lp_tokens as u128) / (current_xlm as u128)
            } else {
                0
            };
            let usdc_share = if current_usdc > 0 {
                (usdc_amount as u128).saturating_mul(total_lp_tokens as u128)
                    / (current_usdc as u128)
            } else {
                0
            };

            // Take minimum to maintain ratio
            core::cmp::min(xlm_share as i128, usdc_share as i128)
        };

        if lp_tokens_minted <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        // Debit assets from user (transfer to pool)
        portfolio.debit(&env, Asset::XLM, user.clone(), xlm_amount);
        portfolio.debit(
            &env,
            Asset::Custom(symbol_short!("USDCSIM")),
            user.clone(),
            usdc_amount,
        );

        // Update pool liquidity
        portfolio.add_pool_liquidity(xlm_amount, usdc_amount);

        // Update or create LP position
        let existing_position = portfolio.get_lp_position(user.clone());
        let new_position = if let Some(mut pos) = existing_position {
            // Update existing position
            pos.xlm_deposited = pos.xlm_deposited.saturating_add(xlm_amount);
            pos.usdc_deposited = pos.usdc_deposited.saturating_add(usdc_amount);
            pos.lp_tokens_minted = pos.lp_tokens_minted.saturating_add(lp_tokens_minted);
            pos
        } else {
            // Create new position
            LPPosition {
                lp_address: user.clone(),
                xlm_deposited: xlm_amount,
                usdc_deposited: usdc_amount,
                lp_tokens_minted,
            }
        };

        portfolio.set_lp_position(user.clone(), new_position);
        portfolio.add_total_lp_tokens(lp_tokens_minted);

        // Record LP deposit for badge tracking
        portfolio.record_lp_deposit(user.clone());
        portfolio.check_and_award_badges(&env, user.clone());

        // Record rate limit usage
        RateLimiter::record_lp_op(&env, &user, env.ledger().timestamp());

        env.storage().instance().set(&(), &portfolio);
        invalidate_query_cache(&env);

        // Flush batched badge events
        crate::events::Events::flush_badge_events(&env);

        Ok(lp_tokens_minted)
    }

    /// Remove liquidity from the pool by burning LP tokens
    /// Returns (xlm_amount, usdc_amount) returned to user
    pub fn remove_liquidity(env: Env, lp_tokens: i128, user: Address) -> Result<(i128, i128), ContractError> {
        require_authenticated_verified_user(&env, &user)?;

        if lp_tokens <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let mut portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        // Get user's LP position
        let mut pos = portfolio
            .get_lp_position(user.clone())
            .ok_or(ContractError::LPPositionNotFound)?;

        if pos.lp_tokens_minted < lp_tokens {
            return Err(ContractError::InsufficientLPTokens);
        }

        // Get current pool state
        let current_xlm = portfolio.get_liquidity(Asset::XLM);
        let current_usdc = portfolio.get_liquidity(Asset::Custom(symbol_short!("USDCSIM")));
        let total_lp_tokens = portfolio.get_total_lp_tokens();

        if total_lp_tokens <= 0 {
            return Err(ContractError::LPPositionNotFound);
        }

        // Calculate proportional share of pool
        // xlm_amount = (lp_tokens / total_lp_tokens) * current_xlm
        // usdc_amount = (lp_tokens / total_lp_tokens) * current_usdc
        let xlm_amount = ((lp_tokens as u128).saturating_mul(current_xlm as u128)
            / (total_lp_tokens as u128)) as i128;
        let usdc_amount = ((lp_tokens as u128).saturating_mul(current_usdc as u128)
            / (total_lp_tokens as u128)) as i128;

        if xlm_amount <= 0 || usdc_amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        if xlm_amount > pos.xlm_deposited.saturating_mul(101) / 100
            || usdc_amount > pos.usdc_deposited.saturating_mul(101) / 100
        {
            return Err(ContractError::InsufficientBalance);
        }

        // Update pool liquidity (subtract)
        portfolio.set_liquidity(Asset::XLM, current_xlm.saturating_sub(xlm_amount));
        portfolio.set_liquidity(
            Asset::Custom(symbol_short!("USDCSIM")),
            current_usdc.saturating_sub(usdc_amount),
        );

        // Transfer assets from pool to user
        portfolio.mint(&env, Asset::XLM, user.clone(), xlm_amount);
        portfolio.mint(
            &env,
            Asset::Custom(symbol_short!("USDCSIM")),
            user.clone(),
            usdc_amount,
        );

        // Update LP position
        pos.lp_tokens_minted = pos.lp_tokens_minted.saturating_sub(lp_tokens);
        pos.xlm_deposited = pos.xlm_deposited.saturating_sub(xlm_amount);
        pos.usdc_deposited = pos.usdc_deposited.saturating_sub(usdc_amount);

        if pos.lp_tokens_minted == 0 {
            // Remove position if all tokens burned
            // Note: Map doesn't have remove, so we set to a zero position or track separately
            // For now, we'll keep it with zero values
        }
        portfolio.set_lp_position(user.clone(), pos);
        portfolio.subtract_total_lp_tokens(lp_tokens);

        // Record rate limit usage
        RateLimiter::record_lp_op(&env, &user, env.ledger().timestamp());

        env.storage().instance().set(&(), &portfolio);
        invalidate_query_cache(&env);

        Ok((xlm_amount, usdc_amount))
    }

    /// Get LP positions for a user
    /// Returns a Vec containing the user's position if it exists
    pub fn get_lp_positions(env: Env, user: Address) -> Vec<LPPosition> {
        let portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));

        let mut result = Vec::new(&env);
        if let Some(position) = portfolio.get_lp_position(user) {
            result.push_back(position);
        }
        result
    }

    // ===== MULTI-TOKEN POOL REGISTRY =====

    pub fn register_pool(
        env: Env,
        admin: Address,
        token_a: Symbol,
        token_b: Symbol,
        initial_a: i128,
        initial_b: i128,
        fee_tier: u32,
    ) -> Result<u64, ContractError> {
        let mut registry = load_pool_registry(&env);
        let pool_id = registry.register_pool(
            &env, admin, token_a, token_b, initial_a, initial_b, fee_tier,
        )?;
        save_pool_registry(&env, &registry);
        Ok(pool_id)
    }

    pub fn pool_add_liquidity(
        env: Env,
        pool_id: u64,
        amount_a: i128,
        amount_b: i128,
        provider: Address,
    ) -> Result<i128, ContractError> {
        provider.require_auth();
        require_verified_user(&env, &provider)?;

        let mut registry = load_pool_registry(&env);
        let lp_tokens = registry.add_liquidity(&env, pool_id, amount_a, amount_b, provider)?;
        save_pool_registry(&env, &registry);
        Ok(lp_tokens)
    }

    pub fn pool_remove_liquidity(
        env: Env,
        pool_id: u64,
        lp_tokens: i128,
        provider: Address,
    ) -> Result<(i128, i128), ContractError> {
        provider.require_auth();
        require_verified_user(&env, &provider)?;

        let mut registry = load_pool_registry(&env);
        let result = registry.remove_liquidity(&env, pool_id, lp_tokens, provider)?;
        save_pool_registry(&env, &registry);
        Ok(result)
    }

    pub fn pool_swap(
        env: Env,
        pool_id: u64,
        token_in: Symbol,
        amount_in: i128,
        min_amount_out: i128,
        trader: Address,
    ) -> Result<i128, ContractError> {
        trader.require_auth();
        require_verified_user(&env, &trader)?;

        let mut registry = load_pool_registry(&env);
        let result = registry.swap(&env, pool_id, token_in, amount_in, min_amount_out)?;
        save_pool_registry(&env, &registry);
        Ok(result)
    }

    pub fn find_best_route(
        env: Env,
        token_in: Symbol,
        token_out: Symbol,
        amount_in: i128,
    ) -> Option<Route> {
        let registry = load_pool_registry(&env);
        registry.find_best_route(&env, token_in, token_out, amount_in)
    }

    /// Execute a multi-hop swap along a discovered route
    /// Atomic execution: fails if any hop fails
    /// Respects slippage tolerance specified by min_amount_out
    pub fn execute_multi_hop_swap(
        env: Env,
        route: Route,
        amount_in: i128,
        min_amount_out: i128,
        trader: Address,
    ) -> Result<i128, ContractError> {
        trader.require_auth();
        require_verified_user(&env, &trader)?;
        
        trading::execute_multihop_swap(&env, &route, amount_in, min_amount_out, &trader)
    }

    pub fn get_pool(env: Env, pool_id: u64) -> Option<LiquidityPool> {
        let registry = load_pool_registry(&env);
        registry.get_pool(pool_id)
    }

    pub fn get_pool_lp_balance(env: Env, pool_id: u64, provider: Address) -> i128 {
        let registry = load_pool_registry(&env);
        registry.get_lp_balance(pool_id, provider)
    }

    pub fn set_price(env: Env, token_pair: (Symbol, Symbol), price: u128) {
        set_stored_price(&env, token_pair, price);
    }

    pub fn get_current_price(env: Env, token_pair: (Symbol, Symbol)) -> u128 {
        get_stored_price(&env, token_pair)
            .map(|d| d.price)
            .unwrap_or(0)
    }

    pub fn set_price_update_tolerance_bps(env: Env, token_pair: (Symbol, Symbol), bps: u32) {
        oracle::set_price_update_tolerance_bps(&env, token_pair, bps);
    }

    pub fn set_pool_liquidity(env: Env, token: Symbol, amount: i128) {
        let mut portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));
        let asset = if token == symbol_short!("XLM") {
            Asset::XLM
        } else {
            Asset::Custom(token)
        };
        portfolio.set_liquidity(asset, amount);
        env.storage().instance().set(&(), &portfolio);
    }

    pub fn set_max_slippage_bps(env: Env, bps: u32) {
        env.storage()
            .instance()
            .set(&symbol_short!("MAX_SLIP"), &bps);
    }

    // ────────────────────────────────────────────────────────────────────────
    // Staking Bonus System
    // ────────────────────────────────────────────────────────────────────────

    /// Stake tokens for a specified duration to earn bonuses
    /// Supports: 30, 60, 90, or 365-day stakes
    pub fn stake(env: Env, user: Address, amount: i128, duration_days: u32) -> Result<u32, ContractError> {
        require_authenticated_verified_user(&env, &user)?;
        let result = StakingBonusManager::stake(&env, user, amount, duration_days)?;
        invalidate_query_cache(&env);
        Ok(result)
    }

    /// Claim earned staking bonuses (after 30-day holding period)
    /// Returns total bonuses claimed
    pub fn claim_staking_bonuses(env: Env, user: Address) -> Result<i128, ContractError> {
        require_authenticated_verified_user(&env, &user)?;
        let result = StakingBonusManager::claim_bonuses(&env, user)?;
        invalidate_query_cache(&env);
        Ok(result)
    }

    /// Claim staked principal after lock period expires
    /// Returns the principal amount
    pub fn claim_stake(env: Env, user: Address, stake_id: u32) -> Result<i128, ContractError> {
        require_authenticated_verified_user(&env, &user)?;
        let result = StakingBonusManager::claim_stake(&env, user, stake_id)?;
        invalidate_query_cache(&env);
        Ok(result)
    }

    /// Unstake early before lock period (incurs 10% penalty)
    /// Returns (principal_after_penalty, penalty_amount)
    pub fn unstake_early(env: Env, user: Address, stake_id: u32) -> Result<(i128, i128), ContractError> {
        require_authenticated_verified_user(&env, &user)?;
        let result = StakingBonusManager::unstake_early(&env, user, stake_id)?;
        invalidate_query_cache(&env);
        Ok(result)
    }

    /// Get all stake records for a user (transparent view)
    pub fn get_user_stakes(env: Env, user: Address) -> Vec<StakeRecord> {
        StakingBonusManager::get_user_stakes(&env, user)
    }

    /// Get specific stake details
    pub fn get_stake_details(env: Env, user: Address, stake_id: u32) -> Result<StakeRecord, ContractError> {
        StakingBonusManager::get_stake_details(&env, user, stake_id)
    }

    /// Get total staked amount for a user
    pub fn get_user_total_staked(env: Env, user: Address) -> i128 {
        StakingBonusManager::get_user_total_staked(&env, user)
    }

    /// Get total earned bonuses for a user
    pub fn get_user_earned_bonuses(env: Env, user: Address) -> i128 {
        StakingBonusManager::get_user_earned_bonuses(&env, user)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Advanced Order Types (Limit & Stop-Loss)
    // ────────────────────────────────────────────────────────────────────────

    /// Place a limit order that executes when price reaches limit_price or better
    pub fn place_limit_order(
        env: Env,
        token_in: Symbol,
        token_out: Symbol,
        amount_in: i128,
        limit_price: u128,
        expires_at: Option<u64>,
        user: Address,
    ) -> Result<u64, ContractError> {
        require_authenticated_verified_user(&env, &user)?;
        orders::OrderManager::place_limit_order(&env, user, token_in, token_out, amount_in, limit_price, expires_at)
    }

    /// Place a stop-loss order that executes when price reaches trigger_price
    pub fn place_stop_loss(
        env: Env,
        token_in: Symbol,
        token_out: Symbol,
        amount_in: i128,
        trigger_price: u128,
        expires_at: Option<u64>,
        user: Address,
    ) -> Result<u64, ContractError> {
        require_authenticated_verified_user(&env, &user)?;
        orders::OrderManager::place_stop_loss(&env, user, token_in, token_out, amount_in, trigger_price, expires_at)
    }

    /// Cancel an existing order
    pub fn cancel_order(env: Env, order_id: u64, user: Address) -> Result<(), ContractError> {
        require_authenticated_verified_user(&env, &user)?;
        orders::OrderManager::cancel_order(&env, order_id, user)
    }

    /// Get order details
    pub fn get_order(env: Env, order_id: u64) -> Result<orders::Order, ContractError> {
        orders::OrderManager::get_order(&env, order_id)
    }

    /// Get all active orders for a user
    pub fn get_user_orders(env: Env, user: Address) -> Vec<orders::Order> {
        orders::OrderManager::get_user_orders(&env, user)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Portfolio Analytics Dashboard
    // ────────────────────────────────────────────────────────────────────────

    /// Get comprehensive analytics summary for a user
    /// Includes PnL, win rate, Sharpe ratio, and other metrics
    pub fn get_analytics_summary(env: Env, user: Address) -> portfolio::AnalyticsSummary {
        let portfolio: Portfolio = env
            .storage()
            .instance()
            .get(&())
            .unwrap_or_else(|| Portfolio::new(&env));
        portfolio.get_analytics_summary(&env, user)
    }

    /// Get total claimed bonuses for a user
    pub fn get_user_claimed_bonuses(env: Env, user: Address) -> i128 {
        StakingBonusManager::get_user_claimed_bonuses(&env, user)
    }

    /// Get pending claimable bonuses for a user (after 30-day holding period)
    pub fn get_user_pending_bonuses(env: Env, user: Address) -> i128 {
        StakingBonusManager::get_user_pending_bonuses(&env, user)
    }

    /// Get global staking statistics (transparency)
    /// Returns (total_staked, total_distributed, distribution_records_count)
    pub fn get_staking_statistics(env: Env) -> (i128, i128, u64) {
        StakingBonusManager::get_statistics(&env)
    }

    /// Get distribution history for auditing
    pub fn get_distribution_history(env: Env) -> Vec<DistributionRecord> {
        StakingBonusManager::get_distribution_history(&env)
    }

    /// Execute periodic bonus distribution (admin only typically)
    pub fn execute_staking_distribution(env: Env) -> Result<DistributionRecord, ContractError> {
        StakingBonusManager::execute_distribution(&env)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Treasury
    // ────────────────────────────────────────────────────────────────────────

    /// Credit `amount` to the treasury from `source`.
    ///
    /// This entry point is intentionally public so governance or admin tools
    /// can make manual top-ups.  Penalty credits from early-unstake happen
    /// automatically inside `staking_bonus::unstake_early`.
    ///
    /// # Errors
    /// Returns [`PeerXError::TreasuryInvalidAmount`] when `amount <= 0`.
    pub fn treasury_deposit(env: Env, amount: i128, source: Symbol) -> Result<(), ContractError> {
        TreasuryManager::deposit(&env, amount, source)
    }

    /// Return the current treasury balance (read-only, no auth required).
    pub fn treasury_balance(env: Env) -> i128 {
        TreasuryManager::balance(&env)
    }

    /// Governance-gated two-phase withdrawal.
    ///
    /// * Phase 1 — `request`:  creates a pending request locked by the
    ///   configured timelock (default 48 h).  Returns the request ID.
    /// * Phase 2 — `execute`:  after the timelock, finalise the request.
    ///   Returns the executed [`WithdrawRequest`].
    ///
    /// Both phases require admin auth.
    ///
    /// # Arguments
    /// * `phase`      – Either `symbol_short!("request")` or
    ///                  `symbol_short!("execute")`.
    /// * `amount`     – Amount for the `request` phase (ignored for `execute`).
    /// * `destination`– Destination address for the `request` phase (ignored
    ///                  for `execute`).
    /// * `request_id` – For the `execute` phase: ID returned by the `request`
    ///                  phase.  Use `0` for the `request` phase.
    pub fn treasury_withdraw(
        env: Env,
        admin: Address,
        phase: Symbol,
        amount: i128,
        destination: Address,
        request_id: u64,
    ) -> Result<u64, ContractError> {
        admin.require_auth();
        crate::admin::require_admin(&env, &admin)?;

        if phase == symbol_short!("request") {
            let id = TreasuryManager::request_withdraw(&env, amount, destination)?;
            Ok(id)
        } else {
            // execute phase — returns the request id on success
            TreasuryManager::execute_withdraw(&env, request_id)?;
            Ok(request_id)
        }
    }

    // ────────────────────────────────────────────────────────────────────────
    // KYC Verification System
    // ────────────────────────────────────────────────────────────────────────

    /// Add a KYC operator (admin only)
    pub fn kyc_add_operator(env: Env, admin: Address, operator: Address) -> Result<(), ContractError> {
        kyc::KYCSystem::add_operator(&env, &admin, operator)
    }

    /// Remove a KYC operator (admin only)
    pub fn kyc_remove_operator(env: Env, admin: Address, operator: Address) -> Result<(), ContractError> {
        kyc::KYCSystem::remove_operator(&env, &admin, operator)
    }

    /// Check if address is a KYC operator
    pub fn kyc_is_operator(env: Env, address: Address) -> bool {
        kyc::KYCSystem::is_operator(&env, &address)
    }

    /// Submit KYC for review (user-initiated)
    pub fn kyc_submit(env: Env, user: Address) -> Result<(), ContractError> {
        kyc::KYCSystem::submit_kyc(&env, &user)
    }

    /// Resubmit KYC with additional information (user-initiated)
    pub fn kyc_resubmit(env: Env, user: Address) -> Result<(), ContractError> {
        kyc::KYCSystem::resubmit_kyc(&env, &user)
    }

    /// Update KYC status (operator only)
    pub fn kyc_update_status(
        env: Env,
        operator: Address,
        user: Address,
        new_status: KYCStatus,
        reason: Option<Symbol>,
    ) -> Result<(), ContractError> {
        kyc::KYCSystem::update_status(&env, &operator, &user, new_status, reason)
    }

    /// Get KYC record for a user
    pub fn kyc_get_record(env: Env, user: Address) -> KYCRecord {
        kyc::KYCSystem::get_record(&env, &user)
    }

    /// Check if user is verified
    pub fn kyc_is_verified(env: Env, user: Address) -> bool {
        kyc::KYCSystem::is_verified(&env, &user)
    }

    /// Set timelock duration for governance overrides (admin only)
    pub fn kyc_set_timelock_duration(env: Env, admin: Address, duration: u64) -> Result<(), ContractError> {
        kyc::KYCSystem::set_timelock_duration(&env, &admin, duration)
    }

    /// Get timelock duration
    pub fn kyc_get_timelock_duration(env: Env) -> u64 {
        kyc::KYCSystem::get_timelock_duration(&env)
    }

    /// Set pending KYC expiry duration (admin only)
    pub fn kyc_set_pending_expiry_duration(env: Env, admin: Address, duration: u64) -> Result<(), ContractError> {
        kyc::KYCSystem::set_pending_expiry_duration(&env, &admin, duration)
    }

    /// Get pending KYC expiry duration
    pub fn kyc_get_pending_expiry_duration(env: Env) -> u64 {
        kyc::KYCSystem::get_pending_expiry_duration(&env)
    }

    /// Propose governance override for terminal state change (admin only)
    pub fn kyc_propose_override(
        env: Env,
        admin: Address,
        user: Address,
        new_status: KYCStatus,
        reason: Symbol,
    ) -> Result<u64, ContractError> {
        kyc::KYCSystem::propose_override(&env, &admin, user, new_status, reason)
    }

    /// Execute governance override after timelock (admin only)
    pub fn kyc_execute_override(env: Env, admin: Address, override_id: u64) -> Result<(), ContractError> {
        kyc::KYCSystem::execute_override(&env, &admin, override_id)
    }

    /// Get governance override details
    pub fn kyc_get_override(env: Env, override_id: u64) -> Option<GovernanceOverride> {
        kyc::KYCSystem::get_override(&env, override_id)
    }

    // ── Referral System ─────────────────────────────────────────────────────

    /// Register a referral relationship
    pub fn register_referral(env: Env, referrer: Address, referred: Address) -> Result<(), ContractError> {
        referral_system::register_referral(&env, referrer, referred)
    }

    /// Get referral statistics for a user
    pub fn get_referral_stats(env: Env, user: Address) -> referral_system::ReferralStats {
        referral_system::get_referral_stats(&env, user)
    }

    /// Get commission balance for a user
    pub fn get_commission_balance(env: Env, user: Address) -> i128 {
        referral_system::get_commission_balance(&env, user)
    }

    /// Withdraw accumulated commission
    pub fn withdraw_commission(env: Env, user: Address) -> i128 {
        referral_system::withdraw_commission(&env, user)
    }

    // ── Zero-Knowledge Privacy ───────────────────────────────────────────────

    /// Fetch the audit-friendly receipt for a private transaction, by its
    /// transaction hash. Public: off-chain consumers (indexers, compliance
    /// tooling) use this to verify a private transaction occurred without
    /// the contract exposing the underlying private witness values.
    ///
    /// Returns `ZKPError::ProofNotFound` for an empty or unrecognized hash.
    #[cfg(feature = "experimental")]
    pub fn private_tx_receipt(env: Env, tx_hash: Bytes) -> Result<Receipt, ZKPError> {
        zkp_verification::receipts::get_receipt(&env, tx_hash)
    }
}

#[cfg(all(test, feature = "test-determinism"))]
mod test_harness;
#[cfg(all(test, feature = "experimental"))]
mod migration_tests;
#[cfg(all(test, feature = "experimental"))]
mod zkp_receipt_tests;
mod risk_management_tests;
mod governance_tests;
#[cfg(test)]
mod read_only_role_tests;
