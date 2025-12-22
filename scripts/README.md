# UAIP Hub Automation Scripts

This directory contains production-grade automation scripts for managing, monitoring, and deploying the UAIP Hub.

## Overview

All scripts are designed with **Google/Apple level engineering standards** in mind:
- Comprehensive error handling
- Colored output for better UX
- Detailed logging
- Configurable via environment variables
- Production-ready with safety checks

## Scripts

### 1. `backup-database.sh` - Database Backup

**Purpose:** Automated PostgreSQL database backups with rotation

**Features:**
- Timestamped backup files (gzip compressed)
- Automatic cleanup of old backups (configurable retention)
- Backup size reporting
- Recent backups listing

**Usage:**
```bash
# Basic usage (defaults)
./scripts/backup-database.sh

# Custom configuration
BACKUP_DIR=/mnt/backups \
DB_NAME=uaip \
DB_HOST=db.example.com \
RETENTION_DAYS=30 \
./scripts/backup-database.sh
```

**Configuration:**
- `BACKUP_DIR` - Backup directory (default: `./backups`)
- `DB_NAME` - Database name (default: `uaip`)
- `DB_USER` - Database user (default: `uaip`)
- `DB_HOST` - Database host (default: `localhost`)
- `DB_PORT` - Database port (default: `5432`)
- `RETENTION_DAYS` - Days to keep backups (default: `7`)

**Output:**
```
Starting database backup...
Database: uaip
Host: localhost:5432
Backup file: ./backups/uaip_20250122_143022.sql.gz
✓ Backup completed successfully
Backup size: 1.2M
```

**Scheduling:**
```bash
# Add to crontab for daily backups at 2 AM
0 2 * * * /path/to/scripts/backup-database.sh
```

---

### 2. `restore-database.sh` - Database Restore

**Purpose:** Restore PostgreSQL database from backup files

**Features:**
- Safety confirmation before destructive operation
- Automatic database recreation
- Post-restore verification
- Lists available backups

**Usage:**
```bash
# List available backups
./scripts/restore-database.sh

# Restore from specific backup
./scripts/restore-database.sh backups/uaip_20250122_143022.sql.gz

# Custom configuration
DB_HOST=db.example.com \
./scripts/restore-database.sh backups/uaip_20250122_143022.sql.gz
```

**⚠️ Warning:** This script will **DROP and recreate** the database, destroying all existing data!

**Configuration:**
- `DB_NAME` - Database name (default: `uaip`)
- `DB_USER` - Database user (default: `uaip`)
- `DB_HOST` - Database host (default: `localhost`)
- `DB_PORT` - Database port (default: `5432`)

**Output:**
```
Database Restore
Database: uaip
Host: localhost:5432
Backup file: backups/uaip_20250122_143022.sql.gz

WARNING: This will DROP and recreate the database!
All existing data will be lost.
Are you sure you want to continue? (yes/no): yes

✓ Database dropped
✓ Database created
✓ Restore completed successfully
Tables restored: 15
```

---

### 3. `health-monitor.sh` - Health Monitoring

**Purpose:** Continuous health monitoring with alerting

**Features:**
- Real-time health status tracking
- Component-level monitoring (PostgreSQL, Redis, NATS)
- Consecutive failure tracking
- Success rate statistics
- Configurable alert thresholds
- Detailed logging

**Usage:**
```bash
# Basic usage (monitor localhost)
./scripts/health-monitor.sh

# Custom configuration
UAIP_URL=https://uaip.example.com \
CHECK_INTERVAL=30 \
./scripts/health-monitor.sh
```

**Configuration:**
- `UAIP_URL` - UAIP Hub URL (default: `http://localhost:8443`)
- `CHECK_INTERVAL` - Seconds between checks (default: `10`)
- `ALERT_ON_FAILURE` - Enable alerting (default: `true`)
- `LOG_FILE` - Log file path (default: `/tmp/uaip-health-monitor.log`)

**Output:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  UAIP Hub Health Monitor
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
URL: http://localhost:8443
Check interval: 10s
Log file: /tmp/uaip-health-monitor.log
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Check #1 - 2025-01-22 14:30:45
  Overall Status: ✓ HEALTHY
  PostgreSQL:     ✓ healthy
  Redis:          ✓ healthy
  NATS:           ✓ healthy
  Success rate:   100.00% (0/1 failures)
```

**Alerting:**
The script includes hooks for custom alerting (email, Slack, PagerDuty). Edit the `send_alert()` function to integrate with your alerting system:

```bash
send_alert() {
    local message=$1
    # Example: Send to Slack
    curl -X POST https://hooks.slack.com/services/YOUR/WEBHOOK/URL \
         -H 'Content-Type: application/json' \
         -d "{\"text\": \"$message\"}"
}
```

**Running as Service:**
```bash
# systemd service example
sudo tee /etc/systemd/system/uaip-health-monitor.service <<EOF
[Unit]
Description=UAIP Hub Health Monitor
After=network.target

