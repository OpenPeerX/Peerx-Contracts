/**
 * PeerX contract error types — auto-generated from errors.rs.
 *
 * DO NOT EDIT BY HAND. Re-run:
 *   node peer-sdk/scripts/gen-errors.mjs
 */

// ── PeerXError discriminants ────────────────────────────────────────────────

export enum PeerXErrorCode {
  NotAdmin = 1,
  NotReadOnlyRole = 2,
  InvalidReadArgs = 3,
  UnsupportedReadOnlyFunction = 4,
  TradingPaused = 10,
  UserFrozen = 11,
  CircuitBreakerTripped = 12,
  InvalidAmount = 100,
  AmountOverflow = 101,
  InvalidTokenSymbol = 102,
  InvalidSwapPair = 103,
  InsufficientBalance = 104,
  ZeroAmountSwap = 105,
  InvariantViolation = 200,
  StalePrice = 201,
  InvalidPrice = 202,
  PriceNotSet = 203,
  OracleNotConfigured = 204,
  OracleNotActive = 205,
  CircuitBreakerActive = 206,
  CircuitBreakerTriggered = 207,
  InvalidConfig = 208,
  RateLimitExceeded = 300,
  SlippageExceeded = 301,
  LPPositionNotFound = 400,
  InsufficientLPTokens = 401,
  KYCVerificationRequired = 500,
  NotKYCOperator = 501,
  InvalidKYCStateTransition = 502,
  KYCTerminalStateImmutable = 503,
  SelfVerificationNotAllowed = 504,
  KYCOverrideNotFound = 505,
  KYCTimelockNotElapsed = 506,
  KYCOverrideAlreadyExecuted = 507,
  InvalidTimelockDuration = 508,
  KYCRequestExpired = 509,
  InvalidExpiryDuration = 510,
  KYCDataTooLarge = 511,
  KYCOperatorLimitReached = 512,
  InvalidStakeDuration = 600,
  StakeNotFound = 601,
  StakeNotActive = 602,
  StakeLocked = 603,
  NoClaimableBonuses = 604,
  DistributionTooEarly = 605,
  NotEmergencyAdmin = 700,
  SelfReferral = 800,
  AlreadyReferred = 801,
  CircularReferral = 802,
}

// ── SwapChecklistError discriminants ────────────────────────────────────────

export enum SwapChecklistErrorCode {
  BalanceCheckFailed = 900,
  KYCNotVerified = 901,
  RateLimitExceeded = 902,
  SlippageCheckFailed = 903,
  OracleStale = 904,
  PoolDepthInsufficient = 905,
  CircuitBreakerActive = 906,
  TradingPaused = 907,
  InvalidSwapPair = 908,
}

// ── Human-readable labels ───────────────────────────────────────────────────

