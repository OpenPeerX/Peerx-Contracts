use crate::config::Config;
use crate::error::{CliError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Health status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// All checks passed
    Healthy,
    /// Some non-critical issues detected
    Warning,
    /// Critical issues detected
    Critical,
}

impl HealthStatus {
    /// Convert to exit code
    pub fn to_exit_code(&self) -> i32 {
        match self {
            HealthStatus::Healthy => 0,
            HealthStatus::Warning => 1,
            HealthStatus::Critical => 2,
        }
    }
    
    /// Get the worst status between two
    pub fn worst(&self, other: &Self) -> Self {
        match (self, other) {
            (HealthStatus::Critical, _) | (_, HealthStatus::Critical) => HealthStatus::Critical,
            (HealthStatus::Warning, _) | (_, HealthStatus::Warning) => HealthStatus::Warning,
            _ => HealthStatus::Healthy,
        }
    }
}

/// Individual health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub checked_at: DateTime<Utc>,
    pub duration_ms: u64,
}

impl HealthCheckResult {
    pub fn new(name: impl Into<String>, status: HealthStatus, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status,
            message: message.into(),
            details: None,
            checked_at: Utc::now(),
            duration_ms: 0,
        }
    }
    
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
    
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}

/// Overall health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub overall_status: HealthStatus,
    pub checks: Vec<HealthCheckResult>,
    pub summary: String,
    pub timestamp: DateTime<Utc>,
    pub total_duration_ms: u64,
}

impl HealthReport {
    pub fn new(checks: Vec<HealthCheckResult>) -> Self {
        let total_duration_ms: u64 = checks.iter().map(|c| c.duration_ms).sum();
        
        // Determine overall status
        let overall_status = checks.iter().fold(HealthStatus::Healthy, |acc, check| {
            acc.worst(&check.status)
        });
        
        // Generate summary
        let healthy_count = checks.iter().filter(|c| c.status == HealthStatus::Healthy).count();
        let warning_count = checks.iter().filter(|c| c.status == HealthStatus::Warning).count();
        let critical_count = checks.iter().filter(|c| c.status == HealthStatus::Critical).count();
        
        let summary = format!(
            "{} checks: {} healthy, {} warnings, {} critical",
            checks.len(),
            healthy_count,
            warning_count,
            critical_count
        );
        
        Self {
            overall_status,
            checks,
            summary,
            timestamp: Utc::now(),
            total_duration_ms,
        }
    }
}

/// Health checker for PeerX contracts
pub struct HealthChecker {
    config: Config,
    client: reqwest::Client,
}

