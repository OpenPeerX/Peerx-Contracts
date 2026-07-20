# PeerX CLI

> Command-line interface for PeerX Contracts with pre-flight health checks and operational tools.

[![Rust](https://img.shields.io/badge/Rust-1.74%2B-orange?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](#license)

## Overview

The PeerX CLI (`peerx`) is a command-line tool for interacting with and monitoring PeerX smart contracts on Soroban. It provides comprehensive health checks, operational utilities, and contract interaction capabilities.

## Features

- **🏥 Health Checks**: Pre-flight health checks with structured output and exit codes
  - RPC endpoint reachability
  - Contract deployment verification
  - Contract pause status monitoring
  - Admin address verification
  - Oracle data freshness checks
- **📊 Multiple Output Formats**: JSON, YAML, and human-readable output
- **🔧 Configuration**: Flexible configuration via environment variables, config files, or CLI arguments
- **⚡ Fast & Reliable**: Built with Rust for performance and reliability

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/coderolisa/Peerx-Contracts.git
cd Peerx-Contracts/peerx-cli

# Build the CLI
cargo build --release

# The binary will be at target/release/peerx
# Optionally, install it to your PATH
cargo install --path .
```

### Prerequisites

- Rust 1.74 or higher
- Cargo package manager
- Access to a Soroban RPC endpoint

## Quick Start

### Basic Health Check

```bash
# Set required environment variables
export PEERX_CONTRACT_ID="CDXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
export PEERX_RPC_URL="https://soroban-testnet.stellar.org"

# Run health checks
peerx health
```

### Expected Output

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PeerX Health Check (Network: testnet)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
RPC: https://soroban-testnet.stellar.org
Contract: CDXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✓ [PASS] RPC Endpoint (145ms)
  RPC endpoint is reachable (145ms)
    url: https://soroban-testnet.stellar.org
    response_time_ms: 145

✓ [PASS] Contract Existence (234ms)
  Contract is deployed and accessible
    contract_id: CDXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
    network: testnet

✓ [PASS] Contract Pause Status (156ms)
  Contract is operational (not paused)
    paused: false

⚠ [WARN] Admin Reachability (89ms)
  Admin address not configured, skipping check

✓ [PASS] Oracle Freshness (123ms)
  Oracle data is fresh (45s old, max 300s)
    last_update: 2026-07-20T10:15:30Z
    staleness_seconds: 45
    max_staleness_seconds: 300
    is_fresh: true

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✓ Overall Status: HEALTHY
  5 checks: 4 healthy, 1 warnings, 0 critical
  Total Duration: 747ms
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Exit Code: 0
```

## Usage

### Health Command

The `health` subcommand performs comprehensive pre-flight checks on your PeerX contract deployment.

```bash
peerx health [OPTIONS]
```

#### Exit Codes

The health command uses specific exit codes to indicate the overall system status:

- **0** - All checks passed (Healthy)
- **1** - Some non-critical issues detected (Warning)
- **2** - Critical issues detected (Critical)

This makes it ideal for use in CI/CD pipelines and monitoring scripts.

#### Options

| Option | Environment Variable | Default | Description |
|--------|---------------------|---------|-------------|
| `--rpc-url <URL>` | `PEERX_RPC_URL` | `https://soroban-testnet.stellar.org` | Soroban RPC endpoint URL |
| `--contract-id <ID>` | `PEERX_CONTRACT_ID` | (required) | Contract ID to check |
| `--admin-address <ADDR>` | `PEERX_ADMIN_ADDRESS` | None | Admin address for verification |
| `--network <NAME>` | `PEERX_NETWORK` | `testnet` | Network name (testnet/mainnet/local) |
| `--max-oracle-staleness <SEC>` | - | `300` | Max acceptable oracle staleness (seconds) |
| `--timeout <SEC>` | - | `30` | Timeout for each check (seconds) |
| `--format <FORMAT>` | - | `human` | Output format (json/yaml/human) |
| `--verbose` | - | false | Enable verbose output |
| `--quiet` | - | false | Suppress all output except errors |

#### Health Checks Performed

1. **RPC Endpoint** - Verifies the Soroban RPC endpoint is reachable and responsive
2. **Contract Existence** - Confirms the contract is deployed and accessible
3. **Contract Pause Status** - Checks if the contract is paused (operations halted)
4. **Admin Reachability** - Verifies admin address is configured and valid
5. **Oracle Freshness** - Ensures oracle data is up-to-date

### Examples

#### Check with custom timeout and staleness

```bash
peerx health \
  --timeout 60 \
  --max-oracle-staleness 600
```

#### Output as JSON for scripting

```bash
peerx health --format json
```

Example JSON output:

```json
{
  "overall_status": "healthy",
  "checks": [
    {
      "name": "RPC Endpoint",
      "status": "healthy",
      "message": "RPC endpoint is reachable (145ms)",
      "details": {
        "url": "https://soroban-testnet.stellar.org",
        "response_time_ms": 145
      },
      "checked_at": "2026-07-20T10:16:15.123456Z",
      "duration_ms": 145
    }
  ],
  "summary": "5 checks: 4 healthy, 1 warnings, 0 critical",
  "timestamp": "2026-07-20T10:16:15.987654Z",
  "total_duration_ms": 747
}
```

#### Use in CI/CD pipeline

```bash
#!/bin/bash

# Run health check
peerx health --format json > health-report.json

# Check exit code
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
  echo "✓ All health checks passed"
elif [ $EXIT_CODE -eq 1 ]; then
  echo "⚠ Health check warnings detected"
  exit 1
else
  echo "✗ Critical health check failures"
  exit 2
fi
```

#### Mainnet monitoring

```bash
peerx health \
  --network mainnet \
  --rpc-url https://soroban-mainnet.stellar.org \
  --contract-id CDMAINNETCONTRACTID... \
  --admin-address GADMIN... \
  --format json \
  | jq '.overall_status'
```

## Configuration

The CLI supports multiple configuration methods, with the following precedence (highest to lowest):

1. **Command-line arguments**
2. **Environment variables**
3. **Configuration file** (`~/.peerx/config.json`)
4. **Default values**

### Configuration File

Create a configuration file at `~/.peerx/config.json`:

```json
{
  "rpc_url": "https://soroban-testnet.stellar.org",
  "contract_id": "CDXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
  "admin_address": "GADMIN...",
  "network": "testnet",
  "oracle_url": null,
  "timeout_seconds": 30,
  "oracle_max_staleness_seconds": 300
}
```

### Environment Variables

```bash
export PEERX_RPC_URL="https://soroban-testnet.stellar.org"
export PEERX_CONTRACT_ID="CDXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
export PEERX_ADMIN_ADDRESS="GADMIN..."
export PEERX_NETWORK="testnet"
export PEERX_TIMEOUT_SECONDS="30"
export PEERX_ORACLE_MAX_STALENESS_SECONDS="300"
```

You can add these to your `.bashrc`, `.zshrc`, or `.env` file for persistence.

## Use Cases

### Pre-deployment Verification

Before deploying updates to your PeerX contract:

```bash
# Verify the current deployment is healthy
peerx health --format json | jq -e '.overall_status == "healthy"' || exit 1
```

### Monitoring & Alerting

Set up continuous monitoring with cron:

```bash
# Add to crontab (check every 5 minutes)
*/5 * * * * /usr/local/bin/peerx health --format json > /var/log/peerx/health-$(date +\%Y\%m\%d-\%H\%M).json

# Alert on failures
*/5 * * * * /usr/local/bin/peerx health || /usr/local/bin/send-alert "PeerX health check failed"
```

### Kubernetes Liveness/Readiness Probes

Use as a readiness probe in Kubernetes:

```yaml
readinessProbe:
  exec:
    command:
    - peerx
    - health
    - --format
    - json
  initialDelaySeconds: 10
  periodSeconds: 30
  timeoutSeconds: 10
  successThreshold: 1
  failureThreshold: 3
```

### Incident Response

During an incident, quickly assess system status:

```bash
# Get detailed status
peerx health --verbose

# Check specific component
peerx health --checks oracle --format json | jq '.checks[0]'
```

## Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/coderolisa/Peerx-Contracts.git
cd Peerx-Contracts/peerx-cli

# Build
cargo build

# Run tests
cargo test

# Build optimized release
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_health_checker
```

### Project Structure

```
peerx-cli/
├── Cargo.toml              # Project manifest
├── README.md               # This file
├── CHANGELOG.md            # Version history
└── src/
    ├── main.rs             # CLI entry point
    ├── commands/           # Command implementations
    │   ├── mod.rs
    │   └── health.rs       # Health check command
    ├── config.rs           # Configuration management
    ├── error.rs            # Error types
    ├── health.rs           # Health check logic
    └── output.rs           # Output formatting
```

## Troubleshooting

### "Contract ID is required" error

Make sure you've set the `PEERX_CONTRACT_ID` environment variable or passed it via `--contract-id`:

```bash
export PEERX_CONTRACT_ID="CDXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
```

### Connection timeout errors

Increase the timeout if your network is slow:

```bash
peerx health --timeout 60
```

### "RPC endpoint unreachable" error

Verify your RPC URL is correct and accessible:

```bash
curl -X POST https://soroban-testnet.stellar.org \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'
```

### Oracle staleness warnings

If oracle data is frequently stale, either:
1. Investigate why oracle updates are delayed
2. Increase the acceptable staleness threshold:

```bash
peerx health --max-oracle-staleness 600  # 10 minutes
```

## Roadmap

Future enhancements planned:

- [ ] Additional commands (deploy, invoke, query)
- [ ] Interactive mode
- [ ] Watch mode for continuous monitoring
- [ ] Historical health data tracking
- [ ] Integration with Prometheus/Grafana
- [ ] Support for multiple contracts
- [ ] Custom health check plugins

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run `cargo fmt` and `cargo clippy`
6. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/coderolisa/Peerx-Contracts/issues)
- **Discussions**: [GitHub Discussions](https://github.com/coderolisa/Peerx-Contracts/discussions)
- **Security**: Report security issues to security@peerx.io

---

**PeerX CLI** - Operational excellence for PeerX Contracts 🚀
