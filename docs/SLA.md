# Security Response SLA

**Version**: 1.0
**Effective Date**: 2026-07-17
**Applies To**: All security disclosures filed against the Peerx-Contracts repository
**Linked From**: [`SECURITY.md`](../SECURITY.md)

---

## Overview

This document defines the **Service Level Agreement (SLA)** for handling security
disclosures in the PeerX Contracts project. It tells reporters exactly what to
expect after filing a security issue and gives maintainers a documented,
enforceable response commitment.

Before this document existed, security disclosures had no documented response
SLA. This SLA closes that gap.

---

## SLA Commitments

| Stage | Commitment | Clock Starts |
|-------|------------|--------------|
| **Triage** | **Acknowledge the report within 24 hours** (`< 24 h` ack) | Moment the disclosure is received (issue filed or email sent) |
| **Patch — High severity** | **Fix shipped within 14 days** (`< 14 days`) | Severity confirmed as High |
| **Patch — Medium severity** | **Fix shipped within 30 days** (`< 30 days`) | Severity confirmed as Medium |
| **Disclosure** | **Coordinated, only after the patch is available** | Patch release |

> Commitments are maxima, not targets — the goal is always to respond faster.

---

## Stage Details

### 1. Triage — `< 24 h` acknowledgment

- Every security disclosure (GitHub issue filed with the
  [security vulnerability template](../.github/ISSUE_TEMPLATE/security_vulnerability.md),
  or email to the security contact in [`SECURITY.md`](../SECURITY.md)) receives a
  human acknowledgment **within 24 hours** of receipt.
- The acknowledgment confirms the report was received, assigns a tracking
  reference, and names the maintainer owning triage.
- Initial severity classification (Critical / High / Medium / Low per the table
  below) is completed as part of triage and communicated to the reporter.

### 2. Patch — severity-based remediation windows

| Severity | Patch SLA | Examples |
|----------|-----------|----------|
| **Critical** | Handled as an emergency — immediately escalated; hotfix pursued ahead of all other work, no later than the High window | Unauthorized fund movement, broken invariant enabling theft, auth-bypass on admin functions |
| **High** | **`< 14 days`** from confirmation | Invariant violation, arithmetic overflow reachable by users, oracle manipulation path, pause/freeze failure |
| **Medium** | **`< 30 days`** from confirmation | Rate-limit bypass with bounded impact, slippage-check edge case, minor accounting drift |
| **Low** | Best effort; scheduled with normal roadmap work | Informational findings, hardening suggestions, non-exploitable hygiene items |

- A "patch" means a fix merged to `main` **and** released/deployed such that the
  vulnerability is no longer exploitable.
- If a patch cannot land inside its window (e.g. dependency on an upstream SDK
  fix), the maintainer notifies the reporter **before** the deadline with an
  updated ETA and any available mitigation.

### 3. Disclosure — coordinated after patch

- Details of a vulnerability are **never disclosed publicly before a patch is
  available**.
- Disclosure is **coordinated with the reporter**: once the patch ships,
  maintainers and the reporter agree on a public disclosure date and the content
  of the advisory.
- The public advisory credits the reporter (unless they prefer anonymity) and
  includes severity, affected versions, the fix reference, and mitigations.
- Reporters are asked to keep findings confidential until coordinated disclosure
  occurs.

---

## Severity Classification Guide

Classification follows the impact categories used in
[`SECURITY.md`](../SECURITY.md) (authorization, arithmetic, invariants,
availability):

| Severity | Guidance |
|----------|----------|
| Critical | Direct, exploitable loss of funds or invariant break with immediate impact |
| High | Exploitable weakness with material impact (fund safety, access control, AMM integrity) |
| Medium | Exploitable under constrained conditions, or limited/ bounded impact |
| Low | Informational, theoretical, or defense-in-depth improvements |

If reporter and maintainer disagree on severity, the maintainer's triage lead
makes the call and documents the reasoning in the tracking issue.

---

## Out of Scope

The SLA does **not** apply to:

- Reports for non-security bugs (use the regular issue process).
- Issues in third-party dependencies with no exploitable path through PeerX
  contracts (report upstream; we still track and upgrade).
- Automated scanner output without a demonstrated exploit or impact analysis.
- The experimental `market-data-streaming` crate while it remains
  non-production (still welcome — best-effort response).

## Exception Handling

If a deadline is at risk, maintainers will:

1. Notify the reporter before the deadline passes.
2. Publish the reason and a revised ETA in the tracking issue.
3. Provide a mitigation or workaround where one exists.

Repeated SLA misses trigger a governance review of security-maintenance bandwidth.

---

## How to Report

- **Preferred**: email the security contact listed in
  [`SECURITY.md`](../SECURITY.md#security-contacts).
- **GitHub**: open an issue with the
  [Security Vulnerability template](../.github/ISSUE_TEMPLATE/security_vulnerability.md)
  (omit exploit details publicly; share them privately via the security contact).

Do **not** open public proof-of-concept exploits for unpatched vulnerabilities.

---

## Changelog

| Date | Version | Changes |
|------|---------|---------|
| 2026-07-17 | 1.0 | Initial security response SLA: `< 24 h` triage ack, `< 14 days` High / `< 30 days` Medium patch windows, coordinated post-patch disclosure |
