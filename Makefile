# UAIP Hub - Development Makefile
# Simplifies common development tasks

.PHONY: help build test clean docker dev up down logs bench fmt lint check install migrate

# Default target
.DEFAULT_GOAL := help

# Colors for output
BLUE := \033[0;34m
GREEN := \033[0;32m
YELLOW := \033[0;33m
RED := \033[0;31m
NC := \033[0m # No Color

##@ Help

help: ## Display this help message
	@awk 'BEGIN {FS = ":.*##"; printf "\n$(BLUE)Usage:$(NC)\n  make $(GREEN)<target>$(NC)\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  $(GREEN)%-15s$(NC) %s\n", $$1, $$2 } /^##@/ { printf "\n$(YELLOW)%s$(NC)\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

##@ Development

install: ## Install development dependencies
	@echo "$(BLUE)Installing Rust toolchain and dependencies...$(NC)"
	rustup update stable
	rustup component add rustfmt clippy
	cargo install cargo-watch cargo-audit cargo-tarpaulin
	@echo "$(GREEN)✓ Dependencies installed$(NC)"

dev: ## Start development environment with hot reload
	@echo "$(BLUE)Starting development server with hot reload...$(NC)"
	cargo watch -x 'run --bin uaip-hub'

build: ## Build the project in debug mode
	@echo "$(BLUE)Building project...$(NC)"
	cargo build --workspace
	@echo "$(GREEN)✓ Build complete$(NC)"

build-release: ## Build the project in release mode
	@echo "$(BLUE)Building release...$(NC)"
	cargo build --workspace --release
	@echo "$(GREEN)✓ Release build complete$(NC)"

run: ## Run the UAIP Hub
	@echo "$(BLUE)Running UAIP Hub...$(NC)"
	cargo run --bin uaip-hub

##@ Testing

test: ## Run all tests
	@echo "$(BLUE)Running tests...$(NC)"
	cargo test --workspace --lib
	@echo "$(GREEN)✓ All tests passed$(NC)"

test-verbose: ## Run tests with verbose output
	@echo "$(BLUE)Running tests (verbose)...$(NC)"
	cargo test --workspace -- --nocapture

test-watch: ## Run tests in watch mode
	@echo "$(BLUE)Starting test watcher...$(NC)"
	cargo watch -x 'test --workspace'

bench: ## Run benchmarks
	@echo "$(BLUE)Running benchmarks...$(NC)"
	cargo bench
	@echo "$(GREEN)✓ Benchmarks complete$(NC)"
	@echo "$(YELLOW)View report: target/criterion/report/index.html$(NC)"

coverage: ## Generate code coverage report
	@echo "$(BLUE)Generating coverage report...$(NC)"
	cargo tarpaulin --workspace --timeout 300 --out Html --output-dir coverage
	@echo "$(GREEN)✓ Coverage report generated$(NC)"
	@echo "$(YELLOW)View report: coverage/index.html$(NC)"

##@ Code Quality

fmt: ## Format code
	@echo "$(BLUE)Formatting code...$(NC)"
	cargo fmt --all
	@echo "$(GREEN)✓ Code formatted$(NC)"

fmt-check: ## Check code formatting
	@echo "$(BLUE)Checking code format...$(NC)"
	cargo fmt --all -- --check

lint: ## Run clippy linter
	@echo "$(BLUE)Running clippy...$(NC)"
	cargo clippy --workspace --all-targets --all-features -- -D warnings
	@echo "$(GREEN)✓ No clippy warnings$(NC)"

check: ## Run all quality checks
	@echo "$(BLUE)Running all quality checks...$(NC)"
	@make fmt-check
	@make lint
	@make test
	@echo "$(GREEN)✓ All checks passed$(NC)"

audit: ## Security audit
	@echo "$(BLUE)Running security audit...$(NC)"
	cargo audit
	@echo "$(GREEN)✓ Security audit complete$(NC)"

##@ Docker

docker-build: ## Build Docker image
	@echo "$(BLUE)Building Docker image...$(NC)"
	docker build -t uaip-hub:latest .
	@echo "$(GREEN)✓ Docker image built$(NC)"

docker-run: ## Run Docker container
	@echo "$(BLUE)Running Docker container...$(NC)"
	docker run --rm -p 8443:8443 --name uaip-hub uaip-hub:latest

docker-push: ## Push Docker image to registry
	@echo "$(BLUE)Pushing Docker image...$(NC)"
	docker push uaip-hub:latest
	@echo "$(GREEN)✓ Image pushed$(NC)"

##@ Docker Compose

up: ## Start all services (docker-compose up)
	@echo "$(BLUE)Starting all services...$(NC)"
	docker-compose -f docker-compose.dev.yml up -d
	@echo "$(GREEN)✓ Services started$(NC)"
	@echo "$(YELLOW)Access points:$(NC)"
	@echo "  - UAIP Hub:     http://localhost:8443"
	@echo "  - Prometheus:   http://localhost:9090"
	@echo "  - Grafana:      http://localhost:3000 (admin/admin)"
	@echo "  - PostgreSQL:   localhost:5432"
	@echo "  - Redis:        localhost:6379"
	@echo "  - NATS:         localhost:4222"

up-tools: ## Start services with management tools
	@echo "$(BLUE)Starting services with tools...$(NC)"
	docker-compose -f docker-compose.dev.yml --profile tools up -d
	@echo "$(GREEN)✓ Services and tools started$(NC)"
	@echo "  - pgAdmin:      http://localhost:5050"
	@echo "  - Redis Cmdr:   http://localhost:8081"

down: ## Stop all services
	@echo "$(BLUE)Stopping all services...$(NC)"
	docker-compose -f docker-compose.dev.yml down
	@echo "$(GREEN)✓ Services stopped$(NC)"

down-volumes: ## Stop services and remove volumes
	@echo "$(RED)Stopping services and removing volumes...$(NC)"
	docker-compose -f docker-compose.dev.yml down -v
	@echo "$(GREEN)✓ Services stopped and volumes removed$(NC)"

logs: ## View logs from all services
	docker-compose -f docker-compose.dev.yml logs -f

logs-hub: ## View logs from UAIP Hub only
	docker-compose -f docker-compose.dev.yml logs -f uaip-hub

restart: ## Restart all services
	@echo "$(BLUE)Restarting all services...$(NC)"
	docker-compose -f docker-compose.dev.yml restart
	@echo "$(GREEN)✓ Services restarted$(NC)"

ps: ## Show running services
	docker-compose -f docker-compose.dev.yml ps

##@ Database

migrate: ## Run database migrations
	@echo "$(BLUE)Running database migrations...$(NC)"
	docker exec -i uaip-postgres psql -U uaip -d uaip < migrations/001_initial_schema.sql
	docker exec -i uaip-postgres psql -U uaip -d uaip < migrations/002_rbac_tables.sql
	@echo "$(GREEN)✓ Migrations complete$(NC)"

db-shell: ## Open PostgreSQL shell
	@echo "$(BLUE)Opening PostgreSQL shell...$(NC)"
	docker exec -it uaip-postgres psql -U uaip -d uaip

db-reset: ## Reset database (WARNING: destroys data)
	@echo "$(RED)Resetting database...$(NC)"
	docker exec -i uaip-postgres psql -U uaip -d postgres -c "DROP DATABASE IF EXISTS uaip;"
	docker exec -i uaip-postgres psql -U uaip -d postgres -c "CREATE DATABASE uaip;"
	@make migrate
	@echo "$(GREEN)✓ Database reset complete$(NC)"

##@ Kubernetes

k8s-deploy: ## Deploy to Kubernetes
	@echo "$(BLUE)Deploying to Kubernetes...$(NC)"
	kubectl apply -f k8s/namespace.yaml
	kubectl apply -f k8s/configmap.yaml
	kubectl apply -f k8s/serviceaccount.yaml
	kubectl apply -f k8s/deployment.yaml
	kubectl apply -f k8s/service.yaml
	kubectl apply -f k8s/hpa.yaml
	@echo "$(GREEN)✓ Deployed to Kubernetes$(NC)"

k8s-delete: ## Delete from Kubernetes
	@echo "$(RED)Deleting from Kubernetes...$(NC)"
	kubectl delete -f k8s/hpa.yaml
	kubectl delete -f k8s/service.yaml
	kubectl delete -f k8s/deployment.yaml
	kubectl delete -f k8s/serviceaccount.yaml
	kubectl delete -f k8s/configmap.yaml
	@echo "$(GREEN)✓ Deleted from Kubernetes$(NC)"

k8s-status: ## Check Kubernetes deployment status
	@echo "$(BLUE)Checking Kubernetes status...$(NC)"
	kubectl get all -n uaip
	kubectl get pods -n uaip -o wide

k8s-logs: ## View Kubernetes logs
	kubectl logs -n uaip -l app=uaip-hub --tail=100 -f

##@ Monitoring

metrics: ## View Prometheus metrics
	@echo "$(BLUE)Fetching metrics...$(NC)"
	curl -s http://localhost:8443/metrics | head -n 50

health: ## Check health status
	@echo "$(BLUE)Checking health...$(NC)"
	curl -s http://localhost:8443/api/v1/system/health | jq .

watch-health: ## Watch health status
	@echo "$(BLUE)Watching health status...$(NC)"
	watch -n 2 'curl -s http://localhost:8443/api/v1/system/health | jq .'

##@ Cleanup

clean: ## Clean build artifacts
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	cargo clean
	rm -rf target/
	rm -rf coverage/
	@echo "$(GREEN)✓ Cleaned$(NC)"

clean-all: clean down-volumes ## Clean everything including Docker volumes
	@echo "$(GREEN)✓ Everything cleaned$(NC)"

##@ CI/CD

ci: check audit ## Run CI checks locally
	@echo "$(GREEN)✓ All CI checks passed$(NC)"

release: ## Create a release build and Docker image
	@echo "$(BLUE)Creating release...$(NC)"
	@make clean
	@make build-release
	@make docker-build
	@echo "$(GREEN)✓ Release created$(NC)"

##@ Quick Commands

quick-start: up migrate ## Quick start (up + migrate)
	@echo "$(GREEN)✓ UAIP Hub ready!$(NC)"
	@echo "$(YELLOW)Run 'make logs-hub' to view logs$(NC)"

full-check: fmt lint test audit ## Run all checks
	@echo "$(GREEN)✓ All checks passed!$(NC)"

dev-full: clean install up ## Full development setup
	@echo "$(GREEN)✓ Development environment ready!$(NC)"
