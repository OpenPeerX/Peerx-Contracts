# Add PeerX CLI with Health Check Subcommand

## 🎯 Problem Statement

The PeerX platform lacked pre-flight health checks for monitoring contract and infrastructure status. Operators had no automated way to verify:
- RPC endpoint reachability
- Contract deployment status  
- Contract pause state
- Admin address validity
- Oracle data freshness

This gap made it difficult to ensure system reliability before deployments and during operations.

## ✅ Solution

Implemented a comprehensive `peerx` CLI tool with a `health` subcommand that provides:

### Core Features
- **🏥 Comprehensive Health Checks**: 5 critical checks covering all key components
- **📊 Structured Output**: Support for JSON, YAML, and human-readable formats
- **🚦 Smart Exit Codes**: 0 (healthy), 1 (warning), 2 (critical) for automation
- **⚙️ Flexible Configuration**: Via CLI args, environment variables, or config file
- **⚡ Fast & Reliable**: Built in Rust with async/await for performance
- **📝 Extensive Documentation**: Complete README with examples and troubleshooting

### Health Checks Performed

1. **RPC Endpoint Reachability** - Verifies Soroban RPC is accessible and responsive
2. **Contract Existence** - Confirms contract is deployed on the network
3. **Contract Pause Status** - Checks if operations are halted
4. **Admin Reachability** - Validates admin address configuration
5. **Oracle Freshness** - Ensures oracle data is current (configurable staleness threshold)

Each check provides:
- Pass/Warn/Fail status
- Detailed error messages
- Execution timing (milliseconds)
- Structured metadata for debugging

## 📁 Files Added

### Core Implementation
- `peerx-cli/Cargo.toml` - Project manifest with dependencies
- `peerx-cli/src/main.rs` - CLI entry point and command routing
- `peerx-cli/src/lib.rs` - Library exports for testing
- `peerx-cli/src/commands/mod.rs` - Command module exports
- `peerx-cli/src/commands/health.rs` - Health check command implementation
- `peerx-cli/src/config.rs` - Configuration management (env vars, files, CLI args)
- `peerx-cli/src/error.rs` - Error types and Result wrapper
- `peerx-cli/src/health.rs` - Health check logic and RPC queries
- `peerx-cli/src/output.rs` - Output formatting (JSON/YAML/human)

### Documentation
- `peerx-cli/README.md` - Comprehensive usage guide with examples
- `peerx-cli/CHANGELOG.md` - Version history and feature documentation

### Tests
- `peerx-cli/tests/health_tests.rs` - Unit and integration tests

### Examples
- `peerx-cli/examples/basic_health_check.sh` - Simple health check script
- `peerx-cli/examples/ci_integration.sh` - CI/CD pipeline integration
- `peerx-cli/examples/monitoring_cron.sh` - Continuous monitoring with alerting

## 📈 Usage Examples

### Basic Health Check
```bash
export PEERX_CONTRACT_ID="CDXXXX..."
export PEERX_RPC_URL="https://soroban-testnet.stellar.org"

peerx health
```

### CI/CD Integration
```bash
# JSON output with exit code handling
peerx health --format json > health-report.json
if [ $? -eq 0 ]; then
  echo "✓ Deployment approved"
else
  echo "✗ Deployment blocked"
  exit 1
fi
```

### Custom Thresholds
```bash
peerx health \
  --timeout 60 \
  --max-oracle-staleness 600 \
  --format json
```

## 🎨 Output Preview

### Human-Readable Output
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PeerX Health Check (Network: testnet)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
RPC: https://soroban-testnet.stellar.org
Contract: CDXXXX...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✓ [PASS] RPC Endpoint (145ms)
  RPC endpoint is reachable (145ms)

✓ [PASS] Contract Existence (234ms)
  Contract is deployed and accessible

✓ [PASS] Contract Pause Status (156ms)
  Contract is operational (not paused)

⚠ [WARN] Admin Reachability (89ms)
  Admin address not configured, skipping check

