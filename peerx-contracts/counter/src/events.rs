use soroban_sdk::{contracttype, Address, Env, Symbol, Vec};

use crate::observability::{log, LogLevel};

#[contracttype]
#[derive(Clone)]
pub struct BadgeEvent {
    pub user: Address,
    pub badge: crate::portfolio::Badge,
    pub timestamp: i64,
}

/// On-chain histogram of swap sizes.  Published as a `SwapDistribution` event
/// every `SWAP_DISTRIBUTION_CADENCE` swaps (default 1 000) or on admin
/// trigger via `emit_swap_distribution`.
///
/// Buckets (inclusive lower, exclusive upper):
///   • `size_0_100`    — [0,   100)
///   • `size_100_1k`   — [100, 1 000)
///   • `size_1k_10k`   — [1 000, 10 000)
///   • `size_10k_plus` — [10 000, ∞)
#[contracttype]
#[derive(Clone)]
pub struct SwapDistribution {
    pub size_0_100: u64,
    pub size_100_1k: u64,
    pub size_1k_10k: u64,
    pub size_10k_plus: u64,
    /// Total swap count captured in this snapshot.
    pub total_swaps: u64,
    pub timestamp: i64,
}

/// Number of swaps between automatic `SwapDistribution` emissions.
pub const SWAP_DISTRIBUTION_CADENCE: u64 = 1_000;

const EVENT_BUFFER_KEY: Symbol = Symbol::short("evt_buf");
/// Persistent key for the global swap counter.
const SWAP_COUNT_KEY: Symbol = Symbol::short("sw_cnt");
/// Persistent key for the per-bucket histogram counters (stored as a 4-element
/// tuple to keep a single storage round-trip).
/// Layout: (size_0_100, size_100_1k, size_1k_10k, size_10k_plus)
const SWAP_BUCKETS_KEY: Symbol = Symbol::short("sw_bkt");

pub struct Events;

impl Events {
    pub fn swap_executed(
        env: &Env,
        from_token: Symbol,
        to_token: Symbol,
        from_amount: i128,
        to_amount: i128,
        user: Address,
        timestamp: i64,
    ) {
        log(
            env,
            LogLevel::Info,
            (Symbol::new(env, "SwapExecuted"), user, from_token, to_token),
            (from_amount, to_amount, timestamp),
        );
    }

    /// Increment the global swap counter and update the size histogram.
    /// Automatically emits a `SwapDistribution` event when the counter
    /// crosses a `SWAP_DISTRIBUTION_CADENCE` boundary.
    pub fn record_swap(env: &Env, amount: i128, timestamp: i64) {
        // --- update counter ---
        let prev_count: u64 = env
            .storage()
            .instance()
            .get(&SWAP_COUNT_KEY)
            .unwrap_or(0u64);
        let new_count = prev_count.saturating_add(1);
        env.storage().instance().set(&SWAP_COUNT_KEY, &new_count);

        // --- update bucket histogram ---
        let (mut b0, mut b1, mut b2, mut b3): (u64, u64, u64, u64) = env
            .storage()
            .instance()
            .get(&SWAP_BUCKETS_KEY)
            .unwrap_or((0u64, 0u64, 0u64, 0u64));
        match amount {
            a if a < 100 => b0 = b0.saturating_add(1),
            a if a < 1_000 => b1 = b1.saturating_add(1),
            a if a < 10_000 => b2 = b2.saturating_add(1),
            _ => b3 = b3.saturating_add(1),
        }
        env.storage()
            .instance()
            .set(&SWAP_BUCKETS_KEY, &(b0, b1, b2, b3));

        // --- auto-emit on cadence boundary ---
        if new_count % SWAP_DISTRIBUTION_CADENCE == 0 {
            Self::emit_swap_distribution_inner(env, b0, b1, b2, b3, new_count, timestamp);
        }
    }

