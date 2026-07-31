mod commands;
mod config;
mod error;
mod health;
mod output;

use clap::{Parser, Subcommand};
use commands::health::HealthCommand;
use error::Result;

/// PeerX CLI - Command-line interface for PeerX Contracts
#[derive(Parser)]
#[command(
    name = "peerx",
    version,
    about = "PeerX CLI - Operational tools for PeerX Contracts",
    long_about = "Command-line interface for managing and monitoring PeerX smart contracts on Soroban.\
                  \n\nProvides health checks, contract interaction, and operational utilities."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format: json, yaml, or human (default)
    #[arg(short, long, global = true, default_value = "human")]
    format: String,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run pre-flight health checks on PeerX contract and infrastructure
    #[command(alias = "check")]
    Health(HealthCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging based on verbosity
    setup_logging(cli.verbose, cli.quiet);

    // Execute the command
    let exit_code = match cli.command {
        Commands::Health(cmd) => cmd.execute(&cli.format).await?,
    };

    std::process::exit(exit_code);
}

fn setup_logging(verbose: bool, quiet: bool) {
    if quiet {
        // Only show errors
        std::env::set_var("RUST_LOG", "error");
    } else if verbose {
        // Show debug logs
        std::env::set_var("RUST_LOG", "debug");
    } else {
        // Default: show info and above
        std::env::set_var("RUST_LOG", "info");
    }
}