✓ [PASS] Oracle Freshness (123ms)
  Oracle data is fresh (45s old, max 300s)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✓ Overall Status: HEALTHY
  5 checks: 4 healthy, 1 warnings, 0 critical
  Total Duration: 747ms
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Exit Code: 0
```

### JSON Output
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

## 🔧 Configuration

The CLI supports three configuration methods (in order of precedence):

1. **Command-line arguments** - Highest priority
2. **Environment variables** - `PEERX_*` prefix
3. **Config file** - `~/.peerx/config.json`

### Environment Variables
```bash
export PEERX_RPC_URL="https://soroban-testnet.stellar.org"
export PEERX_CONTRACT_ID="CDXXXX..."
export PEERX_ADMIN_ADDRESS="GXXXX..."
export PEERX_NETWORK="testnet"
export PEERX_TIMEOUT_SECONDS="30"
export PEERX_ORACLE_MAX_STALENESS_SECONDS="300"
```

## ✅ Acceptance Criteria Met

- [x] `peerx health` subcommand implemented
- [x] Exit codes: 0 (healthy), 1 (warning), 2 (critical)
- [x] Structured output (JSON, YAML, human-readable)
- [x] Admin reachability check
- [x] Contract paused status check
- [x] Oracle freshness check
- [x] Comprehensive documentation in README.md
- [x] Usage examples and troubleshooting guide
- [x] CI/CD integration examples
- [x] Unit tests

## 🏷️ Labels

- `observability` - Provides system visibility and monitoring
- `dx` - Improves developer experience with CLI tooling
- `🥈` - Medium difficulty
- `S` - Small effort (well-scoped feature)

## 🧪 Testing

### Build & Test
```bash
# Build the CLI
cargo build --release --manifest-path peerx-cli/Cargo.toml

# Run tests
cargo test --manifest-path peerx-cli/Cargo.toml

# Test the CLI
./target/release/peerx health --help
```

### Manual Testing
```bash
# Set up test environment
export PEERX_CONTRACT_ID="CDTEST123"
export PEERX_RPC_URL="https://soroban-testnet.stellar.org"

# Run health check
./target/release/peerx health

# Test JSON output
./target/release/peerx health --format json

# Test with custom timeout
./target/release/peerx health --timeout 60
```

## 📚 Documentation

The implementation includes comprehensive documentation:

- **README.md**: Complete usage guide with examples, configuration, troubleshooting
- **CHANGELOG.md**: Version history and feature list
- **Inline code comments**: Explaining complex logic and RPC interactions
- **Example scripts**: Real-world usage patterns for CI/CD and monitoring
- **Help text**: Built-in CLI help via `--help` flags

## 🚀 Future Enhancements

Potential additions (out of scope for this PR):
- Additional commands: `deploy`, `invoke`, `query`
- Watch mode for continuous monitoring
- Historical health data tracking
- Prometheus/Grafana integration
- Multi-contract support
- Custom health check plugins

## 🔍 Implementation Notes

### Architecture
- **Modular design**: Separate modules for config, health checks, output formatting
- **Async/await**: Non-blocking RPC queries for performance
- **Error handling**: Comprehensive error types with context
- **Testable**: Library exports allow for unit testing

### Design Decisions
- **Exit codes**: Standard Unix convention (0=success, 1=warning, 2=critical)
- **Output formats**: JSON for automation, human-readable for operators
- **Configuration hierarchy**: CLI args > env vars > config file > defaults
- **Fail-safe defaults**: Reasonable timeouts and thresholds out of the box

### RPC Integration
The health checks query Soroban RPC using standard JSON-RPC methods:
- `getHealth` - RPC endpoint availability
- `getLedgerEntries` - Contract existence
- `simulateTransaction` - Contract state queries (pause status, admin, oracle)

## 📝 Checklist

- [x] Code compiles without errors
- [x] All tests pass
- [x] Documentation complete
- [x] Examples provided
- [x] Exit codes work correctly
- [x] JSON output is valid
- [x] Human-readable output is clear
- [x] Configuration system works
- [x] Error messages are helpful
- [x] Code follows Rust conventions

## 🤝 Review Notes

This PR is ready for review. Key areas to focus on:

1. **Health check logic** - Verify RPC query patterns match contract interface
2. **Output formatting** - Ensure JSON structure meets requirements
3. **Error handling** - Check edge cases are handled gracefully
4. **Documentation** - Confirm examples are clear and accurate
5. **Configuration** - Validate precedence and defaults make sense

## 🙏 Acknowledgments

Implements issue requirements for observable, operator-friendly pre-flight health checks. Built with a focus on developer experience (DX) and production readiness.

---

**Ready for merge after review** ✅
