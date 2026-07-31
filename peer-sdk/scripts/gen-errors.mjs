#!/usr/bin/env node
/**
 * gen-errors.mjs — Auto-generate peer-sdk/bindings/errors.ts from errors.rs.
 *
 * Usage:
 *   node peer-sdk/scripts/gen-errors.mjs
 *
 * The script parses the Rust source, extracts every `Variant = N` pair from
 * both PeerXError and SwapChecklistError enums, and writes a TypeScript module
 * with:
 *   • an enum for each error type
 *   • a human-readable label map
 *   • a typed Error class
 *   • decode / parse helpers
 */

import { readFileSync, writeFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ERRORS_RS = resolve(__dirname, "../../peerx-contracts/counter/src/errors.rs");
const OUT_TS = resolve(__dirname, "../bindings/errors.ts");

// ── Parse ───────────────────────────────────────────────────────────────────

function parseEnum(src, enumName) {
  // Grab the block between `pub enum EnumName {` and the closing `}`
  const re = new RegExp(
    `pub\\s+enum\\s+${enumName}\\s*\\{([\\s\\S]*?)\\}`,
    "m",
  );
  const match = src.match(re);
  if (!match) throw new Error(`Cannot find enum ${enumName} in ${ERRORS_RS}`);

  const body = match[1];
  const variants = [];
  const variantRe = /(\w+)\s*=\s*(\d+)/g;
  let m;
  while ((m = variantRe.exec(body)) !== null) {
    variants.push({ name: m[1], code: parseInt(m[2], 10) });
  }
  return variants;
}

const src = readFileSync(ERRORS_RS, "utf8");
const peerxErrors = parseEnum(src, "PeerXError");
const checklistErrors = parseEnum(src, "SwapChecklistError");

// ── Emit ────────────────────────────────────────────────────────────────────

function emitEnum(name, variants) {
  const lines = variants.map(
    (v) => `  ${v.name} = ${v.code},`,
  );
  return `export enum ${name} {\n${lines.join("\n")}\n}`;
}

function emitLabelMap(name, enumName, variants) {
  const lines = variants.map(
    (v) =>
      `  [${enumName}.${v.name}]: "${v.name.replace(/([A-Z])/g, " $1").trim()}",`,
  );
  return `const ${name}: Record<${enumName}, string> = {\n${lines.join("\n")}\n};`;
}

const ts = `\
/**
 * PeerX contract error types — auto-generated from errors.rs.
 *
 * DO NOT EDIT BY HAND. Re-run:
 *   node peer-sdk/scripts/gen-errors.mjs
 */

// ── PeerXError discriminants ────────────────────────────────────────────────

${emitEnum("PeerXErrorCode", peerxErrors)}

// ── SwapChecklistError discriminants ────────────────────────────────────────

${emitEnum("SwapChecklistErrorCode", checklistErrors)}

// ── Human-readable labels ───────────────────────────────────────────────────

${emitLabelMap("PEERX_ERROR_LABELS", "PeerXErrorCode", peerxErrors)}

${emitLabelMap("CHECKLIST_ERROR_LABELS", "SwapChecklistErrorCode", checklistErrors)}

// ── Typed error class ───────────────────────────────────────────────────────

export class PeerXError extends Error {
  readonly code: PeerXErrorCode;

  constructor(code: PeerXErrorCode) {
    super(PEERX_ERROR_LABELS[code] ?? \`Unknown PeerX error \${code}\`);
    this.name = "PeerXError";
    this.code = code;
  }
}

export class SwapChecklistError extends Error {
  readonly code: SwapChecklistErrorCode;

  constructor(code: SwapChecklistErrorCode) {
    super(CHECKLIST_ERROR_LABELS[code] ?? \`Unknown checklist error \${code}\`);
    this.name = "SwapChecklistError";
    this.code = code;
  }
}

// ── Decoding helpers ────────────────────────────────────────────────────────

export function decodePeerXError(raw: number): PeerXError {
  const code = raw as PeerXErrorCode;
  if (!(code in PEERX_ERROR_LABELS)) {
    throw new PeerXError(PeerXErrorCode.InvalidConfig);
  }
  throw new PeerXError(code);
}

export function decodeSwapChecklistError(raw: number): SwapChecklistError {
  const code = raw as SwapChecklistErrorCode;
  if (!(code in CHECKLIST_ERROR_LABELS)) {
    throw new SwapChecklistError(SwapChecklistErrorCode.InvalidSwapPair);
  }
  throw new SwapChecklistError(code);
}

function extractErrorCode(response: unknown): number | null {
  if (typeof response !== "object" || response === null) return null;
  const obj = response as Record<string, unknown>;

  if ("result" in obj && typeof obj.result === "object" && obj.result !== null) {
    const result = obj.result as Record<string, unknown>;
    if ("error" in result && typeof result.error === "string") {
      const match = result.error.match(/(\\d+)$/);
      if (match) return parseInt(match[1], 10);
    }
  }

  if ("error" in obj && typeof obj.error === "object" && obj.error !== null) {
    const err = obj.error as Record<string, unknown>;
    if ("data" in err && typeof err.data === "object" && err.data !== null) {
      const data = err.data as Record<string, unknown>;
      if ("contractCode" in data) {
        const code = data.contractCode;
        if (typeof code === "object" && code !== null && "u32" in code) {
          return (code as { u32: number }).u32;
        }
      }
    }
  }

  return null;
}

export function parseSorobanError(
  response: unknown,
): PeerXError | SwapChecklistError | null {
  const raw = extractErrorCode(response);
  if (raw === null) return null;

  if (raw >= 900 && raw <= 999) {
    return new SwapChecklistError(raw as SwapChecklistErrorCode);
  }
  if (raw >= 1 && raw <= 899) {
    return new PeerXError(raw as PeerXErrorCode);
  }
  return null;
}
`;

writeFileSync(OUT_TS, ts, "utf8");
console.log(`Wrote ${OUT_TS}`);
console.log(`  PeerXError variants: ${peerxErrors.length}`);
console.log(`  SwapChecklistError variants: ${checklistErrors.length}`);
