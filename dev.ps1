# Dev Start Script for Windows
# Equivalent to 'make dev' but starts both Backend and Frontend

Write-Host "Starting UAIP Development Environment..." -ForegroundColor Blue

# 1. Start Infrastructure (Docker) & Migrations
Write-Host "1. Initializing Infrastructure..." -ForegroundColor Cyan
./quick-start.ps1

if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to initialize infrastructure." -ForegroundColor Red
    exit 1
}

# 2. Start Backend (Rust)
Write-Host "2. Starting Backend (cargo run)..." -ForegroundColor Cyan
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cargo run --bin uaip-hub"

# 3. Start Frontend (Next.js)
Write-Host "3. Starting Frontend (npm run dev)..." -ForegroundColor Cyan
Set-Location "web"
Start-Process powershell -ArgumentList "-NoExit", "-Command", "npm run dev"

Write-Host ""
Write-Host "Development environment is running!" -ForegroundColor Green
Write-Host "Backend:  http://localhost:8443" -ForegroundColor White
Write-Host "Frontend: http://localhost:3000" -ForegroundColor White
