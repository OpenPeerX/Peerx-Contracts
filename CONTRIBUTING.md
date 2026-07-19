# Contributing to PeerX Contracts

We welcome pull requests for new educational scenarios, additional test
coverage, performance wins, and documentation.

## Getting Started

```bash
git clone <repository-url>
cd Peerx-Contracts
cargo build --workspace
cargo test --workspace --lib
```

## Contribution Guidelines

1. Run `cargo fmt --all` before pushing.
2. Add tests for any new entry point.
3. Add the entry point to `scripts/check_kyc_guards.sh` if it touches user funds.
4. Keep PRs focused — one feature or fix per PR.

## Security Disclosures

**Do not open public proof-of-concept exploits for unpatched vulnerabilities.**

Security reports are handled under our documented
[Security Response SLA](docs/SLA.md). By filing a security disclosure you will
receive, at minimum:

- **Triage**: acknowledgment of your report **< 24 hours** after it is received.
- **Patch**: a shipped fix **< 14 days** for High-severity and **< 30 days** for
  Medium-severity issues, counted from severity confirmation.
- **Disclosure**: public disclosure **coordinated with you, only after the
  patch** is available.

To report:

1. Email the security contact listed in [`SECURITY.md`](SECURITY.md#security-contacts), **or**
2. Open a GitHub issue using the **Security Vulnerability** template
   (keep exploit details out of the public issue; share them privately).

Read the full policy in [`SECURITY.md`](SECURITY.md) and the SLA in
[`docs/SLA.md`](docs/SLA.md).

## Code of Conduct

Be respectful and constructive. We're building an educational tool — the
community around it should be too.
