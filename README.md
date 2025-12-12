# UAIP Hub - Universal AI Integration Protocol

> **Status:** Phase 1 - Foundation & MVP (In Progress)

## Overview

UAIP (Universal AI Integration Protocol) is a universal protocol that enables AI systems to discover, authenticate, control, and monitor physical IoT devices. This is the Hub Core implementation - the central orchestration platform.

## Architecture

The UAIP Hub is built as a modular monolith in Rust, designed to scale horizontally and eventually split into microservices.

### Crates Structure

```
uaip-hub/
├── uaip-core          # Core types, message formats, errors
├── uaip-auth          # JWT, X.509, RBAC authentication
├── uaip-registry      # Device registry & discovery
├── uaip-router        # Message routing, QoS, priority queue
├── uaip-security      # Encryption (AES-256-GCM), TLS
├── uaip-orchestrator  # AI orchestration, rule engine
├── uaip-adapters      # Protocol adapters (MQTT, HTTP, WebSocket)
└── uaip-hub           # Main hub service (binary)
```

## Technology Stack

- **Language:** Rust 1.70+
- **Web Framework:** Axum 0.7
- **Database:** PostgreSQL 16
- **Cache:** Redis 7
- **Message Queue:** NATS 2.10
- **Security:** TLS 1.3, JWT, X.509 certificates, AES-256-GCM

## Prerequisites

- Rust 1.70 or higher
- Docker & Docker Compose
- PostgreSQL 16 (via Docker)
- Redis 7 (via Docker)
- NATS 2.10 (via Docker)

## Quick Start

### 1. Clone and Setup

```bash
git clone <repository-url>
cd UAIP
cp .env.example .env
```

### 2. Start Infrastructure

```bash
# Start PostgreSQL, Redis, and NATS
docker-compose up -d

# Verify services are running
docker-compose ps
```

### 3. Build and Run

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Run the hub (once implemented)
cargo run --bin uaip-hub
```

### 4. Access Services

- **UAIP Hub API:** https://localhost:8443 (when running)
- **PostgreSQL:** localhost:5432
- **Redis:** localhost:6379
- **NATS:** localhost:4222
- **NATS Monitoring:** http://localhost:8222
- **Prometheus:** http://localhost:9090 (with monitoring profile)
- **Grafana:** http://localhost:3000 (with monitoring profile)

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p uaip-core

# Run tests with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Check compilation without building
cargo check
```

### Database Migrations

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run --database-url postgresql://uaip:uaip_password_dev@localhost:5432/uaip
```

## Configuration

Configuration is managed through:
1. `config/default.toml` - Default configuration
2. `.env` - Environment-specific overrides
3. Environment variables - Runtime overrides

Priority: Environment variables > .env > default.toml

## Project Status

### ✅ Milestone 1.1: Scaffold Projet (Week 1) - COMPLETED

- [x] Initialize git repository
- [x] Create Cargo workspace structure (8 crates)
- [x] Implement `uaip-core`: message types, device types, error types
- [x] Docker Compose (PostgreSQL, Redis, NATS)
- [x] Configuration files (.env.example, default.toml)

### 🚧 Milestone 1.2: Service Authentification (Week 2) - TODO

- [ ] JWT generation/validation
- [ ] X.509 certificate parsing
- [ ] Challenge-response authentication
- [ ] RBAC implementation
- [ ] Database models for auth

### 📋 Upcoming Milestones

- Milestone 1.3: Device Registry (Week 3)
- Milestone 1.4: Message Router (Week 4)
- Milestone 1.5: REST API (Week 5)
- Milestone 1.6: WebSocket API (Week 6)

See the [implementation plan](../.claude/plans/polished-tumbling-bumblebee.md) for complete roadmap.

## API Documentation

API documentation will be available at `/api/docs` once the REST API is implemented.

## Security

- **TLS 1.3:** All connections encrypted (production)
- **JWT Authentication:** For AI agents (OAuth 2.0)
- **X.509 Certificates:** For device authentication
- **AES-256-GCM:** End-to-end payload encryption
- **RBAC:** Role-based access control

## Contributing

This is a specification-to-implementation project. Follow the phased approach in the implementation plan.

## License

[To be determined]

## Contact

[To be determined]
