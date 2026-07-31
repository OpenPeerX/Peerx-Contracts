/// Treasury Module
///
/// Collects penalty amounts from early-unstake events (Issue #38) and
/// exposes governance-gated withdrawals protected by a configurable timelock.
///
/// # Storage layout
/// All keys live in `env.storage().persistent()` under `TreasuryKey::*`.
///
/// # Timelock model
/// 1. Admin calls `request_withdraw(amount, destination)` → creates a
///    `WithdrawRequest` and records the current timestamp + timelock duration.
/// 2. After the timelock elapses, admin calls `execute_withdraw(request_id)` to
///    finalise.  The treasury balance is debited at that point.
///
/// # Source field
/// `deposit` accepts a free-form `Symbol` source tag (e.g. `"penalty"`,
/// `"manual"`) that is recorded in the event log for auditing.
use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

use crate::errors::PeerXError;

// ────────────────────────────────────────────────────────────────────────────
// Constants
// ────────────────────────────────────────────────────────────────────────────

/// Default governance timelock: 48 hours in seconds.
pub const DEFAULT_TREASURY_TIMELOCK_SECS: u64 = 48 * 60 * 60;

// ────────────────────────────────────────────────────────────────────────────
// Data Structures
// ────────────────────────────────────────────────────────────────────────────

/// A pending governance withdrawal.
#[derive(Clone, Debug)]
#[contracttype]
pub struct WithdrawRequest {
    /// Unique request identifier (monotonically increasing counter).
    pub id: u64,
    /// Amount to be withdrawn.
    pub amount: i128,
    /// Destination address that will receive the funds.
    pub destination: Address,
    /// Timestamp when the request was created.
    pub requested_at: u64,
    /// Earliest timestamp at which the request can be executed.
    pub executable_after: u64,
    /// Whether the request has already been executed.
    pub executed: bool,
}

/// Persistent storage keys for the treasury.
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum TreasuryKey {
    /// Current treasury balance (i128).
    Balance,
    /// Governance timelock duration in seconds (u64).
    TimelockDuration,
    /// Next available request ID counter (u64).
    NextRequestId,
    /// Individual withdrawal request keyed by ID.
    WithdrawRequest(u64),
    /// Total lifetime deposits (i128) — for auditing.
    TotalDeposited,
    /// Total lifetime withdrawals (i128) — for auditing.
    TotalWithdrawn,
}

// ────────────────────────────────────────────────────────────────────────────
// Treasury Manager
// ────────────────────────────────────────────────────────────────────────────

/// Stateless manager; all state lives in persistent contract storage.
pub struct TreasuryManager;

impl TreasuryManager {
    // ────────────────────────────────────────────────────────────────────────
    // Deposits
    // ────────────────────────────────────────────────────────────────────────

    /// Credit `amount` to the treasury.
    ///
    /// # Arguments
    /// * `env`    – Contract environment.
    /// * `amount` – Positive amount to deposit.
    /// * `source` – Short descriptive tag for the deposit source (e.g.
    ///              `symbol_short!("penalty")`).
    ///
    /// # Errors
    /// Returns [`PeerXError::TreasuryInvalidAmount`] when `amount <= 0`.
    pub fn deposit(env: &Env, amount: i128, source: Symbol) -> Result<(), PeerXError> {
        if amount <= 0 {
            return Err(PeerXError::TreasuryInvalidAmount);
        }

        let prev: i128 = env
            .storage()
            .persistent()
            .get(&TreasuryKey::Balance)
            .unwrap_or(0);

        let new_balance = prev.saturating_add(amount);
        env.storage()
            .persistent()
            .set(&TreasuryKey::Balance, &new_balance);

        // Update lifetime deposit counter.
        let prev_total: i128 = env
            .storage()
            .persistent()
            .get(&TreasuryKey::TotalDeposited)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&TreasuryKey::TotalDeposited, &prev_total.saturating_add(amount));

        env.events().publish(
            (symbol_short!("trs_dep"), source),
            (amount, new_balance),
        );

