# PeerX CLI Usage Guide

Comprehensive guide for using the PeerX CLI health check system.

## Table of Contents

- [Installation](#installation)
- [Configuration](#configuration)
- [Health Check Command](#health-check-command)
- [Exit Codes](#exit-codes)
- [Check Types](#check-types)
- [Output Formats](#output-formats)
- [Automation & CI/CD](#automation--cicd)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Installation

### Build from Source

```bash
cd peerx-cli
cargo build --release

# Binary will be at: target/release/peerx
```

### Install Globally

```bash
cargo install --path peerx-cli
```

### Verify Installation

```bash
peerx --version
peerx --help
```

## Configuration

### Quick Setup

1. **Copy example config:**
   ```bash
   cp peerx-cli/peerx-cli.json.example peerx-cli.json
   ```

2. **Edit with your values:**
   ```json
   {
     "contract": {
       "contract_id": "YOUR_CONTRACT_ID_HERE"
     }
   }
   ```

3. **Or use environment variables:**
   ```bash
   export PEERX_CONTRACT_ID="CXXXXX..."
   export PEERX_NETWORK="testnet"
   ```

### Configuration Methods

#### 1. Environment Variables (Recommended for CI/CD)

```bash
# Required
export PEERX_CONTRACT_ID="CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"

# Optional
export PEERX_NETWORK="testnet"
export PEERX_RPC_URL="https://soroban-testnet.stellar.org"
export PEERX_ADMIN_ADDRESS="GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
```

#### 2. Configuration File (Recommended for Development)

**Location options (in order of precedence):**
- `./peerx-cli.json` (current directory)
- `./.peerx-cli.json` (hidden file in current directory)
- `~/.config/peerx/config.json` (user home directory)

**Full example:**
```json
{
  "network": {
    "name": "testnet",
    "rpc_url": "https://soroban-testnet.stellar.org",
    "horizon_url": "https://horizon-testnet.stellar.org",
    "passphrase": "Test SDF Network ; September 2015"
  },
  "contract": {
    "contract_id": "CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    "admin_address": "GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
  },
  "health": {
    "timeout_seconds": 30,
    "oracle_freshness_threshold_seconds": 300,
    "max_retries": 3
  }
}
```

#### 3. Command-Line Arguments

```bash
peerx health \
  --contract-id CXXXXX... \
  --network testnet \
  --rpc-url https://soroban-testnet.stellar.org
```

### Configuration Precedence

Settings are applied in this order (later overrides earlier):
1. Default values
2. Config file
3. Environment variables
4. Command-line arguments

## Health Check Command

### Basic Usage

```bash
# Simple check
peerx health

# With specific contract
peerx health --contract-id CXXXXX...

# Show all details
peerx health --details

# JSON output
peerx health --format json
```

### All Options

```
peerx health [OPTIONS]

OPTIONS:
  -n, --network <NETWORK>
          Network to check: testnet, mainnet, or local
          [env: PEERX_NETWORK=]

  -c, --contract-id <CONTRACT_ID>
          Contract ID to check (56-character string starting with 'C')
          [env: PEERX_CONTRACT_ID=]

  -r, --rpc-url <RPC_URL>
          Soroban RPC endpoint URL
          [env: PEERX_RPC_URL=]

  -a, --admin-address <ADDRESS>
          Admin address to verify (56-character Stellar address)
          [env: PEERX_ADMIN_ADDRESS=]

  -d, --details
          Show detailed information for each check including metadata

  --only <CHECKS>
          Run only specific checks (comma-separated list)
          Available: rpc_reachable, horizon_reachable, admin_reachable,
                     contract_exists, contract_not_paused, oracle_fresh

  -f, --format <FORMAT>
          Output format [default: human]
          [possible values: human, json, yaml]

  -v, --verbose
          Enable verbose logging output

  -q, --quiet
          Suppress all output except errors

  -h, --help
          Print help information
```

## Exit Codes

The CLI follows standard Unix conventions:

| Code | Status | Meaning | Action |
|------|--------|---------|--------|
| `0` | ✅ Healthy | All checks passed | Proceed with operations |
| `1` | ⚠️ Degraded | Warnings present, no critical failures | Review warnings, may proceed |
| `2` | ❌ Unhealthy | Critical failures detected | **DO NOT proceed** - investigate immediately |

### Exit Code Usage in Scripts

```bash
#!/bin/bash

peerx health --quiet
EXIT_CODE=$?

case $EXIT_CODE in
  0)
    echo "System healthy - deploying..."
    # proceed with deployment
    ;;
  1)
    echo "System degraded - reviewing..."
    # notify team but may proceed
    ;;
  2)
    echo "System unhealthy - aborting!"
    exit 1
    ;;
esac
```

## Check Types

### 1. RPC Reachable ⚠️ CRITICAL

**What it checks:** Soroban RPC endpoint is accessible and responding

**Severity:** Critical - Operations cannot proceed without RPC

**Pass criteria:**
- HTTP 200 response from `/health` endpoint
- Response time < timeout threshold

**Failure reasons:**
- Network connectivity issues
- RPC endpoint down or misconfigured
- DNS resolution failure

**Remediation:**
```bash
# Test manually
curl https://soroban-testnet.stellar.org/health

# Check network
ping soroban-testnet.stellar.org

# Try alternative endpoint
peerx health --rpc-url https://alternative-rpc.stellar.org
```

### 2. Horizon Reachable ⚠️ WARNING

**What it checks:** Horizon API is accessible for account queries

**Severity:** Warning - Some features may be limited

**Pass criteria:**
- HTTP 200 response from Horizon root endpoint

**Failure reasons:**
- Horizon API temporarily unavailable
- Network issues
- Rate limiting

**Remediation:**
- Wait and retry
- Check Stellar status page
- Operations can continue with limited monitoring

### 3. Admin Reachable ⚠️ VARIABLE

**What it checks:** Admin account exists and is accessible on-chain

**Severity:**
- **Critical** if admin account not found (404)
- **Warning** if query fails for other reasons
- **Pass** if admin not configured (skip)

**Pass criteria:**
- Admin account exists on-chain
- Account data is retrievable via Horizon

**Failure reasons:**
- Admin account doesn't exist (not funded)
- Incorrect admin address
- Horizon API issues

**Remediation:**
```bash
# Verify admin address format
echo $PEERX_ADMIN_ADDRESS

# Check account on Stellar Explorer
# https://stellar.expert/explorer/testnet/account/GXXXXX...

# Fund account if needed on testnet
# https://laboratory.stellar.org/#account-creator?network=test
```

### 4. Contract Exists ⚠️ CRITICAL

**What it checks:** Contract ID is valid and contract is deployed

**Severity:** Critical - Cannot interact with invalid contract

**Pass criteria:**
- Contract ID is 56 characters
- Starts with 'C'
- Valid format

**Future enhancement:** Will query contract state via RPC

**Failure reasons:**
- Invalid contract ID format
- Wrong contract ID
- Contract not deployed

**Remediation:**
```bash
# Verify contract ID
soroban contract inspect --id $PEERX_CONTRACT_ID --network testnet

# Re-deploy if needed
soroban contract deploy --wasm contract.wasm --network testnet
```

### 5. Contract Not Paused ⚠️ CRITICAL

**What it checks:** Contract operations are not emergency-paused

**Severity:** Critical - No trading when paused

**Pass criteria:**
- Contract pause state is `false`

**Failure reasons:**
- Admin executed emergency pause
- Circuit breaker triggered
- Maintenance mode

**Remediation:**
```bash
# Check pause state
soroban contract invoke \
  --id $PEERX_CONTRACT_ID \
  --network testnet \
  -- is_paused

# Resume (admin only)
soroban contract invoke \
  --id $PEERX_CONTRACT_ID \
  --network testnet \
  --source ADMIN_SECRET \
  -- emergency_unpause
```

### 6. Oracle Fresh ⚠️ WARNING

**What it checks:** Oracle price data is recent and trustworthy

**Severity:** Warning - Stale prices may cause issues

**Pass criteria:**
- Last oracle update < threshold (default: 5 minutes)

**Failure reasons:**
- Oracle feeder offline
- Network congestion
- Oracle contract issues

**Remediation:**
```bash
# Check last update time
soroban contract invoke \
  --id $PEERX_CONTRACT_ID \
  --network testnet \
  -- get_last_oracle_update

# Restart oracle feeder
# Contact oracle operator

# Adjust threshold if needed
export PEERX_ORACLE_FRESHNESS_THRESHOLD=600  # 10 minutes
```

## Output Formats

### Human-Readable (Default)

Best for: Interactive use, manual checks

```bash
peerx health
```

**Output:**
```
PeerX Health Check
==================================================
Network: testnet
Contract: CABCD...

Check Results:
--------------------------------------------------
✓ rpc_reachable................... (45ms)
✓ horizon_reachable............... (67ms)
✓ admin_reachable................. (89ms)
✓ contract_exists................. (12ms)
✓ contract_not_paused............. (34ms)
✓ oracle_fresh.................... (23ms)

Summary:
--------------------------------------------------
  Total checks: 6
  ✓ Passed: 6

--------------------------------------------------
✓ All checks passed - System is healthy

ℹ Completed in 270ms
ℹ Exit code: 0
```

### JSON

Best for: Automation, parsing, logging

```bash
peerx health --format json
```

**Output:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:45.123Z",
  "checks": [
    {
      "name": "rpc_reachable",
      "status": "pass",
      "message": "RPC endpoint is reachable at https://soroban-testnet.stellar.org",
      "duration_ms": 45,
      "details": {
        "url": "https://soroban-testnet.stellar.org",
        "status_code": 200,
        "response_time_ms": 45
      }
    }
  ],
  "total_duration_ms": 270,
  "summary": {
    "total": 6,
    "passed": 6,
    "warnings": 0,
    "failed": 0,
    "errors": 0
  }
}
```

### With Details

```bash
peerx health --details
```

Shows additional metadata for each check including URLs, response codes, and diagnostic information.

## Automation & CI/CD

### GitHub Actions

```yaml
name: PeerX Health Check

on:
  schedule:
    - cron: '*/15 * * * *'  # Every 15 minutes
  push:
    branches: [main]
  workflow_dispatch:

jobs:
  health-check:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          profile: minimal

      - name: Build CLI
        run: |
          cd peerx-cli
          cargo build --release

      - name: Run Health Check
        env:
          PEERX_CONTRACT_ID: ${{ secrets.PEERX_CONTRACT_ID }}
          PEERX_ADMIN_ADDRESS: ${{ secrets.PEERX_ADMIN_ADDRESS }}
          PEERX_NETWORK: testnet
        run: |
          ./peerx-cli/target/release/peerx health --format json | tee health-report.json

      - name: Upload Report
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: health-report
          path: health-report.json

      - name: Notify on Failure
        if: failure()
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: '🚨 PeerX Health Check Failed',
              body: 'Automated health check detected critical issues. See workflow run for details.',
              labels: ['health-check', 'urgent']
            })
```

### GitLab CI/CD

```yaml
stages:
  - health-check

health-check:
  stage: health-check
  image: rust:latest
  script:
    - cd peerx-cli
    - cargo build --release
    - ./target/release/peerx health --format json
  variables:
    PEERX_CONTRACT_ID: $CONTRACT_ID
    PEERX_NETWORK: testnet
  artifacts:
    when: always
    reports:
      json: health-report.json
  only:
    - schedules
    - main
```

### Docker

```dockerfile
FROM rust:1.74 as builder

WORKDIR /app
COPY peerx-cli/ .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/peerx /usr/local/bin/

ENTRYPOINT ["peerx"]
CMD ["health", "--format", "json"]
```

**Usage:**
```bash
docker build -t peerx-cli .
docker run --rm \
  -e PEERX_CONTRACT_ID=CXXXXX... \
  -e PEERX_NETWORK=testnet \
  peerx-cli
```

### Monitoring Integration

#### Prometheus

```bash
# health_check.sh
#!/bin/bash
peerx health --format json --quiet > /tmp/health.json
EXIT_CODE=$?

# Export metrics
echo "peerx_health_status $EXIT_CODE" > /var/lib/node_exporter/textfile_collector/peerx_health.prom
echo "peerx_health_duration_ms $(jq '.total_duration_ms' /tmp/health.json)" >> /var/lib/node_exporter/textfile_collector/peerx_health.prom
echo "peerx_health_checks_total $(jq '.summary.total' /tmp/health.json)" >> /var/lib/node_exporter/textfile_collector/peerx_health.prom
echo "peerx_health_checks_failed $(jq '.summary.failed' /tmp/health.json)" >> /var/lib/node_exporter/textfile_collector/peerx_health.prom
```

#### Datadog

```bash
#!/bin/bash
peerx health --format json --quiet > /tmp/health.json
EXIT_CODE=$?

# Send to Datadog
curl -X POST "https://api.datadoghq.com/api/v1/series" \
  -H "Content-Type: application/json" \
  -H "DD-API-KEY: ${DD_API_KEY}" \
  -d @- << EOF
{
  "series": [
    {
      "metric": "peerx.health.status",
      "points": [[$(date +%s), $EXIT_CODE]],
      "type": "gauge",
      "tags": ["env:production", "service:peerx-contracts"]
    }
  ]
}
EOF
```

## Best Practices

### 1. Pre-Deployment Checks

Always run health checks before deploying:

```bash
#!/bin/bash
set -e

echo "Running pre-deployment health check..."
peerx health --format json > pre-deploy-health.json

if [ $? -ne 0 ]; then
  echo "❌ Health check failed - aborting deployment"
  exit 1
fi

echo "✅ Health check passed - proceeding with deployment"
# ... deployment commands ...
```

### 2. Continuous Monitoring

Run health checks on a schedule:

```bash
# crontab -e
*/5 * * * * /usr/local/bin/peerx health --quiet || /usr/local/bin/alert-oncall.sh
```

### 3. Post-Incident Validation

After resolving incidents:

```bash
# Keep checking until healthy
while true; do
  peerx health --format json
  if [ $? -eq 0 ]; then
    echo "✅ System recovered"
    break
  fi
  echo "Still unhealthy, checking again in 30s..."
  sleep 30
done
```

### 4. Environment-Specific Configs

```bash
# Production
export PEERX_NETWORK=mainnet
export PEERX_CONTRACT_ID=$PROD_CONTRACT_ID

# Staging
export PEERX_NETWORK=testnet
export PEERX_CONTRACT_ID=$STAGING_CONTRACT_ID
```

### 5. Alerting Thresholds

```bash
peerx health --format json > health.json
FAILED=$(jq '.summary.failed' health.json)
WARNINGS=$(jq '.summary.warnings' health.json)

if [ "$FAILED" -gt 0 ]; then
  alert-critical "PeerX health check failed: $FAILED critical issues"
elif [ "$WARNINGS" -gt 2 ]; then
  alert-warning "PeerX health check: $WARNINGS warnings detected"
fi
```

## Troubleshooting

### Common Issues

#### "Contract ID is required"

**Problem:** No contract ID configured

**Solution:**
```bash
export PEERX_CONTRACT_ID="CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
# Or set in config file
```

#### "RPC endpoint unreachable"

**Problem:** Cannot connect to Soroban RPC

**Debug:**
```bash
# Test connectivity
curl -v https://soroban-testnet.stellar.org/health

# Check DNS
nslookup soroban-testnet.stellar.org

# Try alternative endpoint
peerx health --rpc-url https://rpc.stellar.org
```

#### "Timeout: operation took too long"

**Problem:** Checks taking longer than timeout

**Solution:**
```json
{
  "health": {
    "timeout_seconds": 60
  }
}
```

#### "Invalid contract ID format"

**Problem:** Contract ID doesn't match expected format

**Check:**
- Must be exactly 56 characters
- Must start with 'C'
- Must be valid base32

**Verify:**
```bash
echo $PEERX_CONTRACT_ID | wc -c  # Should be 57 (56 + newline)
echo $PEERX_CONTRACT_ID | head -c1  # Should be 'C'
```

### Debug Mode

```bash
# Enable verbose logging
RUST_LOG=debug peerx health --verbose

# Or maximum verbosity
RUST_LOG=trace peerx health --verbose
```

### Getting Help

```bash
# Command help
peerx health --help

# Version info
peerx --version

# Check configuration
peerx health --details
```

## Advanced Usage

### Custom Check Subsets

Run only specific checks:

```bash
# Only check connectivity
peerx health --only rpc_reachable,horizon_reachable

# Only check contract state
peerx health --only contract_exists,contract_not_paused

# Single check
peerx health --only oracle_fresh
```

### Scripted Monitoring

```bash
#!/bin/bash
# monitor.sh - Continuous health monitoring

INTERVAL=60  # seconds
LOG_FILE="/var/log/peerx-health.log"

while true; do
  TIMESTAMP=$(date -Iseconds)
  peerx health --format json --quiet > /tmp/health.json
  EXIT_CODE=$?
  
  echo "$TIMESTAMP | Exit: $EXIT_CODE | $(cat /tmp/health.json)" >> $LOG_FILE
  
  if [ $EXIT_CODE -eq 2 ]; then
    # Critical failure
    echo "CRITICAL: PeerX health check failed!" | mail -s "PeerX Alert" oncall@example.com
  fi
  
  sleep $INTERVAL
done
```

### Integration with External Systems

```bash
# Send to Slack
peerx health --format json | jq -r '
  "Health Status: \(.status)\n" +
  "Checks: \(.summary.passed)/\(.summary.total) passed\n" +
  "Duration: \(.total_duration_ms)ms"
' | slack-cli --channel ops-alerts

# Send to PagerDuty
if ! peerx health --quiet; then
  pd-send --severity critical --summary "PeerX health check failed"
fi
```

## Support

- **Issues:** https://github.com/coderolisa/Peerx-Contracts/issues
- **Documentation:** https://github.com/coderolisa/Peerx-Contracts/tree/main/peerx-cli
- **Community:** [Discord/Telegram/Forum link]

---

**Last Updated:** 2024-01-15  
**CLI Version:** 0.1.0