    /// Admin-triggered emission of the current swap distribution histogram.
    /// Resets buckets after emission so each window is independent.
    /// Caller must have already been authorized before calling this function.
    pub fn emit_swap_distribution(env: &Env, _admin: Address, timestamp: i64) {
        let total_swaps: u64 = env
            .storage()
            .instance()
            .get(&SWAP_COUNT_KEY)
            .unwrap_or(0u64);
        let (b0, b1, b2, b3): (u64, u64, u64, u64) = env
            .storage()
            .instance()
            .get(&SWAP_BUCKETS_KEY)
            .unwrap_or((0u64, 0u64, 0u64, 0u64));
        Self::emit_swap_distribution_inner(env, b0, b1, b2, b3, total_swaps, timestamp);
        // Reset buckets after manual flush so the next window starts fresh.
        env.storage()
            .instance()
            .set(&SWAP_BUCKETS_KEY, &(0u64, 0u64, 0u64, 0u64));
    }

    /// Returns the current swap counter (useful for tests / analytics).
    pub fn swap_count(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&SWAP_COUNT_KEY)
            .unwrap_or(0u64)
    }

    fn emit_swap_distribution_inner(
        env: &Env,
        b0: u64,
        b1: u64,
        b2: u64,
        b3: u64,
        total_swaps: u64,
        timestamp: i64,
    ) {
        let distribution = SwapDistribution {
            size_0_100: b0,
            size_100_1k: b1,
            size_1k_10k: b2,
            size_10k_plus: b3,
            total_swaps,
            timestamp,
        };
        env.events().publish(
            (Symbol::new(env, "SwapDistribution"),),
            distribution,
        );
    }

    pub fn liquidity_added(
        env: &Env,
        xlm_amount: i128,
        usdc_amount: i128,
        lp_tokens_minted: i128,
        user: Address,
        timestamp: i64,
    ) {
        log(
            env,
            LogLevel::Info,
            (Symbol::new(env, "LiquidityAdded"), user),
            (xlm_amount, usdc_amount, lp_tokens_minted, timestamp),
        );
    }

    pub fn liquidity_removed(
        env: &Env,
        xlm_amount: i128,
        usdc_amount: i128,
        lp_tokens_burned: i128,
        user: Address,
        timestamp: i64,
    ) {
        log(
            env,
            LogLevel::Info,
            (Symbol::new(env, "LiquidityRemoved"), user),
            (xlm_amount, usdc_amount, lp_tokens_burned, timestamp),
        );
    }

    pub fn badge_awarded(env: &Env, user: Address, badge: crate::portfolio::Badge, timestamp: i64) {
        let mut buffer: Vec<BadgeEvent> = env
            .storage()
            .temporary()
            .get(&EVENT_BUFFER_KEY)
            .unwrap_or_else(|| Vec::new(env));
        buffer.push_back(BadgeEvent {
            user,
            badge,
            timestamp,
        });
        env.storage().temporary().set(&EVENT_BUFFER_KEY, &buffer);
    }

    pub fn flush_badge_events(env: &Env) {
        let buffer: Option<Vec<BadgeEvent>> = env.storage().temporary().get(&EVENT_BUFFER_KEY);
        if let Some(events) = buffer {
            if !events.is_empty() {
                log(
                    env,
                    LogLevel::Info,
                    (Symbol::new(env, "BadgesAwarded"),),
                    events,
                );
                env.storage().temporary().remove(&EVENT_BUFFER_KEY);
            }
        }
    }

    pub fn user_tier_changed(
        env: &Env,
        user: Address,
        old_tier: crate::tiers::UserTier,
        new_tier: crate::tiers::UserTier,
        timestamp: i64,
    ) {
        log(
            env,
            LogLevel::Info,
            (Symbol::new(env, "UserTierChanged"), user),
            (old_tier, new_tier, timestamp),
        );
    }

    pub fn admin_paused(env: &Env, admin: Address, timestamp: i64) {
        log(
            env,
            LogLevel::Warn,
            (Symbol::new(env, "AdminPaused"), admin),
            (timestamp,),
        );
    }

    pub fn admin_resumed(env: &Env, admin: Address, timestamp: i64) {
        log(
            env,
            LogLevel::Warn,
            (Symbol::new(env, "AdminResumed"), admin),
            (timestamp,),
        );
    }

    /// Emitted when the admin grants (or replaces) the read-only
    /// auditor/dashboard role. Used to track access-control changes to the
    /// `invoke_read` dispatch path.
    ///
    /// Topic  : ("ReadOnlyRoleSet", admin_address)
    /// Payload: (role_address, timestamp)
    pub fn read_only_role_set(env: &Env, admin: Address, role: Address, timestamp: u64) {
        env.events().publish(
            (Symbol::new(env, "ReadOnlyRoleSet"), admin),
            (role, timestamp),
        );
    }
}

