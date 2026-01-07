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

Write-Host "Services started! Schema will be managed by UAIP Hub." -ForegroundColor Green
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
