#!/bin/bash
# Log Analysis Script for UAIP Hub
# Analyzes logs and generates insights

set -euo pipefail

# Configuration
LOG_SOURCE="${LOG_SOURCE:-docker}"  # docker, k8s, or file
LOG_FILE="${LOG_FILE:-}"
NAMESPACE="${NAMESPACE:-uaip}"
TIME_RANGE="${TIME_RANGE:-1h}"
OUTPUT_FILE="${OUTPUT_FILE:-./log-analysis-$(date +%Y%m%d_%H%M%S).txt}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Display usage
usage() {
    cat <<EOF
Usage: $0 [options]

Options:
  LOG_SOURCE=<source>      Log source: docker, k8s, or file (default: docker)
  LOG_FILE=<path>          Path to log file (for LOG_SOURCE=file)
  NAMESPACE=<namespace>    Kubernetes namespace (default: uaip)
  TIME_RANGE=<range>       Time range for k8s logs (default: 1h)
  OUTPUT_FILE=<path>       Output file for analysis results

Examples:
  $0                                    # Analyze Docker logs
  $0 LOG_SOURCE=k8s                     # Analyze Kubernetes logs
  $0 LOG_SOURCE=file LOG_FILE=app.log   # Analyze file logs

EOF
    exit 1
}

# Get logs based on source
get_logs() {
    case "$LOG_SOURCE" in
        docker)
            docker-compose -f docker-compose.dev.yml logs --no-color uaip-hub 2>/dev/null || \
            docker logs uaip-hub 2>/dev/null || \
            echo ""
            ;;
        k8s)
            kubectl logs -n "$NAMESPACE" -l app=uaip-hub --tail=10000 --since="$TIME_RANGE" 2>/dev/null || echo ""
            ;;
        file)
            if [ -z "$LOG_FILE" ]; then
                echo "Error: LOG_FILE not specified for LOG_SOURCE=file"
                exit 1
            fi
            cat "$LOG_FILE" 2>/dev/null || echo ""
            ;;
        *)
            echo "Error: Invalid LOG_SOURCE: $LOG_SOURCE"
            exit 1
            ;;
    esac
}