/// Emitted whenever an alert fires. Carries enough metadata for an
/// off-chain indexer to route a push notification or webhook call.
///
/// Topic  : ("AlertTriggered", owner_address, alert_id)
/// Payload: (alert_kind, notification_method, timestamp)
///
/// NOTE: This event is also emitted directly inside `alerts.rs` via
/// `emit_alert_triggered`. This stub documents the schema for the audit
/// trail and can be called from `events.rs` if you prefer to centralise
/// event emission in future.
pub fn alert_triggered(
    env: &Env,
    owner: Address,
    alert_id: u64,
    // Using Symbol here keeps the payload ABI-stable regardless of the
    // internal AlertKind enum layout across contract upgrades.
    kind_tag: Symbol,
    notification_method_tag: Symbol,
    timestamp: u64,
) {
    log(
        env,
        LogLevel::Warn,
        (Symbol::new(env, "AlertTriggered"), owner, alert_id),
        (kind_tag, notification_method_tag, timestamp),
    );
}

/// Emitted when an alert is created so indexers can track the full
/// lifecycle (create → trigger → cleanup) without polling storage.
///
/// Topic  : ("AlertCreated", owner_address, alert_id)
/// Payload: (kind_tag, expires_at)
pub fn alert_created(env: &Env, owner: Address, alert_id: u64, kind_tag: Symbol, expires_at: u64) {
    log(
        env,
        LogLevel::Info,
        (Symbol::new(env, "AlertCreated"), owner, alert_id),
        (kind_tag, expires_at),
    );
}

#[cfg(feature = "experimental")]
/// Emitted when performance metrics are calculated for a user.
/// Used for tracking portfolio performance analytics.
///
/// Topic  : ("PerformanceMetricsCalculated", user_address)
/// Payload: (time_window, sharpe_ratio, max_drawdown, timestamp)
#[cfg(feature = "experimental")]
pub fn performance_metrics_calculated(
    env: &Env,
    user: Address,
    time_window: crate::analytics::TimeWindow,
    sharpe_ratio: u128,
    max_drawdown: u128,
    timestamp: i64,
) {
    log(
        env,
        LogLevel::Debug,
        (Symbol::new(env, "PerformanceMetricsCalculated"), user),
        (time_window, sharpe_ratio, max_drawdown, timestamp),
    );
}

#[cfg(feature = "experimental")]
/// Emitted when asset allocation analysis is completed.
/// Used for portfolio diversification tracking.
///
/// Topic  : ("AssetAllocationAnalyzed", user_address)
/// Payload: (total_assets, diversification_score, timestamp)
#[cfg(feature = "experimental")]
pub fn asset_allocation_analyzed(
    env: &Env,
    user: Address,
    total_assets: u32,
    diversification_score: u128,
    timestamp: i64,
) {
    log(
        env,
        LogLevel::Debug,
        (Symbol::new(env, "AssetAllocationAnalyzed"), user),
        (total_assets, diversification_score, timestamp),
    );
}

#[cfg(feature = "experimental")]
/// Emitted when benchmark comparison is calculated.
/// Used for performance relative to market benchmarks.
///
/// Topic  : ("BenchmarkComparisonCalculated", user_address, benchmark_id)
/// Payload: (alpha, beta, timestamp)
#[cfg(feature = "experimental")]
pub fn benchmark_comparison_calculated(
    env: &Env,
    user: Address,
    benchmark_id: Symbol,
    alpha: i128,
    beta: u128,
    timestamp: i64,
) {
    log(
        env,
        LogLevel::Debug,
        (
            Symbol::new(env, "BenchmarkComparisonCalculated"),
            user,
            benchmark_id,
        ),
        (alpha, beta, timestamp),
    );
}