[Service]
Type=simple
User=uaip
Environment="UAIP_URL=http://localhost:8443"
ExecStart=/path/to/scripts/health-monitor.sh
Restart=always

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable uaip-health-monitor
sudo systemctl start uaip-health-monitor
```

---

### 4. `load-test.sh` - Load Testing

**Purpose:** Comprehensive load testing and performance analysis

**Features:**
- Multiple endpoint testing
- Mixed workload simulation
- Automatic report generation
- Support for multiple tools (hey, ab, curl)
- Pre-flight health check
- Real-time metrics display

**Usage:**
```bash
# Basic usage (30s, 10 concurrent)
./scripts/load-test.sh

# Custom configuration
DURATION=60 \
CONCURRENCY=50 \
REQUEST_RATE=1000 \
./scripts/load-test.sh
```

**Configuration:**
- `UAIP_URL` - UAIP Hub URL (default: `http://localhost:8443`)
- `DURATION` - Test duration in seconds (default: `30`)
- `CONCURRENCY` - Concurrent connections (default: `10`)
- `REQUEST_RATE` - Target requests per second (default: `100`)
- `OUTPUT_DIR` - Output directory (default: `./load-test-results`)

**Output:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  UAIP Hub Load Testing
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
URL: http://localhost:8443
Duration: 30s
Concurrency: 10
Target rate: 100 req/s
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Testing: GET /api/v1/system/health
  Using 'hey' for load testing...

Summary:
  Total:        30.0015 secs
  Slowest:      0.0521 secs
  Fastest:      0.0012 secs
  Average:      0.0087 secs
  Requests/sec: 99.95

Response time histogram:
  0.001 [1]     |
  0.006 [2134]  |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.011 [512]   |■■■■■■■
```

**Recommended Tool:**
For best results, install `hey`:
```bash
go install github.com/rakyll/hey@latest
```

---

### 5. `deploy.sh` - Deployment Automation

**Purpose:** Automated deployment to multiple environments

**Features:**
- Multi-environment support (dev, staging, prod)
- Docker Compose deployment (dev)
- Kubernetes deployment (staging/prod)
- Health check verification
- Rollout monitoring
- Production safety checks

**Usage:**
```bash
# Deploy to development
./scripts/deploy.sh dev

# Deploy to staging with specific version
VERSION=v1.2.3 ./scripts/deploy.sh staging

# Deploy to production (with confirmation)
VERSION=v1.2.3 \
DOCKER_REGISTRY=registry.example.com \
./scripts/deploy.sh prod
```

**Configuration:**
- `VERSION` - Docker image version (default: `latest`)
- `NAMESPACE` - Kubernetes namespace (default: `uaip`)
- `DOCKER_REGISTRY` - Docker registry URL
- `HEALTH_CHECK_TIMEOUT` - Health check timeout in seconds (default: `300`)

**Output:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  UAIP Hub Deployment
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Environment: staging
Version: v1.2.3
Namespace: uaip
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[14:30:15] Deploying to staging environment...
[14:30:15] Building Docker image: registry.example.com/uaip-hub:v1.2.3
[14:30:45] Pushing Docker image...
[14:31:20] Applying Kubernetes manifests...
[14:31:25] Watching rollout status...
[14:32:10] ✓ Rollout completed successfully
[14:32:11] ✓ All pods are ready (3/3)
```

**Environments:**
- **dev** - Local development via docker-compose
  - Builds images locally
  - Runs migrations automatically
  - No registry push required
- **staging** - Kubernetes staging environment
  - Builds and pushes to registry
  - Applies K8s manifests
  - Monitors rollout
- **prod** - Kubernetes production environment
  - Same as staging but with safety confirmation
  - Production context verification

---

### 6. `analyze-logs.sh` - Log Analysis

**Purpose:** Comprehensive log analysis and insights

**Features:**
- Multiple log sources (Docker, Kubernetes, files)
- Detailed statistics (errors, warnings, status codes)
- Performance analysis (response times, slow requests)
- Security event detection
- Database activity tracking
- Automated recommendations

**Usage:**
```bash
# Analyze Docker logs
./scripts/analyze-logs.sh

# Analyze Kubernetes logs (last 6 hours)
LOG_SOURCE=k8s TIME_RANGE=6h ./scripts/analyze-logs.sh

# Analyze log file
LOG_SOURCE=file LOG_FILE=/var/log/uaip.log ./scripts/analyze-logs.sh
```