# Analyze logs
analyze_logs() {
    local logs="$1"

    if [ -z "$logs" ]; then
        echo -e "${RED}No logs found${NC}"
        exit 1
    fi

    cat > "$OUTPUT_FILE" <<EOF
UAIP Hub Log Analysis Report
Generated: $(date)
Source: ${LOG_SOURCE}
Time Range: ${TIME_RANGE}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

EOF

    # Count total log lines
    local total_lines=$(echo "$logs" | wc -l)
    echo "SUMMARY" >> "$OUTPUT_FILE"
    echo "Total log entries: ${total_lines}" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"

    # Count log levels
    echo "LOG LEVELS:" >> "$OUTPUT_FILE"
    echo "───────────────────────────────────────────────────────" >> "$OUTPUT_FILE"
    echo "$logs" | grep -oP '(ERROR|WARN|INFO|DEBUG|TRACE)' | sort | uniq -c | sort -rn | \
        awk '{printf "%-10s %6d (%5.2f%%)\n", $2, $1, ($1/'$total_lines')*100}' >> "$OUTPUT_FILE" || \
        echo "  (No log level markers found)" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"

    # Count errors
    local error_count=$(echo "$logs" | grep -ci "error" || echo "0")
    echo "ERRORS:" >> "$OUTPUT_FILE"
    echo "───────────────────────────────────────────────────────" >> "$OUTPUT_FILE"
    echo "Total errors: ${error_count}" >> "$OUTPUT_FILE"
    if [ "$error_count" -gt 0 ]; then
        echo "" >> "$OUTPUT_FILE"
        echo "Top error messages:" >> "$OUTPUT_FILE"
        echo "$logs" | grep -i "error" | sed 's/.*error[^:]*: //I' | sort | uniq -c | sort -rn | head -10 >> "$OUTPUT_FILE"
    fi
    echo "" >> "$OUTPUT_FILE"

    # Count warnings
    local warning_count=$(echo "$logs" | grep -ci "warn" || echo "0")
    echo "WARNINGS:" >> "$OUTPUT_FILE"
    echo "───────────────────────────────────────────────────────" >> "$OUTPUT_FILE"
    echo "Total warnings: ${warning_count}" >> "$OUTPUT_FILE"
    if [ "$warning_count" -gt 0 ]; then
        echo "" >> "$OUTPUT_FILE"
        echo "Top warning messages:" >> "$OUTPUT_FILE"
        echo "$logs" | grep -i "warn" | sed 's/.*warn[^:]*: //I' | sort | uniq -c | sort -rn | head -10 >> "$OUTPUT_FILE"
    fi
    echo "" >> "$OUTPUT_FILE"

    # HTTP status codes
    echo "HTTP STATUS CODES:" >> "$OUTPUT_FILE"
    echo "───────────────────────────────────────────────────────" >> "$OUTPUT_FILE"
    echo "$logs" | grep -oP 'status=\K\d+' | sort | uniq -c | sort -rn | \
        awk '{printf "%3s: %6d requests\n", $2, $1}' >> "$OUTPUT_FILE" || \
        echo "  (No HTTP status codes found)" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"

    # Top endpoints
    echo "TOP ENDPOINTS:" >> "$OUTPUT_FILE"
    echo "───────────────────────────────────────────────────────" >> "$OUTPUT_FILE"
    echo "$logs" | grep -oP '(GET|POST|PUT|DELETE|PATCH)\s+\K[^\s]+' | sort | uniq -c | sort -rn | head -10 | \
        awk '{printf "%-50s %6d\n", $2, $1}' >> "$OUTPUT_FILE" || \
        echo "  (No endpoints found)" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"

    # Response times
    echo "RESPONSE TIMES:" >> "$OUTPUT_FILE"
    echo "───────────────────────────────────────────────────────" >> "$OUTPUT_FILE"
    local response_times=$(echo "$logs" | grep -oP 'duration_ms=\K[\d.]+' || echo "")
    if [ -n "$response_times" ]; then
        local avg_response=$(echo "$response_times" | awk '{sum+=$1; count++} END {printf "%.2f", sum/count}')
        local max_response=$(echo "$response_times" | sort -n | tail -1)
        local min_response=$(echo "$response_times" | sort -n | head -1)
        echo "Average: ${avg_response}ms" >> "$OUTPUT_FILE"
        echo "Min: ${min_response}ms" >> "$OUTPUT_FILE"
        echo "Max: ${max_response}ms" >> "$OUTPUT_FILE"
    else
        echo "  (No response time data found)" >> "$OUTPUT_FILE"
    fi
    echo "" >> "$OUTPUT_FILE"

    # Database queries
    echo "DATABASE ACTIVITY:" >> "$OUTPUT_FILE"
    echo "───────────────────────────────────────────────────────" >> "$OUTPUT_FILE"
    local db_queries=$(echo "$logs" | grep -ci "query\|sql" || echo "0")
    echo "Total database operations: ${db_queries}" >> "$OUTPUT_FILE"
    if [ "$db_queries" -gt 0 ]; then
        echo "" >> "$OUTPUT_FILE"
        echo "Top database operations:" >> "$OUTPUT_FILE"
        echo "$logs" | grep -i "query\|sql" | grep -oP '(SELECT|INSERT|UPDATE|DELETE)\s+[^\s]+' | \
            sort | uniq -c | sort -rn | head -10 >> "$OUTPUT_FILE" || echo "  (Unable to parse operations)" >> "$OUTPUT_FILE"
    fi
    echo "" >> "$OUTPUT_FILE"

    # Unique request IDs
    local unique_requests=$(echo "$logs" | grep -oP 'request_id=\K[^\s]+' | sort -u | wc -l || echo "0")
    echo "REQUESTS:" >> "$OUTPUT_FILE"
    echo "───────────────────────────────────────────────────────" >> "$OUTPUT_FILE"
    echo "Unique requests (by request_id): ${unique_requests}" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"

    # Performance issues
    echo "PERFORMANCE ISSUES:" >> "$OUTPUT_FILE"
    echo "───────────────────────────────────────────────────────" >> "$OUTPUT_FILE"
    local slow_requests=$(echo "$logs" | grep -oP 'duration_ms=\K[\d.]+' | awk '$1 > 1000' | wc -l || echo "0")
    echo "Slow requests (>1s): ${slow_requests}" >> "$OUTPUT_FILE"
    if [ "$slow_requests" -gt 0 ]; then
        echo "" >> "$OUTPUT_FILE"
        echo "Slowest requests:" >> "$OUTPUT_FILE"
        echo "$logs" | grep "duration_ms=" | sort -t= -k5 -n | tail -10 | \
            awk '{print $0}' >> "$OUTPUT_FILE"
    fi
    echo "" >> "$OUTPUT_FILE"

    # Security events
    echo "SECURITY EVENTS:" >> "$OUTPUT_FILE"
    echo "───────────────────────────────────────────────────────" >> "$OUTPUT_FILE"
    local auth_failures=$(echo "$logs" | grep -ci "authentication failed\|unauthorized\|forbidden" || echo "0")
    local rate_limits=$(echo "$logs" | grep -ci "rate limit\|too many requests" || echo "0")
    echo "Authentication failures: ${auth_failures}" >> "$OUTPUT_FILE"
    echo "Rate limit hits: ${rate_limits}" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"

    # Recommendations
    echo "RECOMMENDATIONS:" >> "$OUTPUT_FILE"
    echo "───────────────────────────────────────────────────────" >> "$OUTPUT_FILE"
    if [ "$error_count" -gt 100 ]; then
        echo "⚠ High error rate detected (${error_count} errors)" >> "$OUTPUT_FILE"
        echo "  - Review error logs for patterns" >> "$OUTPUT_FILE"
        echo "  - Check application health" >> "$OUTPUT_FILE"
    fi
    if [ "$slow_requests" -gt 50 ]; then
        echo "⚠ Performance degradation detected (${slow_requests} slow requests)" >> "$OUTPUT_FILE"
        echo "  - Review database query performance" >> "$OUTPUT_FILE"
        echo "  - Check resource utilization" >> "$OUTPUT_FILE"
    fi
    if [ "$auth_failures" -gt 50 ]; then
        echo "⚠ Unusual authentication activity (${auth_failures} failures)" >> "$OUTPUT_FILE"
        echo "  - Investigate potential security threat" >> "$OUTPUT_FILE"
        echo "  - Review authentication logs" >> "$OUTPUT_FILE"
    fi
    if [ "$error_count" -eq 0 ] && [ "$slow_requests" -lt 10 ]; then
        echo "✓ System appears healthy" >> "$OUTPUT_FILE"
    fi
    echo "" >> "$OUTPUT_FILE"
}

# Display header
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  UAIP Hub Log Analysis${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "Source: ${YELLOW}${LOG_SOURCE}${NC}"
echo -e "Output: ${YELLOW}${OUTPUT_FILE}${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Get and analyze logs
echo -e "${BLUE}Fetching logs...${NC}"
logs=$(get_logs)

echo -e "${BLUE}Analyzing logs...${NC}"
analyze_logs "$logs"

# Display results
echo -e "${GREEN}✓ Analysis complete${NC}"
echo ""
cat "$OUTPUT_FILE"
echo ""
echo -e "Full report saved to: ${YELLOW}${OUTPUT_FILE}${NC}"
