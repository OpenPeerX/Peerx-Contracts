//! Property-based fuzz harness for PeerX swap operations.
//!
//! Uses `proptest` to generate random inputs and verify that core
//! invariants hold under adversarial conditions (dust amounts,
//! exact-1.0 ratios, max u32 swap counts, etc.).
//!
//! Run with: `cargo test fuzz_ --workspace`

use proptest::prelude::*;
use soroban_sdk::symbol_short;

use crate::invariants::*;
use crate::portfolio::{Asset, Portfolio};

// ── Strategy definitions ────────────────────────────────────────────────────

/// Swap amount: covers dust (1), realistic (1..1_000_000), and large values.
fn arb_amount() -> impl Strategy<Value = i128> {
    prop_oneof![
        // Dust amounts that can trigger integer-division edge cases
        1i128..=100,
        // Normal range
        100i128..=1_000_000,
        // Large values
        1_000_000i128..=1_000_000_000_000,
        // Edge: exact powers of 2
        prop::sample::select(vec![1024, 4096, 65536, 1_048_576]),
    ]
}

/// Fee in basis points: 0 (fee-free) up to 100 (max 1%).
fn arb_fee_bps() -> impl Strategy<Value = i128> {
    prop_oneof![
        0i128,     // No fee
        1i128,     // Minimal
        30i128,    // Default 0.3%
        100i128,   // Maximum
        50i128,    // Mid-range
    ]
}

/// Oracle staleness in seconds: fresh (0), moderate (300), stale (600+), way stale (3600).
fn arb_oracle_staleness() -> impl Strategy<Value = u64> {
    prop_oneof![
        0u64,         // Fresh
        300u64,       // 5 min (under threshold)
        599u64,       // Just under 10 min threshold
        600u64,       // Exactly at threshold
        601u64,       // Just over threshold (stale)
        3600u64,      // 1 hour (very stale)
    ]
}

/// Slippage tolerance in basis points: 0 (strict) to 10000 (100%).
fn arb_slippage_bps() -> impl Strategy<Value = u32> {
    prop_oneof![
        0u32,         // No slippage allowed
        100u32,       // 1%
        500u32,       // 5%
        1000u32,      // 10%
        5000u32,      // 50%
        10000u32,     // 100% (anything goes)
    ]
}

/// User balance: covers zero, dust, normal, and whale scenarios.
fn arb_user_balance() -> impl Strategy<Value = i128> {
    prop_oneof![
        0i128,                         // Zero balance
        1i128..=100,                   // Dust
        100i128..=1_000_000,           // Normal
        1_000_000i128..=1_000_000_000, // Whale
    ]
}

/// Pool reserves: (xlm_reserve, usdc_reserve).
fn arb_pool_reserves() -> impl Strategy<Value = (i128, i128)> {
    (
        1000i128..=100_000_000,
        1000i128..=100_000_000,
    )
}

