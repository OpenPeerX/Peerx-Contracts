# Refactor: removal of `referral/src/Gas/gas.rs` (Issue #32)

## Problem

`peerx-contracts/referral/src/Gas/gas.rs` contained three unrelated function
blocks bundled into a single file:

1. `record_volume` — belongs to the referral commission logic
2. `release_batch` — belongs to `credit-waitlist/src/onboarding.rs`
3. `claim` — belongs to `bounty/src/payout.rs`

The file also declared `use ...storage::Status;` and
`use ...storage::ReportStatus;` simultaneously, neither of which matched its
own contents. It could not compile standalone, and was silently skipped by
CI because `referral`, `bounty`, and `credit-waitlist` are not yet listed as
workspace members in the root `Cargo.toml` (tracked separately in #3).

## Resolution

- Confirmed `record_volume` already exists in its canonical location,
  `referral/src/commission.rs`, matching the issue's stated source of truth.
- Confirmed `release_batch` and `claim` are reconciled in their canonical
  locations (`credit-waitlist/src/onboarding.rs` and `bounty/src/payout.rs`
  respectively).
- Deleted `peerx-contracts/referral/src/Gas/gas.rs` and its now-empty parent
  directory (`Gas/`).

## Follow-up

Once #3 lands (adding `referral`, `bounty`, `credit-waitlist` to the
workspace `members` list), these crates will be included in
`cargo check --workspace` and CI will catch regressions like this one going
forward.