impl HealthChecker {
    pub fn new(config: Config) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { config, client }
    }
    
    /// Run all health checks
    pub async fn run_all_checks(&self) -> Result<HealthReport> {
        let mut checks = Vec::new();
        
        // Run each check
        checks.push(self.check_rpc_endpoint().await);
        checks.push(self.check_contract_exists().await);
        checks.push(self.check_contract_not_paused().await);
        checks.push(self.check_admin_reachable().await);
        checks.push(self.check_oracle_freshness().await);
        
        Ok(HealthReport::new(checks))
    }
    
    /// Check if RPC endpoint is reachable
    async fn check_rpc_endpoint(&self) -> HealthCheckResult {
        let start = std::time::Instant::now();
        let name = "RPC Endpoint";
        
        match self.ping_rpc_endpoint().await {
            Ok(response_time_ms) => {
                let details = serde_json::json!({
                    "url": self.config.rpc_url,
                    "response_time_ms": response_time_ms,
                });
                
                HealthCheckResult::new(
                    name,
                    HealthStatus::Healthy,
                    format!("RPC endpoint is reachable ({}ms)", response_time_ms),
                )
                .with_details(details)
                .with_duration(start.elapsed().as_millis() as u64)
            }
            Err(e) => HealthCheckResult::new(
                name,
                HealthStatus::Critical,
                format!("RPC endpoint unreachable: {}", e),
            )
            .with_duration(start.elapsed().as_millis() as u64),
        }
    }
    
    /// Check if contract exists and is deployed
    async fn check_contract_exists(&self) -> HealthCheckResult {
        let start = std::time::Instant::now();
        let name = "Contract Existence";
        
        match self.query_contract_exists().await {
            Ok(exists) => {
                if exists {
                    let details = serde_json::json!({
                        "contract_id": self.config.contract_id,
                        "network": self.config.network,
                    });
                    
                    HealthCheckResult::new(
                        name,
                        HealthStatus::Healthy,
                        "Contract is deployed and accessible",
                    )
                    .with_details(details)
                    .with_duration(start.elapsed().as_millis() as u64)
                } else {
                    HealthCheckResult::new(
                        name,
                        HealthStatus::Critical,
                        "Contract not found or not deployed",
                    )
                    .with_duration(start.elapsed().as_millis() as u64)
                }
            }
            Err(e) => HealthCheckResult::new(
                name,
                HealthStatus::Critical,
                format!("Failed to check contract existence: {}", e),
            )
            .with_duration(start.elapsed().as_millis() as u64),
        }
    }
    
    /// Check if contract is paused
    async fn check_contract_not_paused(&self) -> HealthCheckResult {
        let start = std::time::Instant::now();
        let name = "Contract Pause Status";
        
        match self.query_contract_paused().await {
            Ok(is_paused) => {
                if is_paused {
                    HealthCheckResult::new(
                        name,
                        HealthStatus::Warning,
                        "Contract is currently paused",
                    )
                    .with_details(serde_json::json!({ "paused": true }))
                    .with_duration(start.elapsed().as_millis() as u64)
                } else {
                    HealthCheckResult::new(
                        name,
                        HealthStatus::Healthy,
                        "Contract is operational (not paused)",
                    )
                    .with_details(serde_json::json!({ "paused": false }))
                    .with_duration(start.elapsed().as_millis() as u64)
                }
            }
            Err(e) => HealthCheckResult::new(
                name,
                HealthStatus::Warning,
                format!("Could not determine pause status: {}", e),
            )
            .with_duration(start.elapsed().as_millis() as u64),
        }
    }
    
    /// Check if admin is reachable (can query admin address)
    async fn check_admin_reachable(&self) -> HealthCheckResult {
        let start = std::time::Instant::now();
        let name = "Admin Reachability";
        
        if self.config.admin_address.is_none() {
            return HealthCheckResult::new(
                name,
                HealthStatus::Warning,
                "Admin address not configured, skipping check",
            )
            .with_duration(start.elapsed().as_millis() as u64);
        }
        
        match self.query_admin_reachable().await {
            Ok(is_reachable) => {
                if is_reachable {
                    let details = serde_json::json!({
                        "admin_address": self.config.admin_address,
                    });
                    
                    HealthCheckResult::new(
                        name,
                        HealthStatus::Healthy,
                        "Admin address is reachable and valid",
                    )
                    .with_details(details)
                    .with_duration(start.elapsed().as_millis() as u64)
                } else {
                    HealthCheckResult::new(
                        name,
                        HealthStatus::Critical,
                        "Admin address is not reachable",
                    )
                    .with_duration(start.elapsed().as_millis() as u64)
                }
            }
            Err(e) => HealthCheckResult::new(
                name,
                HealthStatus::Warning,
                format!("Could not verify admin reachability: {}", e),
            )
            .with_duration(start.elapsed().as_millis() as u64),
        }
    }
    
    /// Check oracle data freshness
    async fn check_oracle_freshness(&self) -> HealthCheckResult {
        let start = std::time::Instant::now();
        let name = "Oracle Freshness";
        
        match self.query_oracle_freshness().await {
            Ok((last_update, staleness_seconds)) => {
                let max_staleness = self.config.oracle_max_staleness_seconds;
                
                let status = if staleness_seconds > max_staleness {
                    HealthStatus::Warning
                } else {
                    HealthStatus::Healthy
                };
                
                let details = serde_json::json!({
                    "last_update": last_update,
                    "staleness_seconds": staleness_seconds,
                    "max_staleness_seconds": max_staleness,
                    "is_fresh": staleness_seconds <= max_staleness,
                });
                
                let message = if status == HealthStatus::Healthy {
                    format!(
                        "Oracle data is fresh ({}s old, max {}s)",
                        staleness_seconds, max_staleness
                    )
                } else {
                    format!(
                        "Oracle data is stale ({}s old, max {}s)",
                        staleness_seconds, max_staleness
                    )
                };
                
                HealthCheckResult::new(name, status, message)
                    .with_details(details)
                    .with_duration(start.elapsed().as_millis() as u64)
            }
            Err(e) => HealthCheckResult::new(
                name,
                HealthStatus::Warning,
                format!("Could not check oracle freshness: {}", e),
            )
            .with_duration(start.elapsed().as_millis() as u64),
        }
    }
    
    // ===== Low-level query methods =====
    
    async fn ping_rpc_endpoint(&self) -> Result<u64> {
        let start = std::time::Instant::now();
        
        // Try to get network info or health endpoint
        let response = self
            .client
            .post(&self.config.rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getHealth",
            }))
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(start.elapsed().as_millis() as u64)
        } else {
            Err(CliError::Network(response.error_for_status().unwrap_err()))
        }
    }
    
    async fn query_contract_exists(&self) -> Result<bool> {
        // Query contract code/instance
        let response = self
            .client
            .post(&self.config.rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getLedgerEntries",
                "params": {
                    "keys": [self.config.contract_id]
                }
            }))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Ok(false);
        }
        
        let body: serde_json::Value = response.json().await?;
        
        // Check if entries exist
        Ok(body.get("result")
            .and_then(|r| r.get("entries"))
            .and_then(|e| e.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false))
    }
    
    async fn query_contract_paused(&self) -> Result<bool> {
        // Try to invoke a read-only method to check pause status
        // This is a mock implementation - adjust based on actual contract interface
        let response = self
            .client
            .post(&self.config.rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "simulateTransaction",
                "params": {
                    "transaction": format!("contract_call:{}:is_paused", self.config.contract_id)
                }
            }))
            .send()
            .await?;
        
        if !response.status().is_success() {
            // If we can't determine, assume not paused (optimistic)
            return Ok(false);
        }
        
        let body: serde_json::Value = response.json().await?;
        
        // Parse the result - this depends on the actual contract interface
        Ok(body.get("result")
            .and_then(|r| r.get("value"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }
    
    async fn query_admin_reachable(&self) -> Result<bool> {
        // Try to query admin address from contract
        let response = self
            .client
            .post(&self.config.rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "simulateTransaction",
                "params": {
                    "transaction": format!("contract_call:{}:get_admin", self.config.contract_id)
                }
            }))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Ok(false);
        }
        
        let body: serde_json::Value = response.json().await?;
        
        // Check if we got a valid response
        let admin_from_contract = body.get("result")
            .and_then(|r| r.get("value"))
            .and_then(|v| v.as_str());
        
        // If admin is configured, verify it matches
        if let Some(expected_admin) = &self.config.admin_address {
            Ok(admin_from_contract == Some(expected_admin.as_str()))
        } else {
            // If no admin configured, just check we got a response
            Ok(admin_from_contract.is_some())
        }
    }
    
    async fn query_oracle_freshness(&self) -> Result<(DateTime<Utc>, u64)> {
        // Query last oracle update time from contract
        let response = self
            .client
            .post(&self.config.rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "simulateTransaction",
                "params": {
                    "transaction": format!("contract_call:{}:get_last_oracle_update", self.config.contract_id)
                }
            }))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(CliError::Contract(
                "Failed to query oracle timestamp".to_string()
            ));
        }
        
        let body: serde_json::Value = response.json().await?;
        
        // Parse timestamp (assuming Unix timestamp in seconds)
        let timestamp_secs = body.get("result")
            .and_then(|r| r.get("value"))
            .and_then(|v| v.as_i64())
            .ok_or_else(|| CliError::Contract("Invalid oracle timestamp".to_string()))?;
        
        let last_update = DateTime::from_timestamp(timestamp_secs, 0)
            .ok_or_else(|| CliError::Contract("Invalid timestamp value".to_string()))?;
        
        let now = Utc::now();
        let staleness = (now - last_update).num_seconds().max(0) as u64;
        
        Ok((last_update, staleness))
    }
}
