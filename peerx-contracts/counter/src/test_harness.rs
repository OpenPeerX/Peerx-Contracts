use soroban_sdk::{
    events::Event,
    testutils::{Address as _, Ledger},
    Address, Env,
};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Deterministic test harness that provides:
///
/// - A pinned ledger timestamp for bit-for-bit reproducible test runs.
/// - `with_event_trace` to capture the event sequence emitted during a
///   block of code, so failing assertions can be tied to a specific event.
/// - Automatic Jest-style timing artifacts written to
///   `$CARGO_TARGET_TMPDIR/timing/<name>.json` on drop.
///
/// # Example
///
/// ```ignore
/// let h = TestHarness::new("my_test");
/// let (result, events) = h.with_event_trace(|env| {
///     // … exercise contract …
/// });
/// assert_eq!(events.len(), 1);
/// ```
pub struct TestHarness {
    pub env: Env,
    test_name: &'static str,
    test_id: u64,
    start: Instant,
}

impl TestHarness {
    /// Create a new deterministic test environment.
    ///
    /// The ledger timestamp is pinned to `1_700_000_000` (Unix epoch) so
    /// every run starts from the same baseline. Call [`set_timestamp`] to
    /// advance during a multi-step test.
    pub fn new(test_name: &'static str) -> Self {
        let env = Env::default();
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);

        env.ledger().with_mut(|l| {
            l.timestamp = 1_700_000_000;
            l.sequence_number = 1;
        });

        TestHarness {
            env,
            test_name,
            test_id,
            start: Instant::now(),
        }
    }

    /// Override the ledger timestamp.
    ///
    /// Uses the same `env.ledger().with_mut` pattern already exercised in
    /// `rate_limit.rs:sensitive_tests` and `kyc_tests.rs`.
    pub fn set_timestamp(&self, ts: u64) {
        self.env.ledger().with_mut(|l| {
            l.timestamp = ts;
        });
    }

    /// Generate a fresh `Address` from the harness environment.
    pub fn generate_address(&self) -> Address {
        Address::generate(&self.env)
    }

    /// Execute `f` and return its result along with every Soroban event
    /// emitted during the call.
    ///
    /// This lets you assert on the exact event sequence produced by a
    /// contract operation:
    ///
    /// ```ignore
    /// let (_, events) = h.with_event_trace(|env| {
    ///     CounterContractClient::new(env, &id).swap(&xlm, &usdc, &amount, &user)
    /// });
    /// assert_eq!(events[0].topic().0, symbol_short!("swap"));
    /// ```
    pub fn with_event_trace<F, T>(&self, f: F) -> (T, Vec<Event>)
    where
        F: FnOnce(&Env) -> T,
    {
        let baseline = self.env.events().all();
        let result = f(&self.env);
        let all = self.env.events().all();

        let new_events: Vec<Event> = (baseline.len()..all.len())
            .filter_map(|i| all.get(i))
            .collect();

        (result, new_events)
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        let elapsed_ms = self.start.elapsed().as_millis();

        if let Ok(tmpdir) = std::env::var("CARGO_TARGET_TMPDIR") {
            let timing_dir = PathBuf::from(tmpdir).join("timing");
            if fs::create_dir_all(&timing_dir).is_ok() {
                let artifact = format!(
                    r#"{{"testName":"{}","durationMs":{},"testId":{}}}"#,
                    self.test_name, elapsed_ms, self.test_id,
                );
                let path = timing_dir.join(format!("{}.json", self.test_name));
                let _ = fs::write(&path, &artifact);
            }
        }
    }
}
