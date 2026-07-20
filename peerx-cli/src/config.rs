use crate::error::{CliError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for PeerX CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Soroban RPC endpoint URL
    pub rpc_url: String,
    
    /// Contract ID for the PeerX contract
    pub contract_id: String,
    
    /// Admin address for health checks
    pub admin_address: Option<String>,
    
    /// Network (testnet, mainnet, local)
    pub network: String,
    
    /// Oracle endpoint URL (if external oracle)
    pub oracle_url: Option<String>,
    
    /// Timeout for health checks in seconds
    pub timeout_seconds: u64,
    
    /// Maximum acceptable oracle staleness in seconds
    pub oracle_max_staleness_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rpc_url: "https://soroban-testnet.stellar.org".to_string(),
            contract_id: String::new(),
            admin_address: None,
            network: "testnet".to_string(),
            oracle_url: None,
            timeout_seconds: 30,
            oracle_max_staleness_seconds: 300, // 5 minutes
        }
    }
}

impl Config {
    /// Load configuration from environment variables and config file
    pub fn load() -> Result<Self> {
        let mut config = Self::default();
        
        // Override with environment variables
        if let Ok(rpc_url) = std::env::var("PEERX_RPC_URL") {
            config.rpc_url = rpc_url;
        }
        
        if let Ok(contract_id) = std::env::var("PEERX_CONTRACT_ID") {
            config.contract_id = contract_id;
        }
        
        if let Ok(admin_address) = std::env::var("PEERX_ADMIN_ADDRESS") {
            config.admin_address = Some(admin_address);
        }
        
        if let Ok(network) = std::env::var("PEERX_NETWORK") {
            config.network = network;
        }
        
        if let Ok(oracle_url) = std::env::var("PEERX_ORACLE_URL") {
            config.oracle_url = Some(oracle_url);
        }
        
        if let Ok(timeout) = std::env::var("PEERX_TIMEOUT_SECONDS") {
            config.timeout_seconds = timeout.parse().unwrap_or(30);
        }
        
        if let Ok(staleness) = std::env::var("PEERX_ORACLE_MAX_STALENESS_SECONDS") {
            config.oracle_max_staleness_seconds = staleness.parse().unwrap_or(300);
        }
        
        // Try to load from config file if it exists
        if let Ok(file_config) = Self::load_from_file() {
            config = config.merge(file_config);
        }
        
        Ok(config)
    }
    
    /// Load configuration from file
    fn load_from_file() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            return Err(CliError::Config("Config file not found".to_string()));
        }
        
        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = serde_json::from_str(&content)?;
        
        Ok(config)
    }
    
    /// Get the configuration file path
    fn config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .map_err(|_| CliError::Config("HOME environment variable not set".to_string()))?;
        
        Ok(PathBuf::from(home).join(".peerx").join("config.json"))
    }
    
    /// Merge with another config, preferring non-empty values from other
    fn merge(mut self, other: Config) -> Self {
        if !other.rpc_url.is_empty() {
            self.rpc_url = other.rpc_url;
        }
        if !other.contract_id.is_empty() {
            self.contract_id = other.contract_id;
        }
        if other.admin_address.is_some() {
            self.admin_address = other.admin_address;
        }
        if !other.network.is_empty() {
            self.network = other.network;
        }
        if other.oracle_url.is_some() {
            self.oracle_url = other.oracle_url;
        }
        self.timeout_seconds = other.timeout_seconds;
        self.oracle_max_staleness_seconds = other.oracle_max_staleness_seconds;
        
        self
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.contract_id.is_empty() {
            return Err(CliError::Config(
                "Contract ID is required. Set PEERX_CONTRACT_ID or create config file".to_string()
            ));
        }
        
        if self.rpc_url.is_empty() {
            return Err(CliError::Config("RPC URL cannot be empty".to_string()));
        }
        
        // Validate URL format
        if !self.rpc_url.starts_with("http://") && !self.rpc_url.starts_with("https://") {
            return Err(CliError::InvalidUrl(format!(
                "Invalid RPC URL: {}",
                self.rpc_url
            )));
        }
        
        Ok(())
    }
}