#[cfg(feature = "experimental")]
/// Emitted when period returns are calculated.
/// Used for tracking returns over specific time periods.
///
/// Topic  : ("PeriodReturnsCalculated", user_address)
/// Payload: (start_timestamp, end_timestamp, time_weighted_return, timestamp)
#[cfg(feature = "experimental")]
pub fn period_returns_calculated(
    env: &Env,
    user: Address,
    start_timestamp: u64,
    end_timestamp: u64,
    time_weighted_return: i128,
    timestamp: i64,
) {
    log(
        env,
        LogLevel::Debug,
        (Symbol::new(env, "PeriodReturnsCalculated"), user),
        (
            start_timestamp,
            end_timestamp,
            time_weighted_return,
            timestamp,
        ),
    );
}

/// Emitted when network congestion level changes.
/// Used for monitoring network health.
///
/// Topic  : ("NetworkCongestionChanged",)
/// Payload: (previous_level_tag, new_level_tag, capacity_utilization, timestamp)
pub fn network_congestion_changed(
    env: &Env,
    previous_level: Symbol,
    new_level: Symbol,
    capacity_utilization: u32,
    timestamp: u64,
) {
    log(
        env,
        LogLevel::Warn,
        (Symbol::new(env, "NetworkCongestionChanged"),),
        (previous_level, new_level, capacity_utilization, timestamp),
    );
}

/// Emitted when trading fees are adjusted due to congestion.
/// Used for tracking fee changes and their triggers.
///
/// Topic  : ("FeeAdjustmentApplied",)
/// Payload: (previous_fee_bps, new_fee_bps, adjustment_reason_tag, congestion_level_tag, timestamp)
pub fn fee_adjustment_applied(
    env: &Env,
    previous_fee_bps: u32,
    new_fee_bps: u32,
    adjustment_reason: Symbol,
    congestion_level: Symbol,
    timestamp: u64,
) {
    log(
        env,
        LogLevel::Info,
        (Symbol::new(env, "FeeAdjustmentApplied"),),
        (
            previous_fee_bps,
            new_fee_bps,
            adjustment_reason,
            congestion_level,
            timestamp,
        ),
    );
}

/// Emitted when emergency fee override is activated.
/// Used for alerting on extreme network conditions.
///
/// Topic  : ("EmergencyFeeOverrideActivated",)
/// Payload: (fee_cap_bps, reason_tag, timestamp)
pub fn emergency_fee_override_activated(
    env: &Env,
    fee_cap_bps: u32,
    reason: Symbol,
    timestamp: u64,
) {
    log(
        env,
        LogLevel::Error,
        (Symbol::new(env, "EmergencyFeeOverrideActivated"),),
        (fee_cap_bps, reason, timestamp),
    );
}

/// Emitted when emergency fee override is deactivated.
/// Used for tracking recovery from extreme conditions.
///
/// Topic  : ("EmergencyFeeOverrideDeactivated",)
/// Payload: (timestamp,)
pub fn emergency_fee_override_deactivated(env: &Env, timestamp: u64) {
    log(
        env,
        LogLevel::Warn,
        (Symbol::new(env, "EmergencyFeeOverrideDeactivated"),),
        (timestamp,),
    );
}

/// Emitted when fee adjustment configuration is updated.
/// Used for audit trail of configuration changes.
///
/// Topic  : ("FeeConfigurationUpdated",)
/// Payload: (admin_address, config_change_tag, timestamp)
pub fn fee_configuration_updated(env: &Env, admin: Address, change_type: Symbol, timestamp: u64) {
    log(
        env,
        LogLevel::Info,
        (Symbol::new(env, "FeeConfigurationUpdated"), admin),
        (change_type, timestamp),
    );
}

/// Emitted periodically with current fee statistics.
/// Used for analytics and monitoring.
///
/// Topic  : ("FeeStatisticsReport",)
/// Payload: (avg_fee_bps, min_fee_bps, max_fee_bps, volatility, timestamp)
pub fn fee_statistics_report(
    env: &Env,
    avg_fee_bps: u32,
    min_fee_bps: u32,
    max_fee_bps: u32,
    volatility: u32,
    timestamp: u64,
) {
    log(
        env,
        LogLevel::Debug,
        (Symbol::new(env, "FeeStatisticsReport"),),
        (avg_fee_bps, min_fee_bps, max_fee_bps, volatility, timestamp),
    );
}
