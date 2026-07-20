#!/bin/bash
# Example: Basic health check script

set -e

# Configuration
export PEERX_CONTRACT_ID="CDXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
export PEERX_RPC_URL="https://soroban-testnet.stellar.org"
export PEERX_NETWORK="testnet"

echo "Running PeerX health checks..."
echo "================================"

# Run health check
if peerx health; then
    echo ""
    echo "✓ All health checks passed!"
    exit 0
else
    EXIT_CODE=$?
    echo ""
    if [ $EXIT_CODE -eq 1 ]; then
        echo "⚠ Health check warnings detected"
    else
        echo "✗ Critical health check failures"
    fi
    exit $EXIT_CODE
fi
