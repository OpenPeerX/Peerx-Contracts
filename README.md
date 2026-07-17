# PeerX Contracts

> **Production-grade Soroban smart contracts for a risk-free, on-chain trading classroom.**
> PeerX gives users a fully simulated environment to learn market mechanics — AMM swaps, liquidity provision, limit & stop-loss orders, staking, referrals, and portfolio analytics — without exposing real capital.

[![Soroban](https://img.shields.io/badge/Soroban-21.7.1-7C39C9?logo=stellar&logoColor=white)](https://soroban.stellar.org)
[![Rust](https://img.shields.io/badge/Rust-1.74%2B-orange?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](#license)
[![CI](https://img.shields.io/badge/CI-passing-brightgreen?logo=githubactions&logoColor=white)](.github/workflows/ci.yml)
[![Audit](https://img.shields.io/badge/audit-pre--audit--hardening-yellow)](#security-posture)
[![PRs](https://img.shields.io/badge/PRs-welcome-blueviolet)](#contributing)

---

## Table of Contents

- [Overview](#overview)
- [Why PeerX](#why-peerx)
- [Core Capabilities](#core-capabilities)
- [Architecture](#architecture)
- [Repository Layout](#repository-layout)
- [Tech Stack](#tech-stack)
- [Getting Started](#getting-started)
- [Usage Examples](#usage-examples)
- [Security Posture](#security-posture)
- [Testing &amp; Verification](#testing-and-verification)
- [Migrations &amp; Versioning](#migrations-and-versioning)
- [Operational Runbook](#operational-runbook)
- [Performance &amp; Benchmarking](#performance-and-benchmarking)
- [Experimental Modules](#experimental-modules)
- [Contributing](#contributing)
- [License](#license)
- [Contact](#contact)

---

## Overview

**PeerX Contracts** is the on-chain core of PeerX — a Soroban-based educational trading platform that mirrors real-world decentralized exchange mechanics in a fully sandboxed, zero-capital environment.

Every action executes against deterministic contract state with the same primitives users encounter on production AMMs: constant-product pricing, slippage checks, slippage-aware routing across multiple pools, liquidity-provider (LP) positions, fee tiers, and order books. Because the assets are simulated, learners can experiment with adversarial scenarios (front-running, oracle drift, liquidity crises) that would be unsafe and uneconomic against real funds.

## Why PeerX

| Problem | PeerX's approach |
| --- | --- |
| New traders learn by losing real money | Risk-free simulator — failures cost only reputation |
| Most "demo" platforms ignore DEX mechanics | Faithful AMM, LP math, fee curves, oracle slippage |
| Educational tools lack defensive engineering | Hardened contracts with KYC gates, risk limits, circuit breakers, formal invariants |
| Hard to teach advanced order types | Native limit and stop-loss orders on-chain |

---

## Core Capabilities

### 🏦 Trading engine
- **Constant-product AMM swaps** (`XLM` ⇄ simulated issued assets).
- **Multi-pool registry** with admin-managed pool creation and LP shares.
- **Multi-hop routing** — atomic best-route discovery across registered pools.
- **Slippage protection** — configurable basis-point ceiling per swap.
- **Non-panicking `safe_swap`** — records failed orders as a counter rather than reverting.

### 📈 Advanced order types
- **Limit orders** that fill when the oracle price crosses a threshold.
- **Stop-loss orders** that trigger on adverse price movement.
- Order expiry, cancellation, and per-user lookup.

### 💧 Liquidity provision
- **Add / remove liquidity** with proper LP-token minting using the Babylonian integer-sqrt initialization.
- **LP positions** are first-class: per-user deposits, share-of-pool, and proportional withdrawals.

### 👤 Portfolio & identity
- **Multi-asset balances** with non-negative saturation arithmetic.
- **Tier system** (Basic → Silver → Gold → Platinum) drives fee discounts and rate-limit budgets.
- **Portfolio analytics** — PnL, win rate, Sharpe-style ratios, top-traders leaderboard.
- **Instance-storage query cache** with TTL, hit/miss instrumentation, and admin-controlled invalidation.

### 🪪 Compliance
- **KYC state machine** with operators, `Pending` / `Verified` / `Rejected` transitions, and an audit trail.
- **Governance override path** for terminal state changes with a configurable timelock.
- Every sensitive entry point is statically guarded (see `scripts/check_kyc_guards.sh`).

### 🛡 Risk management
- **Circuit breaker** that auto-pauses when swap volume crosses threshold.
- **Concentration limits** — no single user can dominate the pool.
- **Position limits** — per-asset exposure caps.
- **Rate limiter** — tier-aware per-hour budgets for swaps and LP operations.

### 🤝 Social & growth
- **Referral system** — register referrer relationships, earn commissions distributed from swap fees paid by the referred user.
- **Badges & achievements** — on-chain, programmatic, batched emission.

### 🏛 Governance & operations
- **Pause / resume trading** (admin-gated).
- **Freeze / unfreeze** individual accounts.
- **State snapshots** for forensic and recovery workflows.
- **Governance parameters** with proposed-update lifecycle (experimental governance modules suggest design patterns, but stake-weighted voting is **not** implemented).
- **Multi-sig compatible** — admin authority is a single transferable `Address`. Operations teams are expected to secure it behind a multi-sig wallet off-chain.

### 💎 Staking & incentives
- **Staking bonus manager** with 30 / 60 / 90 / 365-day locks, periodic distributions, and 10% early-unstake penalty.
- **Transparent distribution history** for auditing.

### 🧪 Experimental modules
*(feature-gated behind `experimental`)*
- **NFT platform** — minting, marketplace, fractional ownership, lending.
- **ZKP / private transactions** — commitments, range proofs, witness manager, balance proofs.
- **Dynamic fee adjustment** — congestion-aware, history-tracked, with emergency override.
- **Predictive analytics** — confidence-weighted signals.
- **Cross-chain bridge** hooks.

---

## Architecture

```
                  ┌─────────────────────────────────────────┐
                  │              CounterContract            │
                  │   (single WASM artifact, all entry pts) │
                  └──────────────┬──────────────────────────┘
                                 │
        ┌────────────┬───────────┼────────────┬─────────────┐
        ▼            ▼           ▼            ▼             ▼
   Trading        Portfolio     Risk        Governance     Identity
  (AMM, LP,    (balances,    (circuit     (params,       (KYC,
   routing)      analytics)   breaker,    pause, freeze,  tiers)
                                limits)    snapshot)
        │            │           │            │             │
        └────────────┴─────┬─────┴────────────┴─────────────┘
                            ▼
              Instance / Persistent Storage on Soroban
                  (with TTL-scoped query cache)
```

External modules expose a stable, narrow function surface; internal logic is split into focused files (`trading.rs`, `portfolio.rs`, `kyc.rs`, `orders.rs`, `rate_limit.rs`, `risk_management/*`, `governance/*`, …) so features can be reasoned about in isolation.

---

## Repository Layout

```
PeerX-Contracts/
├── Cargo.toml                 # Workspace root (counter, soroban-ping)
├── soroban.toml               # Soroban network profiles (testnet/mainnet/local)
├── README.md                  # ← you are here
├── SECURITY.md                # Auditing, invariants, trust model, SLA summary
├── CONTRIBUTING.md            # Contribution rules + security disclosure SLA acknowledgment
├── docs/
│   └── SLA.md                 # Security response SLA: <24h ack, <14d/<30d patches, coordinated disclosure
├── .github/
│   ├── ISSUE_TEMPLATE/       # Issue templates (security reports acknowledge the SLA)
│   ├── workflows/
│   │   ├── ci.yml             # Single canonical CI (quality + build + test)
│   │   └── README.md          # Workflow rationale
├── scripts/
│   ├── benchmark_cache.py     # Portfolio query cache benchmark runner
│   ├── check_kyc_guards.sh    # Static audit: every sensitive fn has a KYC guard
│   ├── regression_detection.py
│   ├── verify_formal.sh
│   └── build_optimized.sh
├── benches/                   # Release-mode contract benchmarks
├── market-data-streaming/     # Standalone Rust crate (off-chain HFT-grade feed)
└── peerx-contracts/
    ├── counter/               # Main contract crate (CounterContract)
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs                # Contract entry points
    │       ├── trading.rs            # AMM swap & multi-hop routing
    │       ├── portfolio.rs          # Balances, tiers, analytics, caching
    │       ├── orders.rs             # Limit & stop-loss orders
    │       ├── rate_limit.rs         # Tier-aware rate limiter
    │       ├── kyc.rs                # KYC state machine
    │       ├── referral_system.rs    # Referrals & commissions
    │       ├── staking_bonus.rs      # Locked staking & distributions
    │       ├── emergency.rs          # Pause / freeze / snapshot
    │       ├── migration.rs          # Versioned upgrades
    │       ├── events.rs             # Batched on-chain events
    │       └── risk_management/      # Circuit breaker, concentration, limits
    ├── soroban-ping/                  # Hello-world contract, smoke test
    ├── trading_test_runner/           # Off-chain trading test driver
    ├── waitlist/                      # Standalone waitlist contract
    ├── referral/                      # Standalone referral contract
    ├── bounty/                        # Standalone bounty contract
    └── credit-waitlist/               # Standalone credit-waitlist contract
```

---

## Tech Stack

| Layer | Technology |
| --- | --- |
| Smart contracts | **Soroban** (`soroban-sdk = "21.7.1"`) |
| Language | **Rust** (no_std for WASM targets) |
| Build profile | `release`: `opt-level = "z"`, `lto = "fat"`, `panic = "abort"` — small, deterministic WASM |
| Off-chain market data | Standalone Rust crate (`market-data-streaming/`) — WebSocket / REST feeds, order-book analytics, throughput-limiter protections |
| Tooling | Soroban CLI, cargo, rustfmt, clippy |
| CI | GitHub Actions — `quality` → `build` → `test` |

---

## Getting Started

### Prerequisites
- **Rust toolchain** (stable, ≥ 1.74) — [install via rustup](https://rustup.rs).
- **Soroban CLI** — [setup guide](https://soroban.stellar.org/docs/getting-started/setup).
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`.

### Clone & build
```bash
git clone https://github.com/<REPO_URL>.git
cd peerx-contracts

# Type-check the workspace
cargo check --workspace

# Build the WASM artifact (release)
# Binary name comes from peerx-contracts/counter/Cargo.toml [package].name = "counter"
cargo build --release --target wasm32-unknown-unknown \
  --manifest-path peerx-contracts/counter/Cargo.toml

# Run the full test suite
cargo test --workspace --lib
```

### Deploy to testnet
```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/counter.wasm \
  --network testnet \
  --source <YOUR_STELLAR_SECRET_KEY>

soroban contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  --source <YOUR_STELLAR_SECRET_KEY> \
  -- initialize
```

---

## Usage Examples

> All calls shown are thin wrappers around the contract — the same calls are invoked from the JS SDK or CLI.

### Mint a virtual asset
```bash
soroban contract invoke --id <CONTRACT_ID> --network testnet \
  -- mint --token XLM --to <USER_ADDRESS> --amount 1000000
```

### Swap XLM → USDC-SIM
```bash
soroban contract invoke --id <CONTRACT_ID> --network testnet \
  --source <USER_SECRET> \
  -- swap --from XLM --to USDCSIM --amount 1000 --user <USER_ADDRESS>
```

### Add liquidity
```bash
soroban contract invoke --id <CONTRACT_ID> --network testnet \
  --source <USER_SECRET> \
  -- add_liquidity --xlm_amount 5000 --usdc_amount 5000 --user <USER_ADDRESS>
```

### Place a limit order
```bash
soroban contract invoke --id <CONTRACT_ID> --network testnet \
  -- source <USER_SECRET> \
  -- place_limit_order \
       --token_in XLM --token_out USDCSIM \
       --amount_in 100 --limit_price 150 \
       --user <USER_ADDRESS>
```

### Submit KYC
```bash
# User submits
soroban contract invoke --id <CONTRACT_ID> --network testnet \
  --source <USER_SECRET> -- kyc_submit --user <USER_ADDRESS>

# Operator approves after review
soroban contract invoke --id <CONTRACT_ID> --network testnet \
  --source <OPERATOR_SECRET> \
  -- kyc_update_status --operator <OPERATOR_ADDRESS> \
       --user <USER_ADDRESS> --new_status Verified
```

### Get portfolio analytics (cached)
```bash
soroban contract invoke --id <CONTRACT_ID> --network testnet \
  -- get_portfolio --user <USER_ADDRESS>
```

---

## Security Posture

We treat the contract suite as **auditable production code** even though assets are simulated.

- **Authorization** — every privileged function goes through `require_admin` or `require_authenticated_verified_user`. CI statically enforces this (`scripts/check_kyc_guards.sh`).
- **Arithmetic** — all balance and fee math uses `saturating_*` operations; divisions guard zero denominators.
- **Reentrancy** — Soroban's deterministic execution model prevents reentrancy by design; the codebase also never makes cross-contract calls mid-mutation.
- **Risk controls** — circuit breaker, concentration and position limits are evaluated on every swap.
- **Invariants** — asset conservation, AMM constant-product non-increase, fee-bounds, and LP-token conservation are enforced and exposed via formal property tests (`tests/formal_verification_tests.rs`).
- **Rate limits** — tier-aware throttles for both swaps and LP operations, with garbage-collectable per-user state.
- **Emergency** — admin pause, per-account freeze, on-chain state snapshots for forensic recovery.
- **Fuzzing** — `cargo test fuzz_ -- --nocapture` exercises edge cases on every entry point.

See [`SECURITY.md`](SECURITY.md) for the full vulnerability checklist, invariant specifications, and authorization matrix. Security disclosures are handled under a documented [Security Response SLA](docs/SLA.md): acknowledgment **< 24 h**, patches **< 14 days (High)** / **< 30 days (Medium)**, and coordinated disclosure after the patch ships.

---

## Testing &amp; Verification {#testing-and-verification}

```bash
cargo test --workspace --lib --verbose           # Library suites
cargo test --manifest-path peerx-contracts/counter/Cargo.toml \
  --lib kyc_tests -- --nocapture                 # KYC focused
cargo test --manifest-path peerx-contracts/counter/Cargo.toml \
  error_code_tests -- --nocapture                # Error-code regression
cargo test --manifest-path peerx-contracts/counter/Cargo.toml \
  --test formal_verification_tests formal_verification \
  -- --nocapture --test-threads=1                # Property-based invariants
```

CI is a single workflow — `.github/workflows/ci.yml` — running three jobs in series: **quality** (fmt + check + KYC-guard audit) → **build** (full workspace build) → **test** (the four commands above).

---

## Migrations &amp; Versioning {#migrations-and-versioning}

PeerX supports versioned, in-place upgrades with a deterministic migration pipeline.

- `CONTRACT_VERSION` is declared in `lib.rs` and persisted in contract storage.
- `initialize()` is called on fresh deployments to record the active version.
- `migrate()` performs version-to-version data migration; example: `migrate_from_v1_to_v2` augments `Portfolio` with `migration_time`.

**Upgrade checklist**

- [ ] Bump `CONTRACT_VERSION`.
- [ ] Implement `migrate_from_vN_to_vN+1`.
- [ ] Add tests under `migration_tests.rs` simulating the transition.
- [ ] Verify backward compatibility of every persisted struct.
- [ ] Run `migrate()` after deploying the new WASM.

---

## Operational Runbook

For incident response:

| Step | Action | Contract call |
| --- | --- | --- |
| 1 | Investigate | read events, state, snapshots |
| 2 | Halt trading | `emergency_pause(admin)` |
| 3 | Freeze suspect accounts | `freeze_user(admin, user)` |
| 4 | Capture forensic state | `snapshot_state()` |
| 5 | Fix root cause off-chain | deploy new WASM |
| 6 | Migrate & resume | `migrate()` → `emergency_unpause(admin)` |

The **circuit breaker** will auto-pause when swap volume crosses the configured threshold — a safety net, not a substitute for monitoring.

---

## Performance &amp; Benchmarking {#performance-and-benchmarking}

```bash
python3 scripts/benchmark_cache.py
```

Measures cold-vs-warm latency for `get_portfolio` and `get_top_traders`, plus cache hit ratio, against the release-mode ignored test `benchmark_cache_latency_and_hit_ratio`.

For raw contract benchmarks:

```bash
cargo bench --manifest-path benches/Cargo.toml
```

---

## Experimental Modules

The following sit behind the `experimental` feature flag and are **not** mainnet-ready yet:

- 🎨 **NFT platform** — minting, marketplace, fractional ownership, lending (`nft_*` modules).
- 🔐 **ZKP / private transactions** — commitments, range proofs, witness manager (`zkp_*` modules).
- 📈 **Dynamic fee adjustment** — congestion-aware with `FeeHistoryManager` and emergency override.
- 🌉 **Bridge** — cross-chain hooks.
- 📊 **Predictive analytics** — confidence-weighted signals.

Each experimental module has its own test suite gated by the same feature flag; CI does not opt into them, so experiments can iterate quickly without breaking the canonical suite.

---

## Contributing

We welcome pull requests for new educational scenarios, additional test coverage, performance wins, and documentation. Please read [`CONTRIBUTING.md`](CONTRIBUTING.md) — which also acknowledges our [Security Response SLA](docs/SLA.md) for security disclosures — and:

1. Run `cargo fmt --all` before pushing.
2. Add tests for any new entry point.
3. Add the entry point to `scripts/check_kyc_guards.sh` if it touches user funds.
4. Keep PRs focused — one feature or fix per PR.

---

## License

**MIT.** *(No `LICENSE` file is currently committed — add one before publishing. Until then, all rights reserved by default.)*

---

## Contact

- Issues: `https://github.com/<REPO_URL>/issues`
- Security disclosures: **`<SECURITY_EMAIL>`**
- Project: `https://github.com/<REPO_URL>`

*PeerX Contracts — learn markets the hard way, without losing money.*
