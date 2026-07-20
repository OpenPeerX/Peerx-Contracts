#!/bin/bash
# Example: Continuous monitoring with cron
# Add to crontab: */5 * * * * /path/to/monitoring_cron.sh

TIMESTAMP=$(date +%Y%m%d-%H%M%S)
LOG_DIR="/var/log/peerx"
ALERT_WEBHOOK="${SLACK_WEBHOOK_URL:-}"

# Ensure log directory exists
mkdir -p "$LOG_DIR"

# Run health check
HEALTH_OUTPUT=$(peerx health --format json 2>&1)
EXIT_CODE=$?

# Save to log file
echo "$HEALTH_OUTPUT" > "$LOG_DIR/health-$TIMESTAMP.json"

# Alert on failures
if [ $EXIT_CODE -ne 0 ]; then
    OVERALL_STATUS=$(echo "$HEALTH_OUTPUT" | jq -r '.overall_status' 2>/dev/null || echo "unknown")
    SUMMARY=$(echo "$HEALTH_OUTPUT" | jq -r '.summary' 2>/dev/null || echo "Health check failed")
    
    # Send alert (example with curl to webhook)
    if [ -n "$ALERT_WEBHOOK" ]; then
        curl -X POST "$ALERT_WEBHOOK" \
            -H "Content-Type: application/json" \
            -d "{
                \"text\": \"PeerX Health Check Failed\",
                \"attachments\": [{
                    \"color\": \"danger\",
                    \"fields\": [
                        {\"title\": \"Status\", \"value\": \"$OVERALL_STATUS\", \"short\": true},
                        {\"title\": \"Summary\", \"value\": \"$SUMMARY\", \"short\": false},
                        {\"title\": \"Timestamp\", \"value\": \"$TIMESTAMP\", \"short\": true}
                    ]
                }]
            }"
    fi
    
    # Log to syslog
    logger -t peerx-health "Health check failed: $SUMMARY"
fi

# Clean up old logs (keep last 7 days)
find "$LOG_DIR" -name "health-*.json" -mtime +7 -delete
