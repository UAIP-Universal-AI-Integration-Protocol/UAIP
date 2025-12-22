#!/bin/bash
# Database Restore Script for UAIP Hub
# Restores PostgreSQL database from a backup file

set -euo pipefail

# Configuration
DB_NAME="${DB_NAME:-uaip}"
DB_USER="${DB_USER:-uaip}"
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Check if backup file is provided
if [ $# -eq 0 ]; then
    echo -e "${RED}Error: No backup file specified${NC}"
    echo "Usage: $0 <backup_file.sql.gz>"
    echo ""
    echo "Available backups:"
    ls -lht backups/${DB_NAME}_*.sql.gz 2>/dev/null | head -5 | awk '{print "  " $9 " (" $5 ")"}' || echo "  No backups found"
    exit 1
fi

BACKUP_FILE="$1"

# Check if backup file exists
if [ ! -f "$BACKUP_FILE" ]; then
    echo -e "${RED}Error: Backup file not found: ${BACKUP_FILE}${NC}"
    exit 1
fi

echo -e "${BLUE}Database Restore${NC}"
echo -e "Database: ${YELLOW}${DB_NAME}${NC}"
echo -e "Host: ${YELLOW}${DB_HOST}:${DB_PORT}${NC}"
echo -e "Backup file: ${YELLOW}${BACKUP_FILE}${NC}"
echo ""

# Confirm restore operation
echo -e "${RED}WARNING: This will DROP and recreate the database!${NC}"
echo -e "All existing data will be lost."
read -p "Are you sure you want to continue? (yes/no): " CONFIRM

if [ "$CONFIRM" != "yes" ]; then
    echo -e "${YELLOW}Restore cancelled${NC}"
    exit 0
fi

# Drop existing database
echo -e "\n${BLUE}Dropping existing database...${NC}"
if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c "DROP DATABASE IF EXISTS ${DB_NAME};"; then
    echo -e "${GREEN}✓ Database dropped${NC}"
else
    echo -e "${RED}✗ Failed to drop database${NC}"
    exit 1
fi

# Create new database
echo -e "${BLUE}Creating new database...${NC}"
if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c "CREATE DATABASE ${DB_NAME};"; then
    echo -e "${GREEN}✓ Database created${NC}"
else
    echo -e "${RED}✗ Failed to create database${NC}"
    exit 1
fi

# Restore from backup
echo -e "${BLUE}Restoring from backup...${NC}"
if gunzip -c "$BACKUP_FILE" | psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Restore completed successfully${NC}"
else
    echo -e "${RED}✗ Restore failed${NC}"
    exit 1
fi

# Verify restoration
echo -e "${BLUE}Verifying restoration...${NC}"
TABLE_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'public';" | xargs)
echo -e "Tables restored: ${YELLOW}${TABLE_COUNT}${NC}"

echo -e "\n${GREEN}✓ Restore process complete${NC}"
