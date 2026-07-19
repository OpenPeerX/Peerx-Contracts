"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const { importSnapshotState } = require("../docs/forensics/importer.js");

const SCHEMA_PATH = path.join(__dirname, "..", "schemas", "snapshot_state.schema.json");
const FIXTURE_PATH = path.join(__dirname, "fixtures", "snapshot_state.example.json");

function loadJson(p) {
  return JSON.parse(fs.readFileSync(p, "utf8"));
}

test("schemas/snapshot_state.schema.json is well-formed JSON Schema", () => {
  const schema = loadJson(SCHEMA_PATH);
  assert.equal(schema.title, "StateSnapshot");
  assert.equal(schema.type, "object");
  const required = [
    "balances",
    "pool_xlm",
    "pool_usdc",
    "total_fees",
    "badges",
    "tiers",
    "paused",
    "frozen_users",
    "block_volume",
  ];
  assert.deepEqual(schema.required.slice().sort(), required.slice().sort());
  for (const field of required) {
    assert.ok(field in schema.properties, `schema missing property "${field}"`);
  }
});

test("example fixture matches the documented StateSnapshot field set", () => {
  const schema = loadJson(SCHEMA_PATH);
  const example = loadJson(FIXTURE_PATH);

  for (const field of schema.required) {
    assert.ok(field in example, `fixture missing required field "${field}"`);
  }
  for (const field of Object.keys(example)) {
    assert.ok(field in schema.properties, `fixture has undocumented field "${field}"`);
  }
});

test("off-chain JS importer deserializes the example snapshot", () => {
  const example = loadJson(FIXTURE_PATH);
  const snapshot = importSnapshotState(example);

  assert.equal(snapshot.paused, false);
  assert.equal(snapshot.poolXlm, 500000000000n);
  assert.equal(snapshot.poolUsdc, 250000000000n);
  assert.equal(snapshot.totalFees, 1250000n);

  assert.equal(snapshot.balances.length, 2);
  assert.equal(snapshot.balances[0].asset, "XLM");
  assert.equal(snapshot.balances[0].amount, 10000000000n);
  assert.equal(snapshot.balances[1].amount, -2500000n);

  assert.equal(snapshot.badges.length, 1);
  assert.equal(snapshot.badges[0].badge, "EARLY_LP");
  assert.equal(snapshot.badges[0].earned, true);

  assert.equal(snapshot.tiers[0].tier, "GOLD");
  assert.equal(snapshot.frozenUsers.length, 1);

  assert.equal(snapshot.blockVolume.length, 2);
  assert.equal(snapshot.blockVolume[0].block, 1000042n);
  assert.equal(snapshot.blockVolume[0].volume, 300000000n);
});

test("importer rejects a payload missing a required field", () => {
  const example = loadJson(FIXTURE_PATH);
  const broken = { ...example };
  delete broken.paused;
  assert.throws(() => importSnapshotState(broken), /missing required field "paused"/);
});

test("importer rejects a payload with an unexpected field", () => {
  const example = loadJson(FIXTURE_PATH);
  const broken = { ...example, unexpected_field: 1 };
  assert.throws(() => importSnapshotState(broken), /unexpected field "unexpected_field"/);
});

test("importer rejects a malformed address", () => {
  const example = loadJson(FIXTURE_PATH);
  const broken = JSON.parse(JSON.stringify(example));
  broken.frozen_users[0] = "not-an-address";
  assert.throws(() => importSnapshotState(broken), /not a valid strkey address/);
});

test("importer rejects a non-numeric i128 string", () => {
  const example = loadJson(FIXTURE_PATH);
  const broken = { ...example, pool_xlm: "1.5" };
  assert.throws(() => importSnapshotState(broken), /not a valid i128 decimal string/);
});
