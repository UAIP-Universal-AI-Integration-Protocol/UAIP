#!/bin/bash
# Load Testing Script for UAIP Hub
# Performs load testing on various endpoints and generates reports

set -euo pipefail

# Configuration
UAIP_URL="${UAIP_URL:-http://localhost:8443}"
DURATION="${DURATION:-30}"
CONCURRENCY="${CONCURRENCY:-10}"
REQUEST_RATE="${REQUEST_RATE:-100}"
OUTPUT_DIR="${OUTPUT_DIR:-./load-test-results}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Create output directory
mkdir -p "$OUTPUT_DIR"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="${OUTPUT_DIR}/load-test-${TIMESTAMP}.txt"

# Check if required tools are installed
check_tool() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${RED}Error: $1 is not installed${NC}"
        echo "Install with: $2"
        exit 1
    fi
}

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  UAIP Hub Load Testing${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "URL: ${YELLOW}${UAIP_URL}${NC}"
echo -e "Duration: ${YELLOW}${DURATION}s${NC}"
echo -e "Concurrency: ${YELLOW}${CONCURRENCY}${NC}"
echo -e "Target rate: ${YELLOW}${REQUEST_RATE} req/s${NC}"
echo -e "Report: ${YELLOW}${REPORT_FILE}${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Initialize report
cat > "$REPORT_FILE" <<EOF
UAIP Hub Load Test Report
Generated: $(date)
Duration: ${DURATION}s
Concurrency: ${CONCURRENCY}
Target Rate: ${REQUEST_RATE} req/s
URL: ${UAIP_URL}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

EOF

# Function to run load test on an endpoint
load_test_endpoint() {
    local endpoint=$1
    local method=${2:-GET}
    local name=${3:-$endpoint}

    echo -e "${BLUE}Testing: ${YELLOW}${method} ${endpoint}${NC}"

    # Check if hey is available (fast HTTP load generator)
    if command -v hey &> /dev/null; then
        echo "  Using 'hey' for load testing..."
        hey -z "${DURATION}s" -c "$CONCURRENCY" -m "$method" "${UAIP_URL}${endpoint}" 2>&1 | tee -a "$REPORT_FILE"
    # Fallback to Apache Bench (ab)
    elif command -v ab &> /dev/null; then
        echo "  Using 'ab' for load testing..."
        local total_requests=$((REQUEST_RATE * DURATION))
        ab -t "$DURATION" -c "$CONCURRENCY" -n "$total_requests" "${UAIP_URL}${endpoint}" 2>&1 | tee -a "$REPORT_FILE"
    # Fallback to curl loop
    else
        echo "  Using 'curl' for basic load testing..."
        echo "  (Install 'hey' or 'ab' for better load testing)"

        local start_time=$(date +%s)
        local end_time=$((start_time + DURATION))
        local requests=0
        local errors=0
        local total_time=0

        while [ $(date +%s) -lt $end_time ]; do
            for i in $(seq 1 $CONCURRENCY); do
                {
                    local req_start=$(date +%s.%N)
                    if curl -s -o /dev/null -w "%{http_code}" "${UAIP_URL}${endpoint}" | grep -q "^2"; then
                        local req_end=$(date +%s.%N)
                        local req_time=$(echo "$req_end - $req_start" | bc)
                        total_time=$(echo "$total_time + $req_time" | bc)
                    else
                        ((errors++))
                    fi
                    ((requests++))
                } &
            done
            wait
            sleep 0.1
        done

        local actual_duration=$(($(date +%s) - start_time))
        local rps=$(echo "scale=2; $requests / $actual_duration" | bc)
        local avg_latency=$(echo "scale=3; $total_time / $requests" | bc)

        cat >> "$REPORT_FILE" <<EOF

Endpoint: ${method} ${endpoint}
Total requests: ${requests}
Errors: ${errors}
Duration: ${actual_duration}s
Requests/sec: ${rps}
Average latency: ${avg_latency}s

EOF

        echo -e "  ${GREEN}✓ Completed${NC} - ${requests} requests, ${rps} req/s, ${avg_latency}s avg latency"
    fi

    echo "" | tee -a "$REPORT_FILE"
}

# Pre-flight check
echo -e "${BLUE}Pre-flight check...${NC}"
if curl -s -o /dev/null -w "%{http_code}" "${UAIP_URL}/api/v1/system/health" | grep -q "^2"; then
    echo -e "${GREEN}✓ UAIP Hub is reachable${NC}"
else
    echo -e "${RED}✗ UAIP Hub is not reachable at ${UAIP_URL}${NC}"
    exit 1
fi
echo ""

# Run load tests on different endpoints
echo -e "${BLUE}Starting load tests...${NC}\n"

# Test 1: Health endpoint
load_test_endpoint "/api/v1/system/health" "GET" "Health Check"

# Test 2: Metrics endpoint
load_test_endpoint "/metrics" "GET" "Metrics"

# Test 3: Device list endpoint (if authenticated)
# Uncomment and add authentication if needed
# load_test_endpoint "/api/v1/devices" "GET" "Device List"

# Test 4: Mixed workload simulation
echo -e "${BLUE}Running mixed workload simulation...${NC}"
echo "Mixed Workload Simulation" >> "$REPORT_FILE"

if command -v hey &> /dev/null; then
    # 70% health checks, 30% metrics
    {
        hey -z "$((DURATION * 7 / 10))s" -c "$((CONCURRENCY * 7 / 10))" "${UAIP_URL}/api/v1/system/health" &
        hey -z "$((DURATION * 3 / 10))s" -c "$((CONCURRENCY * 3 / 10))" "${UAIP_URL}/metrics" &
        wait
    } 2>&1 | tee -a "$REPORT_FILE"
fi
echo ""

# Generate summary
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✓ Load test complete${NC}"
echo -e "Report saved to: ${YELLOW}${REPORT_FILE}${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Show metrics
echo -e "${BLUE}Current system metrics:${NC}"
curl -s "${UAIP_URL}/metrics" | grep -E "^uaip_" | head -10 || echo "  (Metrics endpoint unavailable)"

echo ""
echo -e "${YELLOW}Tip: Install 'hey' for better load testing:${NC}"
echo -e "  go install github.com/rakyll/hey@latest"
