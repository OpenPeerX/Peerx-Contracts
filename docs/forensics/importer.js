"use strict";

/**
 * Off-chain JS importer for the `StateSnapshot` forensic snapshot format.
 * Validates a raw JSON payload against the constraints described in
 * schemas/snapshot_state.schema.json and deserializes it into a
 * JS-friendly object (BigInt for i128/u64 fields).
 *
 * Dependency-free by design so it can run in any Node.js (>=14) or
 * browser environment without a bundler.
 */

const ADDRESS_RE = /^[GC][A-Z2-7]{55}$/;
const SYMBOL_RE = /^[A-Za-z0-9_]{1,32}$/;
const INT_STRING_RE = /^-?(0|[1-9][0-9]*)$/;
const UINT_STRING_RE = /^(0|[1-9][0-9]*)$/;

function fail(path, message) {
  throw new Error(`snapshot_state: ${path}: ${message}`);
}

function expectType(value, type, path) {
  if (type === "array") {
    if (!Array.isArray(value)) fail(path, `expected array, got ${typeof value}`);
  } else if (typeof value !== type) {
    fail(path, `expected ${type}, got ${typeof value}`);
  }
}

function parseAddress(value, path) {
  expectType(value, "string", path);
  if (!ADDRESS_RE.test(value)) fail(path, `not a valid strkey address: ${value}`);
  return value;
}

function parseSymbol(value, path) {
  expectType(value, "string", path);
  if (!SYMBOL_RE.test(value)) fail(path, `not a valid symbol: ${value}`);
  return value;
}

function parseI128(value, path) {
  expectType(value, "string", path);
  if (!INT_STRING_RE.test(value)) fail(path, `not a valid i128 decimal string: ${value}`);
  return BigInt(value);
}

function parseU64(value, path) {
  expectType(value, "string", path);
  if (!UINT_STRING_RE.test(value)) fail(path, `not a valid u64 decimal string: ${value}`);
  return BigInt(value);
}

function parseBoolean(value, path) {
  expectType(value, "boolean", path);
  return value;
}

function checkNoExtraKeys(obj, allowed, path) {
  for (const key of Object.keys(obj)) {
    if (!allowed.includes(key)) fail(path, `unexpected field "${key}"`);
  }
}

function parseArray(value, path, itemParser) {
  expectType(value, "array", path);
  return value.map((item, i) => itemParser(item, `${path}[${i}]`));
}

function parseObject(value, path, fields) {
  expectType(value, "object", path);
  if (value === null) fail(path, "expected object, got null");
  checkNoExtraKeys(value, Object.keys(fields), path);
  const out = {};
  for (const [key, parser] of Object.entries(fields)) {
    if (!(key in value)) fail(path, `missing required field "${key}"`);
    out[key] = parser(value[key], `${path}.${key}`);
  }
  return out;
}

function parseBalanceEntry(value, path) {
  const o = parseObject(value, path, {
    address: parseAddress,
    asset: parseSymbol,
    amount: parseI128,
  });
  return { address: o.address, asset: o.asset, amount: o.amount };
}

function parseBadgeEntry(value, path) {
  const o = parseObject(value, path, {
    address: parseAddress,
    badge: parseSymbol,
    earned: parseBoolean,
  });
  return { address: o.address, badge: o.badge, earned: o.earned };
}

function parseTierEntry(value, path) {
  const o = parseObject(value, path, {
    address: parseAddress,
    tier: parseSymbol,
  });
  return { address: o.address, tier: o.tier };
}

function parseBlockVolumeEntry(value, path) {
  const o = parseObject(value, path, {
    block: parseU64,
    volume: parseI128,
  });
  return { block: o.block, volume: o.volume };
}

/**
 * Validates and deserializes a raw JSON `StateSnapshot` payload.
 * Throws an Error describing the first schema violation encountered.
 *
 * @param {unknown} json
 * @returns {{
 *   balances: {address: string, asset: string, amount: bigint}[],
 *   poolXlm: bigint,
 *   poolUsdc: bigint,
 *   totalFees: bigint,
 *   badges: {address: string, badge: string, earned: boolean}[],
 *   tiers: {address: string, tier: string}[],
 *   paused: boolean,
 *   frozenUsers: string[],
 *   blockVolume: {block: bigint, volume: bigint}[],
 * }}
 */
function importSnapshotState(json) {
  const o = parseObject(json, "$", {
    balances: (v, p) => parseArray(v, p, parseBalanceEntry),
    pool_xlm: parseI128,
    pool_usdc: parseI128,
    total_fees: parseI128,
    badges: (v, p) => parseArray(v, p, parseBadgeEntry),
    tiers: (v, p) => parseArray(v, p, parseTierEntry),
    paused: parseBoolean,
    frozen_users: (v, p) => parseArray(v, p, (item, ip) => parseAddress(item, ip)),
    block_volume: (v, p) => parseArray(v, p, parseBlockVolumeEntry),
  });

  return {
    balances: o.balances,
    poolXlm: o.pool_xlm,
    poolUsdc: o.pool_usdc,
    totalFees: o.total_fees,
    badges: o.badges,
    tiers: o.tiers,
    paused: o.paused,
    frozenUsers: o.frozen_users,
    blockVolume: o.block_volume,
  };
}

module.exports = { importSnapshotState };
