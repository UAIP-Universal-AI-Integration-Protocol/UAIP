#!/bin/bash
# Database Backup Script for UAIP Hub
# Backs up PostgreSQL database to a timestamped file

set -euo pipefail

# Configuration
BACKUP_DIR="${BACKUP_DIR:-./backups}"
DB_NAME="${DB_NAME:-uaip}"
DB_USER="${DB_USER:-uaip}"
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
RETENTION_DAYS="${RETENTION_DAYS:-7}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Generate timestamp
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/${DB_NAME}_${TIMESTAMP}.sql.gz"

echo -e "${BLUE}Starting database backup...${NC}"
echo -e "Database: ${YELLOW}${DB_NAME}${NC}"
echo -e "Host: ${YELLOW}${DB_HOST}:${DB_PORT}${NC}"
echo -e "Backup file: ${YELLOW}${BACKUP_FILE}${NC}"

# Perform backup
echo -e "${BLUE}Dumping database...${NC}"
if pg_dump -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" | gzip > "$BACKUP_FILE"; then
    BACKUP_SIZE=$(du -h "$BACKUP_FILE" | cut -f1)
    echo -e "${GREEN}✓ Backup completed successfully${NC}"
    echo -e "Backup size: ${YELLOW}${BACKUP_SIZE}${NC}"
else
    echo -e "${RED}✗ Backup failed${NC}"
    exit 1
fi

# Clean up old backups
echo -e "${BLUE}Cleaning up old backups (older than ${RETENTION_DAYS} days)...${NC}"
DELETED_COUNT=$(find "$BACKUP_DIR" -name "${DB_NAME}_*.sql.gz" -type f -mtime +${RETENTION_DAYS} -delete -print | wc -l)
if [ "$DELETED_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓ Deleted ${DELETED_COUNT} old backup(s)${NC}"
else
    echo -e "${YELLOW}No old backups to delete${NC}"
fi

# List recent backups
echo -e "\n${BLUE}Recent backups:${NC}"
ls -lht "$BACKUP_DIR"/${DB_NAME}_*.sql.gz | head -5 | awk '{print "  " $9 " (" $5 ")"}'

echo -e "\n${GREEN}✓ Backup process complete${NC}"
