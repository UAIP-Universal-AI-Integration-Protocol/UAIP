@echo off
REM UAIP Quick Start Script for Windows
REM Equivalent to 'make quick-start'

echo Starting UAIP Hub...
echo.

echo Starting Docker services...
docker-compose -f docker-compose.dev.yml up -d
if errorlevel 1 (
    echo Failed to start Docker services
    exit /b 1
)

echo Waiting for PostgreSQL to be ready...
timeout /t 5 /nobreak >nul

echo Running database migrations...
if exist migrations\001_initial_schema.sql (
    echo   - Applying 001_initial_schema.sql
    type migrations\001_initial_schema.sql | docker exec -i uaip-postgres psql -U uaip -d uaip
)

if exist migrations\002_rbac_tables.sql (
    echo   - Applying 002_rbac_tables.sql
    type migrations\002_rbac_tables.sql | docker exec -i uaip-postgres psql -U uaip -d uaip
)

if exist migrations\003_performance_indexes.sql (
    echo   - Applying 003_performance_indexes.sql
    type migrations\003_performance_indexes.sql | docker exec -i uaip-postgres psql -U uaip -d uaip
)

if exist migrations\004_media_and_streaming.sql (
    echo   - Applying 004_media_and_streaming.sql
    type migrations\004_media_and_streaming.sql | docker exec -i uaip-postgres psql -U uaip -d uaip
)

echo.
echo ========================================
echo   UAIP Hub is ready!
echo ========================================
echo.
echo Services available at:
echo   UAIP Hub API:  http://localhost:8443
echo   Grafana:       http://localhost:3000 (admin/admin)
echo   Prometheus:    http://localhost:9090
echo   PostgreSQL:    localhost:5432 (uaip/uaip_password_dev)
echo   Redis:         localhost:6379
echo   NATS:          localhost:4222
echo.
echo To view logs: docker-compose -f docker-compose.dev.yml logs -f uaip-hub
echo To stop:      docker-compose -f docker-compose.dev.yml down
echo.
