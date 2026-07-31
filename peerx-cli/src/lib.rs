// Library exports for testing
pub mod commands;
pub mod config;
pub mod error;
pub mod health;
pub mod output;

// Re-export commonly used types
pub use config::Config;
pub use error::{CliError, Result};
pub use health::{HealthChecker, HealthReport, HealthStatus, HealthCheckResult};