        Ok(())
    }

    // ────────────────────────────────────────────────────────────────────────
    // Balance Query
    // ────────────────────────────────────────────────────────────────────────

    /// Return the current treasury balance.  Always succeeds; returns 0 when
    /// no deposits have been made yet.
    pub fn balance(env: &Env) -> i128 {
        env.storage()
            .persistent()
            .get(&TreasuryKey::Balance)
            .unwrap_or(0)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Timelock Configuration
    // ────────────────────────────────────────────────────────────────────────

    /// Return the configured timelock duration (seconds).
    /// Falls back to [`DEFAULT_TREASURY_TIMELOCK_SECS`] when not yet set.
    pub fn get_timelock_duration(env: &Env) -> u64 {
        env.storage()
            .persistent()
            .get(&TreasuryKey::TimelockDuration)
            .unwrap_or(DEFAULT_TREASURY_TIMELOCK_SECS)
    }

    /// Override the timelock duration.  Admin-only call-site enforcement is
    /// expected at the entry-point layer in `lib.rs`.
    pub fn set_timelock_duration(env: &Env, duration_secs: u64) {
        env.storage()
            .persistent()
            .set(&TreasuryKey::TimelockDuration, &duration_secs);
    }

    // ────────────────────────────────────────────────────────────────────────
    // Withdrawal — request phase
    // ────────────────────────────────────────────────────────────────────────

    /// Create a pending withdrawal request.
    ///
    /// The request is locked until `now + timelock_duration`.  Returns the
    /// newly created request ID.
    ///
    /// # Errors
    /// * [`PeerXError::TreasuryInvalidAmount`]      – `amount <= 0`.
    /// * [`PeerXError::TreasuryInsufficientFunds`]  – `amount > current balance`.
    pub fn request_withdraw(
        env: &Env,
        amount: i128,
        destination: Address,
    ) -> Result<u64, PeerXError> {
        if amount <= 0 {
            return Err(PeerXError::TreasuryInvalidAmount);
        }

        let current_balance = Self::balance(env);
        if amount > current_balance {
            return Err(PeerXError::TreasuryInsufficientFunds);
        }

        let now = env.ledger().timestamp();
        let timelock = Self::get_timelock_duration(env);
        let executable_after = now.saturating_add(timelock);

        // Allocate a unique request ID.
        let request_id: u64 = env
            .storage()
            .persistent()
            .get(&TreasuryKey::NextRequestId)
            .unwrap_or(0);

        let request = WithdrawRequest {
            id: request_id,
            amount,
            destination: destination.clone(),
            requested_at: now,
            executable_after,
            executed: false,
        };

        env.storage()
            .persistent()
            .set(&TreasuryKey::WithdrawRequest(request_id), &request);

        // Advance counter.
        env.storage()
            .persistent()
            .set(&TreasuryKey::NextRequestId, &(request_id + 1));

        env.events().publish(
            (symbol_short!("trs_req"), destination),
            (request_id, amount, executable_after),
        );

        Ok(request_id)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Withdrawal — execute phase
    // ────────────────────────────────────────────────────────────────────────

    /// Execute a previously created withdrawal request after the timelock has
    /// elapsed.
    ///
    /// On success the treasury balance is debited and the request is marked as
    /// executed.  The caller is responsible for actually transferring the funds
    /// to `request.destination` at the entry-point layer (this module only
    /// tracks the on-chain balance accounting).
    ///
    /// # Returns
    /// The [`WithdrawRequest`] that was executed (so the caller can read
    /// `destination` and `amount`).
    ///
    /// # Errors
    /// * [`PeerXError::TreasuryWithdrawNotFound`]      – Unknown `request_id`.
    /// * [`PeerXError::TreasuryWithdrawAlreadyExecuted`] – Already executed.
    /// * [`PeerXError::TreasuryTimelockNotElapsed`]    – Too early.
    /// * [`PeerXError::TreasuryInsufficientFunds`]     – Balance was reduced
    ///   since the request was created.
    pub fn execute_withdraw(env: &Env, request_id: u64) -> Result<WithdrawRequest, PeerXError> {
        let mut request: WithdrawRequest = env
            .storage()
            .persistent()
            .get(&TreasuryKey::WithdrawRequest(request_id))
            .ok_or(PeerXError::TreasuryWithdrawNotFound)?;

        if request.executed {
            return Err(PeerXError::TreasuryWithdrawAlreadyExecuted);
        }

        let now = env.ledger().timestamp();
        if now < request.executable_after {
            return Err(PeerXError::TreasuryTimelockNotElapsed);
        }

        let current_balance = Self::balance(env);
        if request.amount > current_balance {
            return Err(PeerXError::TreasuryInsufficientFunds);
        }

        // Debit the treasury.
        let new_balance = current_balance.saturating_sub(request.amount);
        env.storage()
            .persistent()
            .set(&TreasuryKey::Balance, &new_balance);

        // Update lifetime withdrawal counter.
        let prev_withdrawn: i128 = env
            .storage()
            .persistent()
            .get(&TreasuryKey::TotalWithdrawn)
            .unwrap_or(0);
        env.storage().persistent().set(
            &TreasuryKey::TotalWithdrawn,
            &prev_withdrawn.saturating_add(request.amount),
        );

        // Mark the request as executed.
        request.executed = true;
        env.storage()
            .persistent()
            .set(&TreasuryKey::WithdrawRequest(request_id), &request);

        env.events().publish(
            (symbol_short!("trs_exe"), request.destination.clone()),
            (request_id, request.amount, new_balance),
        );

        Ok(request)
    }

    // ────────────────────────────────────────────────────────────────────────
    // Queries
    // ────────────────────────────────────────────────────────────────────────

    /// Retrieve a withdrawal request by ID.
    pub fn get_withdraw_request(
        env: &Env,
        request_id: u64,
    ) -> Result<WithdrawRequest, PeerXError> {
        env.storage()
            .persistent()
            .get(&TreasuryKey::WithdrawRequest(request_id))
            .ok_or(PeerXError::TreasuryWithdrawNotFound)
    }

    /// Return (total_deposited, total_withdrawn, current_balance) for auditing.
    pub fn get_stats(env: &Env) -> (i128, i128, i128) {
        let deposited: i128 = env
            .storage()
            .persistent()
            .get(&TreasuryKey::TotalDeposited)
            .unwrap_or(0);
        let withdrawn: i128 = env
            .storage()
            .persistent()
            .get(&TreasuryKey::TotalWithdrawn)
            .unwrap_or(0);
        let balance = Self::balance(env);
        (deposited, withdrawn, balance)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::{Address, Env};

    fn make_env() -> Env {
        Env::default()
    }

    /// Helper: advance ledger time by `secs` seconds.
    fn advance_time(env: &Env, secs: u64) {
        env.ledger().with_mut(|l| {
            l.timestamp = l.timestamp.saturating_add(secs);
        });
    }

    #[test]
    fn deposit_increases_balance() {
        let env = make_env();
        env.mock_all_auths();

        TreasuryManager::deposit(&env, 1_000, symbol_short!("penalty")).unwrap();
        assert_eq!(TreasuryManager::balance(&env), 1_000);

        TreasuryManager::deposit(&env, 500, symbol_short!("manual")).unwrap();
        assert_eq!(TreasuryManager::balance(&env), 1_500);
    }

    #[test]
    fn deposit_rejects_zero_or_negative() {
        let env = make_env();
        env.mock_all_auths();

        assert_eq!(
            TreasuryManager::deposit(&env, 0, symbol_short!("penalty")),
            Err(PeerXError::TreasuryInvalidAmount)
        );
        assert_eq!(
            TreasuryManager::deposit(&env, -1, symbol_short!("penalty")),
            Err(PeerXError::TreasuryInvalidAmount)
        );
    }

    #[test]
    fn request_withdraw_creates_record() {
        let env = make_env();
        env.mock_all_auths();

        let dest = Address::generate(&env);
        TreasuryManager::deposit(&env, 2_000, symbol_short!("penalty")).unwrap();
        let id = TreasuryManager::request_withdraw(&env, 500, dest.clone()).unwrap();

        let req = TreasuryManager::get_withdraw_request(&env, id).unwrap();
        assert_eq!(req.amount, 500);
        assert_eq!(req.destination, dest);
        assert!(!req.executed);
        // Balance unchanged until execute
        assert_eq!(TreasuryManager::balance(&env), 2_000);
    }

    #[test]
    fn request_withdraw_rejects_insufficient_funds() {
        let env = make_env();
        env.mock_all_auths();

        let dest = Address::generate(&env);
        TreasuryManager::deposit(&env, 100, symbol_short!("penalty")).unwrap();
        assert_eq!(
            TreasuryManager::request_withdraw(&env, 200, dest),
            Err(PeerXError::TreasuryInsufficientFunds)
        );
    }

    #[test]
    fn execute_withdraw_before_timelock_fails() {
        let env = make_env();
        env.mock_all_auths();

        let dest = Address::generate(&env);
        TreasuryManager::deposit(&env, 1_000, symbol_short!("penalty")).unwrap();
        let id = TreasuryManager::request_withdraw(&env, 1_000, dest).unwrap();

        // Only advance 1 hour (< 48h default timelock)
        advance_time(&env, 3_600);

        assert_eq!(
            TreasuryManager::execute_withdraw(&env, id),
            Err(PeerXError::TreasuryTimelockNotElapsed)
        );
    }

    #[test]
    fn execute_withdraw_after_timelock_succeeds() {
        let env = make_env();
        env.mock_all_auths();

        let dest = Address::generate(&env);
        TreasuryManager::deposit(&env, 1_000, symbol_short!("penalty")).unwrap();
        let id = TreasuryManager::request_withdraw(&env, 600, dest.clone()).unwrap();

        // Advance past the 48h default timelock
        advance_time(&env, DEFAULT_TREASURY_TIMELOCK_SECS + 1);

        let executed_req = TreasuryManager::execute_withdraw(&env, id).unwrap();
        assert!(executed_req.executed);
        assert_eq!(executed_req.destination, dest);
        assert_eq!(TreasuryManager::balance(&env), 400);
    }

    #[test]
    fn execute_withdraw_double_execution_fails() {
        let env = make_env();
        env.mock_all_auths();

        let dest = Address::generate(&env);
        TreasuryManager::deposit(&env, 1_000, symbol_short!("penalty")).unwrap();
        let id = TreasuryManager::request_withdraw(&env, 1_000, dest).unwrap();
        advance_time(&env, DEFAULT_TREASURY_TIMELOCK_SECS + 1);

        TreasuryManager::execute_withdraw(&env, id).unwrap();
        assert_eq!(
            TreasuryManager::execute_withdraw(&env, id),
            Err(PeerXError::TreasuryWithdrawAlreadyExecuted)
        );
    }

    #[test]
    fn get_stats_reflects_all_operations() {
        let env = make_env();
        env.mock_all_auths();

        let dest = Address::generate(&env);
        TreasuryManager::deposit(&env, 1_000, symbol_short!("penalty")).unwrap();
        TreasuryManager::deposit(&env, 200, symbol_short!("manual")).unwrap();

        let id = TreasuryManager::request_withdraw(&env, 300, dest).unwrap();
        advance_time(&env, DEFAULT_TREASURY_TIMELOCK_SECS + 1);
        TreasuryManager::execute_withdraw(&env, id).unwrap();

        let (deposited, withdrawn, balance) = TreasuryManager::get_stats(&env);
        assert_eq!(deposited, 1_200);
        assert_eq!(withdrawn, 300);
        assert_eq!(balance, 900);
    }

    #[test]
    fn custom_timelock_is_respected() {
        let env = make_env();
        env.mock_all_auths();

        // Set a very short timelock of 10 seconds
        TreasuryManager::set_timelock_duration(&env, 10);

        let dest = Address::generate(&env);
        TreasuryManager::deposit(&env, 500, symbol_short!("penalty")).unwrap();
        let id = TreasuryManager::request_withdraw(&env, 500, dest).unwrap();

        // Should fail before 10s
        advance_time(&env, 5);
        assert_eq!(
            TreasuryManager::execute_withdraw(&env, id),
            Err(PeerXError::TreasuryTimelockNotElapsed)
        );

        // Should succeed after 10s
        advance_time(&env, 6); // total 11s
        TreasuryManager::execute_withdraw(&env, id).unwrap();
        assert_eq!(TreasuryManager::balance(&env), 0);
    }
}
