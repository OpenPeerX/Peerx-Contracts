use peerx_cli::config::Config;
use peerx_cli::health::{HealthChecker, HealthStatus};

#[tokio::test]
async fn test_health_status_exit_codes() {
    assert_eq!(HealthStatus::Healthy.to_exit_code(), 0);
    assert_eq!(HealthStatus::Warning.to_exit_code(), 1);
    assert_eq!(HealthStatus::Critical.to_exit_code(), 2);
}

#[tokio::test]
async fn test_health_status_worst() {
    assert_eq!(
        HealthStatus::Healthy.worst(&HealthStatus::Healthy),
        HealthStatus::Healthy
    );
    assert_eq!(
        HealthStatus::Healthy.worst(&HealthStatus::Warning),
        HealthStatus::Warning
    );
    assert_eq!(
        HealthStatus::Healthy.worst(&HealthStatus::Critical),
        HealthStatus::Critical
    );
    assert_eq!(
        HealthStatus::Warning.worst(&HealthStatus::Critical),
        HealthStatus::Critical
    );
}

#[test]
fn test_config_validation() {
    let mut config = Config::default();
    
    // Empty contract ID should fail
    assert!(config.validate().is_err());
    
    // Valid config should pass
    config.contract_id = "CDTEST123".to_string();
    config.rpc_url = "https://test.example.com".to_string();
    assert!(config.validate().is_ok());
    
    // Invalid URL should fail
    config.rpc_url = "not-a-url".to_string();
    assert!(config.validate().is_err());
}

#[test]
fn test_config_merge() {
    let mut base = Config {
        rpc_url: "https://base.com".to_string(),
        contract_id: "BASE_CONTRACT".to_string(),
        admin_address: None,
        network: "testnet".to_string(),
        oracle_url: None,
        timeout_seconds: 30,
        oracle_max_staleness_seconds: 300,
    };
    
    let override_config = Config {
        rpc_url: "https://override.com".to_string(),
        contract_id: "OVERRIDE_CONTRACT".to_string(),
        admin_address: Some("ADMIN_ADDR".to_string()),
        network: "mainnet".to_string(),
        oracle_url: Some("https://oracle.com".to_string()),
        timeout_seconds: 60,
        oracle_max_staleness_seconds: 600,
    };
    
    let merged = base.merge(override_config);
    
    assert_eq!(merged.rpc_url, "https://override.com");
    assert_eq!(merged.contract_id, "OVERRIDE_CONTRACT");
    assert_eq!(merged.admin_address, Some("ADMIN_ADDR".to_string()));
    assert_eq!(merged.network, "mainnet");
    assert_eq!(merged.timeout_seconds, 60);
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    // These tests would require a mock Soroban RPC server
    // For now, they're placeholders for future implementation
    
    #[tokio::test]
    #[ignore] // Requires mock server
    async fn test_health_checker_all_checks() {
        let config = Config {
            rpc_url: "http://localhost:8000".to_string(),
            contract_id: "TEST_CONTRACT".to_string(),
            admin_address: Some("TEST_ADMIN".to_string()),
            network: "local".to_string(),
            oracle_url: None,
            timeout_seconds: 5,
            oracle_max_staleness_seconds: 300,
        };
        
        let checker = HealthChecker::new(config);
        let report = checker.run_all_checks().await;
        
        // With a proper mock, we would assert:
        // assert!(report.is_ok());
        // assert_eq!(report.unwrap().checks.len(), 5);
    }
}
