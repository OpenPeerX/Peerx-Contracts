#!/bin/bash
# Example: CI/CD pipeline integration

set -e

echo "PeerX Pre-Deployment Health Check"
echo "===================================="

# Run health check with JSON output
HEALTH_OUTPUT=$(peerx health --format json)
EXIT_CODE=$?

# Save report
echo "$HEALTH_OUTPUT" > health-report.json
echo "Health report saved to health-report.json"

# Parse results
OVERALL_STATUS=$(echo "$HEALTH_OUTPUT" | jq -r '.overall_status')
SUMMARY=$(echo "$HEALTH_OUTPUT" | jq -r '.summary')

echo ""
echo "Overall Status: $OVERALL_STATUS"
echo "Summary: $SUMMARY"

# Check for critical failures
if [ $EXIT_CODE -eq 2 ]; then
    echo ""
    echo "❌ CRITICAL: Deployment blocked due to health check failures"
    echo "Failed checks:"
    echo "$HEALTH_OUTPUT" | jq -r '.checks[] | select(.status == "critical") | "  - \(.name): \(.message)"'
    exit 2
fi

# Check for warnings
if [ $EXIT_CODE -eq 1 ]; then
    echo ""
    echo "⚠️  WARNING: Health check warnings detected"
    echo "Warning checks:"
    echo "$HEALTH_OUTPUT" | jq -r '.checks[] | select(.status == "warning") | "  - \(.name): \(.message)"'
    
    # Optionally block deployment on warnings
    if [ "${BLOCK_ON_WARNINGS}" = "true" ]; then
        echo "BLOCK_ON_WARNINGS is set, blocking deployment"
        exit 1
    fi
fi

echo ""
echo "✅ Health checks passed, proceeding with deployment"
exit 0
