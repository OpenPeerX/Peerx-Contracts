# `snapshot_state` forensic snapshot format

## Purpose

The emergency module's forensic snapshot function
(`emergency::snapshot` in
[`peerx-contracts/counter/src/emergency.rs`](../../peerx-contracts/counter/src/emergency.rs),
invoked operationally as `snapshot_state()` — see the Operational
Runbook in [`README.md`](../../README.md)) captures a point-in-time
view of critical contract state for use during incident response.
It returns a `StateSnapshot` struct.

Soroban contracts exchange values as XDR (`ScVal`), not JSON. Forensic
tooling that pulls a snapshot off-chain (via `simulateTransaction` /
`getLedgerEntries` and XDR decoding) needs a stable, documented JSON
shape to parse into. That shape is defined machine-readably in
[`schemas/snapshot_state.schema.json`](../../schemas/snapshot_state.schema.json).

## Field reference

| JSON field     | Rust type                          | Notes                                                                 |
|----------------|-------------------------------------|------------------------------------------------------------------------|
| `balances`     | `Vec<((Address, Symbol), i128)>`   | Array of `{ address, asset, amount }`. `amount` is a decimal string.  |
| `pool_xlm`     | `i128`                              | Decimal string.                                                        |
| `pool_usdc`    | `i128`                              | Decimal string.                                                        |
| `total_fees`   | `i128`                              | Decimal string.                                                        |
| `badges`       | `Vec<((Address, Symbol), bool)>`   | Array of `{ address, badge, earned }`.                                 |
| `tiers`        | `Vec<(Address, Symbol)>`           | Array of `{ address, tier }`.                                          |
| `paused`       | `bool`                              | Emergency pause switch state.                                          |
| `frozen_users` | `Vec<Address>`                     | Array of strkey address strings.                                       |
| `block_volume` | `Vec<(u64, i128)>`                 | Array of `{ block, volume }`; both encoded as decimal strings.        |

`i128` and `u64` values are encoded as decimal **strings**, not JSON
numbers, because both types can exceed JavaScript's safe integer
range (`Number.MAX_SAFE_INTEGER`, 2^53 - 1). `Address` and `Symbol`
values are encoded as their Stellar strkey / symbol string
representations respectively.

Rust tuples (`(Address, Symbol)`) and tuple-keyed maps
(`Vec<((Address, Symbol), i128)>`) have no native JSON equivalent, so
they are projected as arrays of named objects — this keeps the schema
self-describing for forensic reviewers who aren't reading the Rust
source alongside it.

## Validating a snapshot

[`schemas/snapshot_state.schema.json`](../../schemas/snapshot_state.schema.json)
is a standard JSON Schema (2020-12 dialect) and can be used with any
conforming validator (`ajv`, `jsonschema`, etc.).

An example snapshot payload is provided at
[`tests/fixtures/snapshot_state.example.json`](../../tests/fixtures/snapshot_state.example.json)
and is validated against the schema, then deserialized with the
off-chain JS importer, in
[`tests/snapshot_state_schema.test.js`](../../tests/snapshot_state_schema.test.js).

## Off-chain JS importer

[`docs/forensics/importer.js`](importer.js) exports
`importSnapshotState(json)`, which validates a raw JSON snapshot
against the schema's constraints and deserializes it into a
JS-friendly object with `BigInt` amounts:

```js
const { importSnapshotState } = require("../../docs/forensics/importer.js");
const fs = require("fs");

const raw = JSON.parse(fs.readFileSync("snapshot.json", "utf8"));
const snapshot = importSnapshotState(raw);

console.log(snapshot.poolXlm); // BigInt
console.log(snapshot.frozenUsers); // string[]
```

It throws a descriptive `Error` if the payload does not match the
schema (missing/extra fields, malformed addresses, non-numeric
amount strings, etc.), so it can be used both as a runtime guard in
forensic pipelines and as the basis for schema-conformance tests.
