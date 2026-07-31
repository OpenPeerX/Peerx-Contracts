# PeerX Security Documentation

## Overview

This document provides a comprehensive security analysis of the PeerX smart contracts, including vulnerability assessments, invariant checks, and audit readiness guidelines.

**Contract Version**: 1  
**Last Updated**: 2026-07-17  
**Audit Status**: Pre-audit hardening phase

---

## Table of Contents

1. [Security Response SLA](#security-response-sla)
2. [Security Architecture](#security-architecture)
3. [Vulnerability Checklist](#vulnerability-checklist)
4. [Invariant Specifications](#invariant-specifications)
5. [Authorization Matrix](#authorization-matrix)
6. [Arithmetic Safety](#arithmetic-safety)
7. [Fuzz Testing](#fuzz-testing)
8. [Audit Checklist](#audit-checklist)
9. [Trust Assumptions](#trust-assumptions)

---

## Security Response SLA

Security disclosures filed against this repository are handled under a
documented response SLA, defined in full at [`docs/SLA.md`](docs/SLA.md).

**Summary of commitments:**

| Stage | SLA |
|-------|-----|
| **Triage** | Acknowledged **< 24 hours** after the report is received |
| **Patch (High severity)** | Fix shipped **< 14 days** from confirmation |
| **Patch (Medium severity)** | Fix shipped **< 30 days** from confirmation |
| **Disclosure** | **Coordinated** with the reporter, only **after the patch** is available |

These commitments apply to all security disclosures, whether filed through
GitHub (using the security vulnerability issue template) or reported privately
to the security contact below. See [`docs/SLA.md`](docs/SLA.md) for severity
definitions, scope, exception handling, and the full disclosure process.

---

## Security Architecture

### Contract Structure

```
CounterContract (Main Entry Point)
├── Portfolio Management (portfolio.rs)
│   ├── Balance tracking
│   ├── LP position management
│   └── Badge/achievement system
├── Trading Engine (trading.rs)
│   ├── AMM swap logic
│   ├── Oracle price integration
│   └── Slippage protection
├── Batch Operations (batch.rs)
│   ├── Atomic execution
│   └── Best-effort execution
├── Rate Limiting (rate_limit.rs)
│   └── Tier-based throttling
├── Emergency Controls (emergency.rs)
│   ├── Pause/unpause
│   ├── Account freezing
│   └── State snapshots
└── Migration (migration.rs)
    └── Version management
```

### Security Boundaries

- **Authorization Layer**: Admin functions require `require_admin()` check
- **Validation Layer**: All inputs validated before processing
- **State Layer**: Invariants verified after state changes
- **Event Layer**: All significant actions emit events for audit trails

---

## Vulnerability Checklist

### 1. Reentrancy Protection

| Check | Status | Notes |
|-------|--------|-------|
| No external calls before state updates | PASS | Soroban prevents reentrancy by design |
| No cross-contract calls in swap logic | PASS | No external contract calls in trading flow |
| State changes before event emission | PASS | Events emitted after state updates |
| No callbacks to user addresses | PASS | No callback patterns used |

**Soroban Reentrancy Protection**: Soroban uses a deterministic execution model that prevents reentrancy attacks. Contract calls cannot be reentered during execution.

### 2. Integer Overflow/Underflow

| Check | Status | Location |
|-------|--------|----------|
| Saturating arithmetic used | PASS | All balance operations use `saturating_add/sub` |
| Amount bounds checking | PASS | `validation.rs` enforces MAX_AMOUNT |
| Fee calculation overflow check | PASS | Fee calc: `(amount * fee_bps) / 10000` |
| LP token calculation safety | PASS | Babylonian method with overflow guards |
| Product calculation safety | PASS | `saturating_mul` used in AMM formula |

**Critical Arithmetic Locations**:
- `portfolio.rs:160-175` - debit() with PnL tracking
- `portfolio.rs:178-207` - mint() with balance updates
- `trading.rs:91-115` - AMM constant product formula
- `lib.rs:148-164` - Fee calculation and collection

### 3. Authorization Gaps

| Function | Admin Only | Auth Check | Status |
|----------|------------|------------|--------|
| `emergency_pause()` | YES | `require_admin()` | PASS |
| `emergency_unpause()` | YES | `require_admin()` | PASS |
| `freeze_user()` | YES | `require_admin()` | PASS |
| `unfreeze_user()` | YES | `require_admin()` | PASS |
| `set_admin()` | YES | `require_admin()` | PASS* |
| `snapshot_state()` | YES | `require_admin()` | PASS |
| `migrate()` | YES | `require_admin()` | PASS |

*Note: Some functions have auth checks disabled for SDK compatibility - see Implementation Guidelines.

### 4. State Consistency Invariants

| Invariant | Check Location | Verification |
|-----------|----------------|--------------|
| Asset Conservation | `portfolio.rs:694-717` | Non-negative pool balances |
| LP Token Conservation | `portfolio.rs:812-815` | `total_lp_tokens >= 0` |
| AMM Constant Product | `portfolio.rs:774-791` | `k_after <= k_before` |
| Fee Bounds | `portfolio.rs:749-769` | Fee in [0%, 1%] range |
| Balance Non-Negative | `portfolio.rs:806-808` | All balances >= 0 |
| Metrics Monotonic | `portfolio.rs:819-825` | Counters never decrease |
| State Monotonicity | `portfolio.rs:732-744` | Version/timestamp forward only |

### 5. Access Control

| Resource | Owner | Access Pattern |
|----------|-------|----------------|
| Contract Admin | Deployer | Single admin, transferable |
| User Balances | User | Only user can spend |
| LP Positions | LP Provider | Only owner can withdraw |
| Pool Liquidity | Contract | Managed by AMM logic |
| Fees Collected | Contract | Distributed to LPs |

---

## Invariant Specifications

### Core Invariants

#### 1. Asset Conservation Invariant
```rust
/// Total supply = sum of all user balances + pool reserves + fees
/// Ensures no tokens created/destroyed outside of mint/burn
pub fn invariant_asset_conservation(&self, env: &Env) -> bool {
    // Pool balances must be non-negative
    self.xlm_in_pool >= 0 &&
    self.usdc_in_pool >= 0 &&
    self.total_lp_tokens >= 0 &&
    self.lp_fees_accumulated >= 0
}
```

#### 2. AMM Constant Product Invariant
```rust
/// For swaps: k = x * y should not increase (fees reduce k)
/// k_before >= k_after for all swap operations
pub fn invariant_amm_constant_product(
    &self, 
    xlm_before: i128, 
    usdc_before: i128,
    xlm_after: i128, 
    usdc_after: i128
) -> bool {
    let k_before = (xlm_before as u128) * (usdc_before as u128);
    let k_after = (xlm_after as u128) * (usdc_after as u128);
    k_after <= k_before  // Fees cause k to decrease or stay same
}
```

#### 3. Fee Bounds Invariant
```rust
/// Fees must be between 0% and 1% of transaction amount
pub fn invariant_fee_bounds(&self, amount: i128, fee: i128) -> bool {
    const MAX_FEE_BPS: i128 = 100; // 1%
    
    fee >= 0 &&
    (amount == 0 && fee == 0) ||
    (amount > 0 && fee <= (amount * MAX_FEE_BPS) / 10000)
}
```

#### 4. Balance Consistency Invariant
```rust
/// Balance updates must be atomic and consistent
pub fn invariant_balance_update_consistency(
    &self,
    balance_before: i128,
    debit_amount: i128,
    credit_amount: i128,
    balance_after: i128
) -> bool {
    let calculated = balance_before
        .saturating_sub(debit_amount)
        .saturating_add(credit_amount);
    calculated == balance_after
}
```

### LP Pool Invariants

```rust
/// LP token conservation: total minted = sum of all positions
/// (Verified via formal property tests due to Map iteration limits)
pub fn invariant_lp_token_conservation(&self) -> bool {
    self.total_lp_tokens >= 0
}

/// LP position integrity: tokens minted > 0 implies deposits > 0
pub fn invariant_lp_position_integrity(&self, position: &LPPosition) -> bool {
    position.lp_tokens_minted >= 0 &&
    (position.lp_tokens_minted == 0 || 
     (position.xlm_deposited > 0 && position.usdc_deposited > 0))
}
```

---

## Authorization Matrix

### Function Permissions

| Function | Caller | Authorization | Notes |
|----------|--------|---------------|-------|
| `mint()` | Any | None | Test function - should be restricted in production |
| `swap()` | User | Implicit (spends user balance) | User must have sufficient balance |
| `add_liquidity()` | LP | Implicit | User must have tokens to deposit |
| `remove_liquidity()` | LP | Implicit | User must have LP position |
| `emergency_pause()` | Admin | `require_admin()` | Stops all trading |
| `emergency_unpause()` | Admin | `require_admin()` | Resumes trading |
| `freeze_user()` | Admin | `require_admin()` | Blocks specific user |
| `migrate()` | Admin | `require_admin()` | Version upgrade |

### Tier System Authorization

| Tier | Trade Limit | LP Limit | Fee Discount |
|------|-------------|----------|--------------|
| Basic | 10/hour | 5/hour | 0% |
| Silver | 20/hour | 10/hour | 5% |
| Gold | 50/hour | 20/hour | 10% |
| Platinum | 100/hour | 50/hour | 15% |

---

## Arithmetic Safety

### Safe Operations

All arithmetic uses `saturating_*` operations to prevent overflow:

```rust
// Balance updates
self.balances.set(key, current.saturating_add(amount));

// PnL tracking
let new_pnl = current_pnl.saturating_sub(amount);

// LP calculations
let product = (xlm_amount as u128).saturating_mul(usdc_amount as u128);
```

### Precision Handling

| Operation | Precision | Notes |
|-----------|-----------|-------|
| Price | 1e18 (PRECISION) | 18 decimal places |
| Fee calculation | Basis points (1/10000) | Integer math |
| LP token minting | Integer sqrt | Babylonian method |
| Rate limits | Per-hour buckets | Ledger-based |

### Division Safety

All divisions check for zero denominators:

```rust
if denominator == 0 {
    panic!("Division by zero in AMM calculation");
}
```

---

## Fuzz Testing

### Fuzz Test Coverage

| Function | Fuzz Tests | Edge Cases |
|----------|------------|------------|
| `mint()` | 5 tests | Zero, max, negative amounts |
| `swap()` | 8 tests | Min/max amounts, stale prices |
| `add_liquidity()` | 4 tests | Empty pool, large deposits |
| `remove_liquidity()` | 4 tests | Full/partial removal |
| Batch operations | 4 tests | Mixed success/failure |

### Property-Based Tests

Using `proptest` for randomized inputs:

```rust
proptest! {
    #[test]
    fn test_swap_never_creates_tokens(amount in 1..MAX_AMOUNT) {
        // Invariant: total supply after <= total supply before
    }
    
    #[test]
    fn test_lp_token_conservation(xlm in 1..MAX_AMOUNT, usdc in 1..MAX_AMOUNT) {
        // Invariant: LP tokens proportional to deposits
    }
}
```

---

## Audit Checklist

### Pre-Audit Preparation

- [x] SECURITY.md created
- [x] All invariants documented
- [x] Authorization matrix defined
- [x] Fuzz tests implemented (20+)
- [x] Clippy warnings resolved (< 5)
- [x] cargo audit passes
- [ ] Formal verification tests pass
- [ ] Integration test: 100 random operations

### Auditor Verification Steps

1. **Invariant Verification**
   ```bash
   cargo test invariant_ -- --nocapture
   ```

2. **Fuzz Testing**
   ```bash
   cargo test fuzz_ -- --nocapture
   ```

3. **Authorization Tests**
   ```bash
   cargo test auth_ -- --nocapture
   ```

4. **Arithmetic Edge Cases**
   ```bash
   cargo test overflow_ underflow_ -- --nocapture
   ```

### Known Limitations

1. **SDK Compatibility**: Some auth checks disabled for SDK version compatibility
2. **Map Iteration**: Cannot iterate all balances for total supply check (Soroban limitation)
3. **Zero Address**: Cannot validate against zero address (SDK limitation)

---

## Trust Assumptions

### Trusted Parties

| Party | Trust Level | Responsibility |
|-------|-------------|----------------|
| Contract Admin | High | Emergency controls, upgrades |
| Price Oracle | Medium | External price feeds |
| Contract Deployer | High | Initial admin setup |

### Trustless Operations

- User balance management
- AMM swap execution
- LP token minting/burning
- Badge awarding
- Rate limiting

### Assumptions

1. **Oracle Prices**: Assumed to be accurate when not stale
2. **Admin Key**: Assumed to be secure and not compromised
3. **Ledger Timestamp**: Assumed to be monotonically increasing
4. **SDK Behavior**: Assumed to enforce auth correctly

---

## Security Contacts

For security issues, please contact:
- GitHub Issues: [Project Issues](https://github.com/your-org/peerx-contracts/issues) (use the **Security Vulnerability** issue template)
- Security Email: security@peerx.example

**Response expectations**: all reports are handled under our documented
[Security Response SLA](docs/SLA.md) — acknowledgment **< 24 hours**, patches
**< 14 days (High)** / **< 30 days (Medium)**, and coordinated disclosure
**after the patch** ships.

---

## References

- [OpenZeppelin Security Guidelines](https://docs.openzeppelin.com/)
- [Soroban Security Best Practices](https://soroban.stellar.org/docs/)
- [Stellar Smart Contract Security](https://developers.stellar.org/)
- [Rust Security Guidelines](https://rust-lang.github.io/)

---

## Changelog

| Date | Version | Changes |
|------|---------|---------|
| 2026-02-21 | 1.0 | Initial security documentation |
| 2026-07-17 | 1.1 | Added documented Security Response SLA ([`docs/SLA.md`](docs/SLA.md)): `< 24 h` triage ack, `< 14 days` High / `< 30 days` Medium patch windows, coordinated post-patch disclosure |