// ── Fuzz targets ────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// Fuzz: AMM constant-product invariant holds for random swap inputs.
    ///
    /// Given random reserves and swap amount, verify that the constant-product
    /// formula (k must not increase) is maintained after the swap.
    #[test]
    fn fuzz_swap_amm_invariant(
        amount in arb_amount(),
        fee_bps in arb_fee_bps(),
        (reserve_xlm, reserve_usdc) in arb_pool_reserves(),
    ) {
        let reserve_in = reserve_xlm as u128;
        let reserve_out = reserve_usdc as u128;
        let amount_u = amount as u128;

        // Calculate k before
        let k_before = reserve_in.saturating_mul(reserve_out);

        // AMM output with fee (same formula as perform_swap in trading.rs)
        let fee_bps_u = fee_bps as u128;
        let amount_after_fee = amount_u.saturating_mul(10000u128.saturating_sub(fee_bps_u)) / 10000;

        let actual_out = if reserve_in > 0 && reserve_out > 0 {
            let num = reserve_out.saturating_mul(amount_after_fee);
            let den = reserve_in.saturating_add(amount_after_fee);
            if den == 0 { 0 } else { num / den }
        } else {
            amount_u
        };

        // New reserves
        let reserve_in_after = reserve_in.saturating_add(amount_after_fee);
        let reserve_out_after = reserve_out.saturating_sub(actual_out);

        // k after should be <= k before (fees reduce k)
        let k_after = reserve_in_after.saturating_mul(reserve_out_after);

        prop_assert!(
            k_after <= k_before,
            "AMM invariant violated: k_before={}, k_after={}, amount={}, fee_bps={}",
            k_before, k_after, amount, fee_bps,
        );
    }

    /// Fuzz: output amount is positive for positive input with valid reserves.
    #[test]
    fn fuzz_swap_positive_output(
        amount in 1i128..=1_000_000_000,
        (reserve_xlm, reserve_usdc) in arb_pool_reserves(),
    ) {
        let reserve_in = reserve_xlm as u128;
        let reserve_out = reserve_usdc as u128;
        let amount_u = amount as u128;

        let amount_after_fee = amount_u * 9970 / 10000; // 0.3% fee

        let actual_out = if reserve_in > 0 && reserve_out > 0 {
            let num = reserve_out.saturating_mul(amount_after_fee);
            let den = reserve_in.saturating_add(amount_after_fee);
            if den == 0 { 0 } else { num / den }
        } else {
            amount_u
        };

        prop_assert!(
            actual_out > 0,
            "Output should be positive for positive input: amount={}, out={}",
            amount, actual_out,
        );
    }

    /// Fuzz: fee is within bounds for all valid inputs.
    #[test]
    fn fuzz_fee_within_bounds(
        amount in arb_amount(),
        fee_bps in arb_fee_bps(),
    ) {
        let fee = (amount * fee_bps) / 10000;

        prop_assert!(fee >= 0, "Fee must be non-negative");
        prop_assert!(fee <= amount, "Fee {} must not exceed amount {}", fee, amount);

        // Fee should not exceed 1% of amount (100 bps max)
        let max_fee = amount / 100;
        prop_assert!(
            fee <= max_fee,
            "Fee {} exceeds 1% of amount {} (max {})",
            fee, amount, max_fee,
        );
    }

    /// Fuzz: slippage calculation is consistent.
    #[test]
    fn fuzz_slippage_consistency(
        (reserve_xlm, reserve_usdc) in arb_pool_reserves(),
        amount in 1i128..=1_000_000,
        max_slippage in arb_slippage_bps(),
    ) {
        let reserve_in = reserve_xlm as u128;
        let reserve_out = reserve_usdc as u128;
        let amount_u = amount as u128;

        // Theoretical output (no fee)
        let theoretical_out = if reserve_in > 0 && reserve_out > 0 {
            let num = reserve_out * amount_u;
            let den = reserve_in + amount_u;
            if den == 0 { 0 } else { num / den }
        } else {
            amount_u
        };

        // Actual output (with 0.3% fee)
        let amount_after_fee = amount_u * 9970 / 10000;
        let actual_out = if reserve_in > 0 && reserve_out > 0 {
            let num = reserve_out.saturating_mul(amount_after_fee);
            let den = reserve_in.saturating_add(amount_after_fee);
            if den == 0 { 0 } else { num / den }
        } else {
            amount_u
        };

        if theoretical_out > 0 {
            let slippage_bps = ((theoretical_out - actual_out) * 10000) / theoretical_out;

            if slippage_bps > max_slippage as u128 {
                // Slippage exceeds tolerance — this is expected, just verify
                // the calculation is consistent (actual_out <= theoretical_out)
                prop_assert!(
                    actual_out <= theoretical_out,
                    "Actual output {} should not exceed theoretical {}",
                    actual_out, theoretical_out,
                );
            }
        }
    }

    /// Fuzz: balance operations maintain non-negative invariant.
    #[test]
    fn fuzz_balance_non_negative(
        initial_balance in 0i128..=1_000_000_000,
        debit_amount in 0i128..=500_000_000,
        credit_amount in 0i128..=500_000_000,
    ) {
        let after_debit = initial_balance.saturating_sub(debit_amount);
        let after_credit = after_debit.saturating_add(credit_amount);

        prop_assert!(
            after_debit >= 0,
            "Balance went negative after debit: {} - {} = {}",
            initial_balance, debit_amount, after_debit,
        );
        prop_assert!(
            after_credit >= 0,
            "Balance went negative after credit: {} + {} = {}",
            after_debit, credit_amount, after_credit,
        );
    }

    /// Fuzz: AMM output is bounded by input amount and pool reserves.
    #[test]
    fn fuzz_swap_output_bounded(
        amount in 1i128..=1_000_000_000,
        (reserve_xlm, reserve_usdc) in arb_pool_reserves(),
    ) {
        let reserve_in = reserve_xlm as u128;
        let reserve_out = reserve_usdc as u128;
        let amount_u = amount as u128;

        let amount_after_fee = amount_u * 9970 / 10000;
        let actual_out = if reserve_in > 0 && reserve_out > 0 {
            let num = reserve_out.saturating_mul(amount_after_fee);
            let den = reserve_in.saturating_add(amount_after_fee);
            if den == 0 { 0 } else { num / den }
        } else {
            amount_u
        };

        // Output should never exceed the output reserve
        if reserve_in > 0 && reserve_out > 0 {
            prop_assert!(
                actual_out <= reserve_out,
                "Output {} exceeds output reserve {}",
                actual_out, reserve_out,
            );
        }

        // Output should be less than input (due to AMM + fees)
        prop_assert!(
            actual_out < amount_u,
            "Output {} should be less than input {} (AMM + fees)",
            actual_out, amount_u,
        );
    }

    /// Fuzz: dust amounts don't cause zero output.
    #[test]
    fn fuzz_dust_swap_no_zero_div(
        dust_amount in 1i128..=100,
        (reserve_xlm, reserve_usdc) in (10000i128..=100_000_000, 10000i128..=100_000_000),
    ) {
        let reserve_in = reserve_xlm as u128;
        let reserve_out = reserve_usdc as u128;
        let amount_u = dust_amount as u128;

        let amount_after_fee = amount_u * 9970 / 10000;

        // Should not panic on division by zero
        let actual_out = if reserve_in > 0 && reserve_out > 0 {
            let num = reserve_out.saturating_mul(amount_after_fee);
            let den = reserve_in.saturating_add(amount_after_fee);
            num / den
        } else {
            amount_u
        };

        prop_assert!(
            actual_out > 0,
            "Dust swap produced zero output: amount={}, reserves={}/{}",
            dust_amount, reserve_xlm, reserve_usdc,
        );
    }
}
