#!/bin/bash
# Health Monitoring Script for UAIP Hub
# Continuously monitors system health and alerts on issues

set -euo pipefail

# Configuration
UAIP_URL="${UAIP_URL:-http://localhost:8443}"
CHECK_INTERVAL="${CHECK_INTERVAL:-10}"
ALERT_ON_FAILURE="${ALERT_ON_FAILURE:-true}"
LOG_FILE="${LOG_FILE:-/tmp/uaip-health-monitor.log}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Counters
TOTAL_CHECKS=0
FAILED_CHECKS=0
CONSECUTIVE_FAILURES=0

# Log function
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"
}

# Check health endpoint
check_health() {
    local response
    local http_code
    local status

    response=$(curl -s -w "\n%{http_code}" "${UAIP_URL}/api/v1/system/health" 2>/dev/null || echo "000")
    http_code=$(echo "$response" | tail -n 1)

    if [ "$http_code" = "200" ]; then
        status=$(echo "$response" | head -n -1 | jq -r '.status' 2>/dev/null || echo "unknown")
        if [ "$status" = "healthy" ]; then
            return 0
        else
            return 1
        fi
    else
        return 1
    fi
}

# Check component health
check_component() {
    local component=$1
    local response

    response=$(curl -s "${UAIP_URL}/api/v1/system/health" | jq -r ".dependencies[] | select(.name == \"$component\") | .status" 2>/dev/null || echo "unknown")
    echo "$response"
}

# Alert function (can be extended to send emails, Slack messages, etc.)
send_alert() {
    local message=$1
    echo -e "${RED}ALERT: ${message}${NC}"
    log "ALERT: ${message}"

    # Add your alerting logic here (email, Slack, PagerDuty, etc.)
    # Example: curl -X POST https://hooks.slack.com/... -d "{\"text\": \"$message\"}"
}

# Display header
clear
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  UAIP Hub Health Monitor${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "URL: ${YELLOW}${UAIP_URL}${NC}"
echo -e "Check interval: ${YELLOW}${CHECK_INTERVAL}s${NC}"
echo -e "Log file: ${YELLOW}${LOG_FILE}${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

log "Health monitor started"

# Main monitoring loop
while true; do
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

    echo -e "${BLUE}Check #${TOTAL_CHECKS}${NC} - $(date '+%Y-%m-%d %H:%M:%S')"

    # Check overall health
    if check_health; then
        echo -e "  Overall Status: ${GREEN}✓ HEALTHY${NC}"
        CONSECUTIVE_FAILURES=0

        # Check individual components
        postgres_status=$(check_component "PostgreSQL")
        redis_status=$(check_component "Redis")
        nats_status=$(check_component "NATS")

        echo -e "  PostgreSQL:     $([ "$postgres_status" = "healthy" ] && echo -e "${GREEN}✓${NC}" || echo -e "${YELLOW}⚠${NC}") ${postgres_status}"
        echo -e "  Redis:          $([ "$redis_status" = "healthy" ] && echo -e "${GREEN}✓${NC}" || echo -e "${YELLOW}⚠${NC}") ${redis_status}"
        echo -e "  NATS:           $([ "$nats_status" = "healthy" ] && echo -e "${GREEN}✓${NC}" || echo -e "${YELLOW}⚠${NC}") ${nats_status}"

        # Check for degraded components
        if [ "$postgres_status" != "healthy" ] || [ "$redis_status" != "healthy" ] || [ "$nats_status" != "healthy" ]; then
            log "WARNING: System degraded - PostgreSQL: $postgres_status, Redis: $redis_status, NATS: $nats_status"
            if [ "$ALERT_ON_FAILURE" = "true" ]; then
                send_alert "UAIP Hub is degraded - Check component health"
            fi
        fi
    else
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
        CONSECUTIVE_FAILURES=$((CONSECUTIVE_FAILURES + 1))
        echo -e "  Overall Status: ${RED}✗ UNHEALTHY${NC}"
        echo -e "  Consecutive failures: ${RED}${CONSECUTIVE_FAILURES}${NC}"

        log "ERROR: Health check failed (consecutive failures: ${CONSECUTIVE_FAILURES})"

        # Alert on first failure and every 5 consecutive failures
        if [ "$ALERT_ON_FAILURE" = "true" ] && { [ "$CONSECUTIVE_FAILURES" -eq 1 ] || [ $((CONSECUTIVE_FAILURES % 5)) -eq 0 ]; }; then
            send_alert "UAIP Hub is unhealthy (${CONSECUTIVE_FAILURES} consecutive failures)"
        fi
    fi

    # Display statistics
    SUCCESS_RATE=$(awk "BEGIN {printf \"%.2f\", 100 * (1 - ${FAILED_CHECKS}/${TOTAL_CHECKS})}")
    echo -e "  Success rate:   ${YELLOW}${SUCCESS_RATE}%${NC} (${FAILED_CHECKS}/${TOTAL_CHECKS} failures)"
    echo ""

    sleep "$CHECK_INTERVAL"
done