**Configuration:**
- `LOG_SOURCE` - Log source: `docker`, `k8s`, or `file` (default: `docker`)
- `LOG_FILE` - Path to log file (for `LOG_SOURCE=file`)
- `NAMESPACE` - Kubernetes namespace (default: `uaip`)
- `TIME_RANGE` - Time range for k8s logs (default: `1h`)
- `OUTPUT_FILE` - Output file for analysis results

**Output:**
```
UAIP Hub Log Analysis Report
Generated: 2025-01-22 14:35:12
Source: docker
Time Range: 1h

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

SUMMARY
Total log entries: 15432

LOG LEVELS:
───────────────────────────────────────────────────────
INFO        12345 (80.00%)
WARN          234 ( 1.52%)
ERROR          45 ( 0.29%)
DEBUG        2808 (18.19%)

ERRORS:
───────────────────────────────────────────────────────
Total errors: 45

Top error messages:
     12 Database connection timeout
      8 Redis connection failed
      5 Invalid authentication token

HTTP STATUS CODES:
───────────────────────────────────────────────────────
200:  14523 requests
404:    234 requests
500:     45 requests

TOP ENDPOINTS:
───────────────────────────────────────────────────────
/api/v1/system/health                              12345
/metrics                                            1234
/api/v1/devices                                      234

RESPONSE TIMES:
───────────────────────────────────────────────────────
Average: 12.34ms
Min: 0.52ms
Max: 1234.56ms

PERFORMANCE ISSUES:
───────────────────────────────────────────────────────
Slow requests (>1s): 23

RECOMMENDATIONS:
───────────────────────────────────────────────────────
⚠ Performance degradation detected (23 slow requests)
  - Review database query performance
  - Check resource utilization
```

---

## Best Practices

### Security

1. **Never commit credentials** to scripts
2. **Use environment variables** for sensitive configuration
3. **Restrict script permissions**: `chmod 700 scripts/*.sh`
4. **Use separate credentials** for production vs development
5. **Enable audit logging** for all script executions

### Production Deployment

```bash
# 1. Backup database before deployment
./scripts/backup-database.sh

# 2. Run load tests in staging
UAIP_URL=https://staging.example.com ./scripts/load-test.sh

# 3. Analyze logs for issues
LOG_SOURCE=k8s NAMESPACE=staging ./scripts/analyze-logs.sh

# 4. Deploy to production
VERSION=v1.2.3 ./scripts/deploy.sh prod

# 5. Monitor health post-deployment
UAIP_URL=https://uaip.example.com ./scripts/health-monitor.sh
```

### Monitoring

```bash
# Set up continuous health monitoring
UAIP_URL=https://uaip.example.com \
CHECK_INTERVAL=60 \
ALERT_ON_FAILURE=true \
./scripts/health-monitor.sh &

# Set up automated backups (crontab)
0 2 * * * /path/to/scripts/backup-database.sh
0 3 * * 0 /path/to/scripts/analyze-logs.sh
```

### Error Handling

All scripts:
- Exit with non-zero code on errors
- Use `set -euo pipefail` for strict error handling
- Provide detailed error messages
- Log all operations for audit trails

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Deploy to Production

on:
  push:
    tags:
      - 'v*'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Backup database
        run: ./scripts/backup-database.sh
        env:
          DB_HOST: ${{ secrets.PROD_DB_HOST }}
          DB_USER: ${{ secrets.PROD_DB_USER }}

      - name: Deploy to production
        run: ./scripts/deploy.sh prod
        env:
          VERSION: ${{ github.ref_name }}
          DOCKER_REGISTRY: ${{ secrets.DOCKER_REGISTRY }}

      - name: Health check
        run: ./scripts/health-monitor.sh
        env:
          UAIP_URL: https://uaip.example.com
          CHECK_INTERVAL: 10
        timeout-minutes: 5
```

## Troubleshooting

### Script Permission Errors

```bash
# Make scripts executable
chmod +x scripts/*.sh
```

### Database Connection Issues

```bash
# Test connection
psql -h localhost -U uaip -d uaip -c "SELECT 1"

# Check environment variables
echo $DB_HOST $DB_USER $DB_NAME
```

### Kubernetes Deployment Failures

```bash
# Check kubectl context
kubectl config current-context

# Verify secrets exist
kubectl get secrets -n uaip

# Check pod status
kubectl get pods -n uaip
kubectl describe pod <pod-name> -n uaip
```

## Contributing

When adding new scripts:

1. Follow the existing pattern (error handling, colors, configuration)
2. Add comprehensive usage documentation
3. Include examples in this README
4. Test in multiple environments
5. Add error messages for common failure modes

## Support

For issues or questions:
- GitHub Issues: https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP/issues
- Documentation: https://docs.uaip.io
