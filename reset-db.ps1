# Reset Database Script
# WARNING: This deletes all data!

Write-Host "Resetting Database..." -ForegroundColor Red
docker exec -i uaip-postgres psql -U uaip -d postgres -c "DROP DATABASE IF EXISTS uaip WITH (FORCE);"
docker exec -i uaip-postgres psql -U uaip -d postgres -c "CREATE DATABASE uaip;"
Write-Host "Database reset complete. Migrations will run on next app startup." -ForegroundColor Green
