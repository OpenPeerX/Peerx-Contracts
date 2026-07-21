use soroban_sdk::contracterror;

/// Unified error catalog for PeerX contracts.
///
/// Code ranges:
///   1–9      Admin / access control
///   10–19    Trading / contract state
///   100–109  Validation (amounts, tokens, pairs)
///   200–209  Oracle / invariants
///   300–309  Rate limiting / slippage
///   400–409  Liquidity pool
///   500–509  KYC
///   600–609  Staking
///   700–709  Emergency / circuit-breaker
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PeerXError {
    // ── Admin / access control ──────────────────────────────────────────────
    NotAdmin = 1,
    /// Caller is not the currently configured read-only role (see
    /// `set_read_only_role` / `invoke_read`).
    NotReadOnlyRole = 2,
    /// Arguments passed to `invoke_read` didn't decode into the shape the
    /// target read-only function expects.
    InvalidReadArgs = 3,
    /// `invoke_read`'s `fn_name` isn't on the read-only allowlist - this
    /// includes every mutating entry point, by construction.
    UnsupportedReadOnlyFunction = 4,

    // ── Trading / contract state ────────────────────────────────────────────
    TradingPaused = 10,
    UserFrozen = 11,
    CircuitBreakerTripped = 12,

    // ── Validation ──────────────────────────────────────────────────────────
    InvalidAmount = 100,
    AmountOverflow = 101,
    InvalidTokenSymbol = 102,
    InvalidSwapPair = 103,
    InsufficientBalance = 104,
    ZeroAmountSwap = 105,

    // ── Oracle / invariants ─────────────────────────────────────────────────
    InvariantViolation = 200,
    StalePrice = 201,
    InvalidPrice = 202,
    PriceNotSet = 203,
    OracleNotConfigured = 204,
    OracleNotActive = 205,
    CircuitBreakerActive = 206,
    CircuitBreakerTriggered = 207,
    InvalidConfig = 208,

    // ── Rate limiting / slippage ────────────────────────────────────────────
    RateLimitExceeded = 300,
    SlippageExceeded = 301,

    // ── Liquidity pool ──────────────────────────────────────────────────────
    LPPositionNotFound = 400,
    InsufficientLPTokens = 401,

    // ── KYC ─────────────────────────────────────────────────────────────────
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
    /// KYC input data exceeds the allowed size limit (#160).
    KYCDataTooLarge = 511,
    /// Maximum number of KYC operators already registered (#160).
    KYCOperatorLimitReached = 512,

    // ── Staking ─────────────────────────────────────────────────────────────
    InvalidStakeDuration = 600,
    StakeNotFound = 601,
    StakeNotActive = 602,
    StakeLocked = 603,
    NoClaimableBonuses = 604,
    DistributionTooEarly = 605,

    // ── Emergency / circuit-breaker ─────────────────────────────────────────
    NotEmergencyAdmin = 700,

    // ── Referral system ─────────────────────────────────────────────────────
    SelfReferral = 800,
    AlreadyReferred = 801,
    CircularReferral = 802,
}

/// Alias kept for modules that still import `ContractError` by name.
pub type ContractError = PeerXError;

/// Pre-flight checklist result returned by `preflight_swap`.
///
/// Every field is `true` when the corresponding on-chain guard passes.
/// A fully-green checklist means the swap **should** succeed (barring
/// race conditions between the pre-flight read and the actual tx).
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SwapChecklistError {
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

/// Aggregate result of a pre-flight swap validation.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SwapChecklist {
    pub balance_ok: bool,
    pub kyc_ok: bool,
    pub rate_limit_ok: bool,
    pub slippage_ok: bool,
    pub oracle_fresh_ok: bool,
    pub pool_depth_ok: bool,
    pub circuit_breaker_ok: bool,
    pub trading_paused_ok: bool,
    pub pair_ok: bool,
}
