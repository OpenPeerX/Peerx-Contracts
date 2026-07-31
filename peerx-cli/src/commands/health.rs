use crate::config::Config;
use crate::error::Result;
use crate::health::{HealthChecker, HealthStatus};
use crate::output::OutputFormatter;
use clap::Args;
use colored::*;

#[derive(Args, Debug)]
pub struct HealthCommand {
    /// RPC endpoint URL (overrides config/env)
    #[arg(long, env = "PEERX_RPC_URL")]
    rpc_url: Option<String>,

    /// Contract ID (overrides config/env)
    #[arg(long, env = "PEERX_CONTRACT_ID")]
    contract_id: Option<String>,

    /// Admin address for verification (overrides config/env)
    #[arg(long, env = "PEERX_ADMIN_ADDRESS")]
    admin_address: Option<String>,

    /// Network name (testnet, mainnet, local)
    #[arg(long, env = "PEERX_NETWORK", default_value = "testnet")]
    network: String,

    /// Maximum acceptable oracle staleness in seconds
    #[arg(long, default_value = "300")]
    max_oracle_staleness: u64,

    /// Timeout for each check in seconds
    #[arg(long, default_value = "30")]
    timeout: u64,

    /// Only run specific checks (comma-separated: rpc,contract,pause,admin,oracle)
    #[arg(long)]
    checks: Option<String>,

    /// Fail fast on first critical error
    #[arg(long)]
    fail_fast: bool,
}

impl HealthCommand {
    pub async fn execute(&self, format: &str) -> Result<i32> {
        // Load base configuration
        let mut config = Config::load().unwrap_or_default();

        // Apply command-line overrides
        if let Some(rpc_url) = &self.rpc_url {
            config.rpc_url = rpc_url.clone();
        }
        if let Some(contract_id) = &self.contract_id {
            config.contract_id = contract_id.clone();
        }
        if let Some(admin_address) = &self.admin_address {
            config.admin_address = Some(admin_address.clone());
        }
        config.network = self.network.clone();
        config.oracle_max_staleness_seconds = self.max_oracle_staleness;
        config.timeout_seconds = self.timeout;

        // Validate configuration
        config.validate()?;

        // Print header in human format
        if format == "human" {
            self.print_header(&config);
        }

        // Create health checker
        let checker = HealthChecker::new(config);

        // Run health checks
        let report = checker.run_all_checks().await?;

        // Format and print output
        match format {
            "json" => {
                println!("{}", OutputFormatter::format(&report, "json")?);
            }
            "yaml" => {
                println!("{}", OutputFormatter::format(&report, "yaml")?);
            }
            "human" | _ => {
                self.print_human_report(&report);
            }
        }

        // Return appropriate exit code
        Ok(report.overall_status.to_exit_code())
    }

    fn print_header(&self, config: &Config) {
        println!("{}", "━".repeat(80).dimmed());
        println!(
            "{} {}",
            "PeerX Health Check".bold(),
            format!("(Network: {})", config.network).dimmed()
        );
        println!("{}", "━".repeat(80).dimmed());
        println!(
            "{} {}",
            "RPC:".bold(),
            config.rpc_url.dimmed()
        );
        println!(
            "{} {}",
            "Contract:".bold(),
            config.contract_id.dimmed()
        );
        println!("{}", "━".repeat(80).dimmed());
        println!();
    }

    fn print_human_report(&self, report: &crate::health::HealthReport) {
        // Print individual checks
        for check in &report.checks {
            let icon = match check.status {
                HealthStatus::Healthy => "✓".green(),
                HealthStatus::Warning => "⚠".yellow(),
                HealthStatus::Critical => "✗".red(),
            };

            let status_text = match check.status {
                HealthStatus::Healthy => "PASS".green(),
                HealthStatus::Warning => "WARN".yellow(),
                HealthStatus::Critical => "FAIL".red(),
            };

            println!(
                "{} {} {} {}",
                icon,
                format!("[{}]", status_text).bold(),
                check.name.bold(),
                format!("({}ms)", check.duration_ms).dimmed()
            );
            println!("  {}", check.message);

            // Print details if present
            if let Some(details) = &check.details {
                match details.as_object() {
                    Some(obj) => {
                        for (key, value) in obj {
                            println!(
                                "    {} {}",
                                format!("{}:", key).dimmed(),
                                self.format_detail_value(value)
                            );
                        }
                    }
                    None => {}
                }
            }
            println!();
        }

        // Print summary
        println!("{}", "━".repeat(80).dimmed());
        
        let summary_icon = match report.overall_status {
            HealthStatus::Healthy => "✓".green(),
            HealthStatus::Warning => "⚠".yellow(),
            HealthStatus::Critical => "✗".red(),
        };

        let summary_text = match report.overall_status {
            HealthStatus::Healthy => "HEALTHY".green().bold(),
            HealthStatus::Warning => "WARNING".yellow().bold(),
            HealthStatus::Critical => "CRITICAL".red().bold(),
        };

        println!(
            "{} Overall Status: {}",
            summary_icon,
            summary_text
        );
        println!("  {}", report.summary);
        println!(
            "  Total Duration: {}ms",
            report.total_duration_ms
        );
        println!("{}", "━".repeat(80).dimmed());

        // Print exit code hint
        let exit_code = report.overall_status.to_exit_code();
        println!(
            "\n{} Exit code: {}",
            "Exit Code:".dimmed(),
            match exit_code {
                0 => exit_code.to_string().green(),
                1 => exit_code.to_string().yellow(),
                _ => exit_code.to_string().red(),
            }
        );
    }

    fn format_detail_value(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Bool(b) => {
                if *b {
                    "true".green().to_string()
                } else {
                    "false".red().to_string()
                }
            }
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            _ => value.to_string(),
        }
    }
}
