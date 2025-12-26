#!/bin/bash
# Database Performance Analysis Script for UAIP Hub
# Analyzes PostgreSQL database and provides optimization recommendations

set -euo pipefail

# Configuration
DB_NAME="${DB_NAME:-uaip}"
DB_USER="${DB_USER:-uaip}"
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
OUTPUT_FILE="${OUTPUT_FILE:-./database-analysis-$(date +%Y%m%d_%H%M%S).txt}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Parse command line arguments
ANALYSIS_TYPE="full"
while [[ $# -gt 0 ]]; do
    case $1 in
        --slow-queries)
            ANALYSIS_TYPE="slow-queries"
            shift
            ;;
        --index-usage)
            ANALYSIS_TYPE="index-usage"
            shift
            ;;
        --full)
            ANALYSIS_TYPE="full"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--slow-queries|--index-usage|--full]"
            exit 1
            ;;
    esac
done

# Function to run SQL query
run_query() {
    local query=$1
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -A -F"," -c "$query" 2>/dev/null || echo ""
}

# Function to run SQL and format output
run_query_formatted() {
    local query=$1
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "$query" 2>/dev/null || echo ""
}

# Display header
header() {
    echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# Check connection
check_connection() {
    header "Database Connection Check"
    echo "Database: ${YELLOW}${DB_NAME}${NC}"
    echo "Host: ${YELLOW}${DB_HOST}:${DB_PORT}${NC}"
    echo "User: ${YELLOW}${DB_USER}${NC}"
    echo ""

    if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT 1" &>/dev/null; then
        echo -e "${GREEN}✓ Connection successful${NC}"
    else
        echo -e "${RED}✗ Connection failed${NC}"
        echo "Please check your database credentials and ensure PostgreSQL is running"
        exit 1
    fi
}

# Database overview
database_overview() {
    header "Database Overview"

    # Database size
    local db_size=$(run_query "SELECT pg_size_pretty(pg_database_size('$DB_NAME'));")
    echo "Database size: ${YELLOW}${db_size}${NC}"

    # Number of tables
    local table_count=$(run_query "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';")
    echo "Number of tables: ${YELLOW}${table_count}${NC}"

    # Number of indexes
    local index_count=$(run_query "SELECT COUNT(*) FROM pg_indexes WHERE schemaname = 'public';")
    echo "Number of indexes: ${YELLOW}${index_count}${NC}"

    # Active connections
    local connections=$(run_query "SELECT COUNT(*) FROM pg_stat_activity WHERE datname = '$DB_NAME';")
    echo "Active connections: ${YELLOW}${connections}${NC}"

    echo ""
}

# Table statistics
table_statistics() {
    header "Table Statistics"

    run_query_formatted "
    SELECT
        schemaname,
        relname as table_name,
        n_live_tup as live_rows,
        n_dead_tup as dead_rows,
        pg_size_pretty(pg_total_relation_size(schemaname||'.'||relname)) as total_size,
        ROUND(100.0 * n_dead_tup / NULLIF(n_live_tup + n_dead_tup, 0), 2) as dead_ratio
    FROM pg_stat_user_tables
    WHERE schemaname = 'public'
    ORDER BY pg_total_relation_size(schemaname||'.'||relname) DESC;
    "

    echo ""
}

# Index usage analysis
index_usage() {
    header "Index Usage Analysis"

    echo -e "${YELLOW}Most Used Indexes:${NC}"
    run_query_formatted "
    SELECT
        schemaname,
        tablename,
        indexname,
        idx_scan as scans,
        pg_size_pretty(pg_relation_size(indexrelid)) as size
    FROM pg_stat_user_indexes
    WHERE schemaname = 'public'
    ORDER BY idx_scan DESC
    LIMIT 10;
    "

    echo ""
    echo -e "${YELLOW}Unused Indexes (potential candidates for removal):${NC}"
    run_query_formatted "
    SELECT
        schemaname,
        tablename,
        indexname,
        idx_scan as scans,
        pg_size_pretty(pg_relation_size(indexrelid)) as size
    FROM pg_stat_user_indexes
    WHERE idx_scan = 0
    AND schemaname = 'public'
    AND indexname NOT LIKE '%_pkey';
    "

    echo ""
    echo -e "${YELLOW}Index Hit Rate (should be > 95%):${NC}"
    run_query_formatted "
    SELECT
        ROUND(100.0 * sum(idx_blks_hit) / NULLIF(sum(idx_blks_hit + idx_blks_read), 0), 2) as index_hit_rate
    FROM pg_statio_user_indexes;
    "

    echo ""
}

# Slow queries
slow_queries() {
    header "Currently Running Queries"

    run_query_formatted "
    SELECT
        pid,
        usename,
        application_name,
        state,
        EXTRACT(EPOCH FROM (now() - query_start)) as duration_seconds,
        LEFT(query, 80) as query
    FROM pg_stat_activity
    WHERE state != 'idle'
    AND datname = '$DB_NAME'
    ORDER BY query_start ASC;
    "

    echo ""
    echo -e "${YELLOW}Note: Enable slow query logging to track historical slow queries${NC}"
    echo "Run: ALTER SYSTEM SET log_min_duration_statement = 1000; -- 1 second"
    echo "Then: SELECT pg_reload_conf();"
    echo ""
}

# Cache hit ratios
cache_analysis() {
    header "Cache Hit Ratios"

    echo -e "${YELLOW}Table Cache Hit Rate (should be > 95%):${NC}"
    run_query_formatted "
    SELECT
        ROUND(100.0 * sum(heap_blks_hit) / NULLIF(sum(heap_blks_hit + heap_blks_read), 0), 2) as cache_hit_rate
    FROM pg_statio_user_tables;
    "

    echo ""
    echo -e "${YELLOW}Index Cache Hit Rate (should be > 95%):${NC}"
    run_query_formatted "
    SELECT
        ROUND(100.0 * sum(idx_blks_hit) / NULLIF(sum(idx_blks_hit + idx_blks_read), 0), 2) as index_cache_hit_rate
    FROM pg_statio_user_indexes;
    "

    echo ""
}

# Table bloat analysis
bloat_analysis() {
    header "Table Bloat Analysis"

    echo -e "${YELLOW}Tables with High Dead Tuple Ratio (> 10%):${NC}"
    run_query_formatted "
    SELECT
        schemaname,
        relname as table_name,
        n_live_tup as live_rows,
        n_dead_tup as dead_rows,
        ROUND(100.0 * n_dead_tup / NULLIF(n_live_tup + n_dead_tup, 0), 2) as dead_ratio,
        last_vacuum,
        last_autovacuum
    FROM pg_stat_user_tables
    WHERE schemaname = 'public'
    AND n_dead_tup > 0
    AND ROUND(100.0 * n_dead_tup / NULLIF(n_live_tup + n_dead_tup, 0), 2) > 10
    ORDER BY dead_ratio DESC;
    "

    echo ""
    echo -e "${YELLOW}Recommendation: Run VACUUM ANALYZE on tables with high dead tuple ratio${NC}"
    echo ""
}

# Lock information
lock_analysis() {
    header "Lock Information"

    run_query_formatted "
    SELECT
        pg_stat_activity.pid,
        pg_class.relname as table_name,
        pg_locks.mode,
        pg_locks.granted,
        pg_stat_activity.query
    FROM pg_locks
    JOIN pg_class ON pg_locks.relation = pg_class.oid
    JOIN pg_stat_activity ON pg_locks.pid = pg_stat_activity.pid
    WHERE pg_locks.granted = false;
    "

    echo ""
}

# Generate recommendations
generate_recommendations() {
    header "Optimization Recommendations"

    local recommendations=()

    # Check cache hit rate
    local cache_hit_rate=$(run_query "SELECT ROUND(100.0 * sum(heap_blks_hit) / NULLIF(sum(heap_blks_hit + heap_blks_read), 0), 2) FROM pg_statio_user_tables;")
    if (( $(echo "$cache_hit_rate < 95" | bc -l) )); then
        recommendations+=("⚠ Cache hit rate is ${cache_hit_rate}% (< 95%). Consider increasing shared_buffers.")
    fi

    # Check for unused indexes
    local unused_indexes=$(run_query "SELECT COUNT(*) FROM pg_stat_user_indexes WHERE idx_scan = 0 AND schemaname = 'public' AND indexname NOT LIKE '%_pkey';")
    if [ "$unused_indexes" -gt 0 ]; then
        recommendations+=("⚠ Found ${unused_indexes} unused indexes. Consider removing them to save space and improve write performance.")
    fi

    # Check for tables needing vacuum
    local bloated_tables=$(run_query "SELECT COUNT(*) FROM pg_stat_user_tables WHERE schemaname = 'public' AND n_dead_tup > 0 AND ROUND(100.0 * n_dead_tup / NULLIF(n_live_tup + n_dead_tup, 0), 2) > 10;")
    if [ "$bloated_tables" -gt 0 ]; then
        recommendations+=("⚠ Found ${bloated_tables} tables with >10% dead tuples. Run VACUUM ANALYZE.")
    fi

    # Check connection count
    local connections=$(run_query "SELECT COUNT(*) FROM pg_stat_activity WHERE datname = '$DB_NAME';")
    local max_connections=$(run_query "SHOW max_connections;" | grep -oP '\d+')
    local connection_percent=$(echo "scale=2; 100.0 * $connections / $max_connections" | bc)
    if (( $(echo "$connection_percent > 80" | bc -l) )); then
        recommendations+=("⚠ Using ${connection_percent}% of max connections. Consider increasing max_connections or implementing connection pooling.")
    fi

    if [ ${#recommendations[@]} -eq 0 ]; then
        echo -e "${GREEN}✓ No critical issues found. Database is well-optimized!${NC}"
    else
        for rec in "${recommendations[@]}"; do
            echo -e "${YELLOW}${rec}${NC}"
        done
    fi

    echo ""
}

# Main analysis function
main() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  UAIP Hub Database Performance Analysis${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    # Start output file
    exec > >(tee -a "$OUTPUT_FILE")

    check_connection

    case "$ANALYSIS_TYPE" in
        slow-queries)
            slow_queries
            ;;
        index-usage)
            index_usage
            ;;
        full)
            database_overview
            table_statistics
            index_usage
            cache_analysis
            bloat_analysis
            slow_queries
            lock_analysis
            generate_recommendations
            ;;
    esac

    echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}✓ Analysis complete!${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "\n${YELLOW}Report saved to: ${NC}${OUTPUT_FILE}"
    echo ""
}

# Run main function
main
