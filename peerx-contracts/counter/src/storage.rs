use soroban_sdk::{Address, Symbol, contracttype, symbol_short};

pub const ADMIN_KEY: Symbol = symbol_short!("admin");
pub const PAUSED_KEY: Symbol = symbol_short!("paused");
pub const POOL_REGISTRY_KEY: Symbol = symbol_short!("pools");
pub const READ_ONLY_ROLE_KEY: Symbol = symbol_short!("ro_role");

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    // Existing keys
    Admin,
    Paused,
    PoolRegistry,
    
    // Referral system keys
    Referrer(Address),
    ReferralInfo(Address),
    ReferralStats(Address),
    TradingVolume(Address),
    CommissionBalance(Address),
}