const PEERX_ERROR_LABELS: Record<PeerXErrorCode, string> = {
  [PeerXErrorCode.NotAdmin]: "Not Admin",
  [PeerXErrorCode.NotReadOnlyRole]: "Not Read Only Role",
  [PeerXErrorCode.InvalidReadArgs]: "Invalid Read Args",
  [PeerXErrorCode.UnsupportedReadOnlyFunction]: "Unsupported Read Only Function",
  [PeerXErrorCode.TradingPaused]: "Trading Paused",
  [PeerXErrorCode.UserFrozen]: "User Frozen",
  [PeerXErrorCode.CircuitBreakerTripped]: "Circuit Breaker Tripped",
  [PeerXErrorCode.InvalidAmount]: "Invalid Amount",
  [PeerXErrorCode.AmountOverflow]: "Amount Overflow",
  [PeerXErrorCode.InvalidTokenSymbol]: "Invalid Token Symbol",
  [PeerXErrorCode.InvalidSwapPair]: "Invalid Swap Pair",
  [PeerXErrorCode.InsufficientBalance]: "Insufficient Balance",
  [PeerXErrorCode.ZeroAmountSwap]: "Zero Amount Swap",
  [PeerXErrorCode.InvariantViolation]: "Invariant Violation",
  [PeerXErrorCode.StalePrice]: "Stale Price",
  [PeerXErrorCode.InvalidPrice]: "Invalid Price",
  [PeerXErrorCode.PriceNotSet]: "Price Not Set",
  [PeerXErrorCode.OracleNotConfigured]: "Oracle Not Configured",
  [PeerXErrorCode.OracleNotActive]: "Oracle Not Active",
  [PeerXErrorCode.CircuitBreakerActive]: "Circuit Breaker Active",
  [PeerXErrorCode.CircuitBreakerTriggered]: "Circuit Breaker Triggered",
  [PeerXErrorCode.InvalidConfig]: "Invalid Config",
  [PeerXErrorCode.RateLimitExceeded]: "Rate Limit Exceeded",
  [PeerXErrorCode.SlippageExceeded]: "Slippage Exceeded",
  [PeerXErrorCode.LPPositionNotFound]: "L P Position Not Found",
  [PeerXErrorCode.InsufficientLPTokens]: "Insufficient L P Tokens",
  [PeerXErrorCode.KYCVerificationRequired]: "K Y C Verification Required",
  [PeerXErrorCode.NotKYCOperator]: "Not K Y C Operator",
  [PeerXErrorCode.InvalidKYCStateTransition]: "Invalid K Y C State Transition",
  [PeerXErrorCode.KYCTerminalStateImmutable]: "K Y C Terminal State Immutable",
  [PeerXErrorCode.SelfVerificationNotAllowed]: "Self Verification Not Allowed",
  [PeerXErrorCode.KYCOverrideNotFound]: "K Y C Override Not Found",
  [PeerXErrorCode.KYCTimelockNotElapsed]: "K Y C Timelock Not Elapsed",
  [PeerXErrorCode.KYCOverrideAlreadyExecuted]: "K Y C Override Already Executed",
  [PeerXErrorCode.InvalidTimelockDuration]: "Invalid Timelock Duration",
  [PeerXErrorCode.KYCRequestExpired]: "K Y C Request Expired",
  [PeerXErrorCode.InvalidExpiryDuration]: "Invalid Expiry Duration",
  [PeerXErrorCode.KYCDataTooLarge]: "K Y C Data Too Large",
  [PeerXErrorCode.KYCOperatorLimitReached]: "K Y C Operator Limit Reached",
  [PeerXErrorCode.InvalidStakeDuration]: "Invalid Stake Duration",
  [PeerXErrorCode.StakeNotFound]: "Stake Not Found",
  [PeerXErrorCode.StakeNotActive]: "Stake Not Active",
  [PeerXErrorCode.StakeLocked]: "Stake Locked",
  [PeerXErrorCode.NoClaimableBonuses]: "No Claimable Bonuses",
  [PeerXErrorCode.DistributionTooEarly]: "Distribution Too Early",
  [PeerXErrorCode.NotEmergencyAdmin]: "Not Emergency Admin",
  [PeerXErrorCode.SelfReferral]: "Self Referral",
  [PeerXErrorCode.AlreadyReferred]: "Already Referred",
  [PeerXErrorCode.CircularReferral]: "Circular Referral",
};

const CHECKLIST_ERROR_LABELS: Record<SwapChecklistErrorCode, string> = {
  [SwapChecklistErrorCode.BalanceCheckFailed]: "Balance Check Failed",
  [SwapChecklistErrorCode.KYCNotVerified]: "K Y C Not Verified",
  [SwapChecklistErrorCode.RateLimitExceeded]: "Rate Limit Exceeded",
  [SwapChecklistErrorCode.SlippageCheckFailed]: "Slippage Check Failed",
  [SwapChecklistErrorCode.OracleStale]: "Oracle Stale",
  [SwapChecklistErrorCode.PoolDepthInsufficient]: "Pool Depth Insufficient",
  [SwapChecklistErrorCode.CircuitBreakerActive]: "Circuit Breaker Active",
  [SwapChecklistErrorCode.TradingPaused]: "Trading Paused",
  [SwapChecklistErrorCode.InvalidSwapPair]: "Invalid Swap Pair",
};

// ── Typed error class ───────────────────────────────────────────────────────

export class PeerXError extends Error {
  readonly code: PeerXErrorCode;

  constructor(code: PeerXErrorCode) {
    super(PEERX_ERROR_LABELS[code] ?? `Unknown PeerX error ${code}`);
    this.name = "PeerXError";
    this.code = code;
  }
}

export class SwapChecklistError extends Error {
  readonly code: SwapChecklistErrorCode;

  constructor(code: SwapChecklistErrorCode) {
    super(CHECKLIST_ERROR_LABELS[code] ?? `Unknown checklist error ${code}`);
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
      const match = result.error.match(/(\d+)$/);
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
