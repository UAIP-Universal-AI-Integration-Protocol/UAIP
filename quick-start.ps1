# UAIP Quick Start Script for Windows
# Equivalent to 'make quick-start'

Write-Host "Starting UAIP Hub..." -ForegroundColor Blue

# Start Docker services
Write-Host "Starting Docker services..." -ForegroundColor Cyan
docker-compose -f docker-compose.dev.yml up -d

if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to start Docker services" -ForegroundColor Red
    exit 1
}

Write-Host "Waiting for PostgreSQL to be ready..." -ForegroundColor Cyan
Start-Sleep -Seconds 5

# Run database migrations
Write-Host "Running database migrations..." -ForegroundColor Cyan

$migrations = @(
    "migrations/001_initial_schema.sql",
    "migrations/002_rbac_tables.sql",
    "migrations/003_performance_indexes.sql",
    "migrations/004_media_and_streaming.sql"
)

foreach ($migration in $migrations) {
    if (Test-Path $migration) {
        Write-Host "  -> Applying $migration" -ForegroundColor Gray
        Get-Content $migration | docker exec -i uaip-postgres psql -U uaip -d uaip
        if ($LASTEXITCODE -ne 0) {
            Write-Host "Failed to apply migration: $migration" -ForegroundColor Red
            exit 1
        }
    }
}

Write-Host ""
Write-Host "UAIP Hub is ready!" -ForegroundColor Green
Write-Host ""
Write-Host "Services available at:" -ForegroundColor Yellow
Write-Host "  UAIP Hub API:  http://localhost:8443" -ForegroundColor White
Write-Host "  Grafana:       http://localhost:3000 (admin/admin)" -ForegroundColor White
Write-Host "  Prometheus:    http://localhost:9090" -ForegroundColor White
Write-Host "  PostgreSQL:    localhost:5432 (uaip/uaip_password_dev)" -ForegroundColor White
Write-Host "  Redis:         localhost:6379" -ForegroundColor White
Write-Host "  NATS:          localhost:4222" -ForegroundColor White
Write-Host ""
Write-Host "To view logs: docker-compose -f docker-compose.dev.yml logs -f uaip-hub" -ForegroundColor Yellow
Write-Host "To stop:      docker-compose -f docker-compose.dev.yml down" -ForegroundColor Yellow
