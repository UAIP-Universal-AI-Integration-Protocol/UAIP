# UAIP Hub - Universal AI Integration Protocol

> **Status:** Phase 1 - Foundation & MVP (In Progress)
> **Created by:** [Hakille](https://github.com/Hakille)
> **License:** Apache 2.0

## Overview

UAIP (Universal AI Integration Protocol) is a universal protocol that enables AI systems to discover, authenticate, control, and monitor physical IoT devices. This is the Hub Core implementation - the central orchestration platform.

**UAIP is created and owned by Hakille.**

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
# Option 1: Using Docker (Recommended)
docker exec -i uaip-postgres psql -U uaip -d uaip < migrations/001_initial_schema.sql
docker exec -i uaip-postgres psql -U uaip -d uaip < migrations/002_rbac_tables.sql

# Option 2: Using psql directly
psql -U uaip -d uaip -f migrations/001_initial_schema.sql
psql -U uaip -d uaip -f migrations/002_rbac_tables.sql

# Option 3: Using sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run --database-url postgresql://uaip:uaip_password_dev@localhost:5432/uaip

# Verify migrations
docker exec -it uaip-postgres psql -U uaip -d uaip -c "\dt"
docker exec -it uaip-postgres psql -U uaip -d uaip -c "SELECT name FROM roles;"
```

See [migrations/README.md](migrations/README.md) for detailed documentation.

## Configuration

Configuration is managed through:
1. `config/default.toml` - Default configuration
2. `.env` - Environment-specific overrides
3. Environment variables - Runtime overrides

Priority: Environment variables > .env > default.toml

## Project Status

### ✅ Milestone 1.1: Project Scaffold (Week 1) - COMPLETED

- [x] Initialize git repository
- [x] Create Cargo workspace structure (8 crates)
- [x] Implement `uaip-core`: message types, device types, error types (10 tests)
- [x] Docker Compose (PostgreSQL, Redis, NATS)
- [x] Configuration files (.env.example, default.toml)

**Tests:** 10/10 passing ✅

### ✅ Milestone 1.2: Authentication Service (Week 2) - COMPLETED

- [x] JWT generation/validation with OAuth 2.0 (8 tests)
- [x] X.509 certificate parsing and validation (5 tests)
- [x] Challenge-response authentication flow
- [x] RBAC implementation with 5 default roles (11 tests)
- [x] Database schema and migrations (10 tables, 3 functions, 1 trigger)

**Tests:** 24/24 passing ✅
**Database:** 10 tables, 5 roles, 12 permissions configured

**Features:**
- OAuth 2.0 client_credentials flow for AI agents
- X.509 certificate-based device authentication
- Role-based access control (admin, device_manager, device_operator, monitor, ai_agent)
- Certificate revocation list (CRL)
- Token refresh mechanism
- Wildcard permission matching (*:*)
- PostgreSQL stored functions for permission checking

### 🚧 Milestone 1.3: Device Registry (Week 3) - IN PROGRESS

- [ ] PostgreSQL device repository
- [ ] Device CRUD operations
- [ ] Device registration workflow (3-step challenge)
- [ ] Device status tracking (online/offline/error)
- [ ] Capability storage and querying
- [ ] Redis caching for device states

### 📋 Upcoming Milestones

- Milestone 1.4: Message Router (Week 4)
- Milestone 1.5: REST API (Week 5)
- Milestone 1.6: WebSocket API (Week 6)

See the [implementation plan](.claude/plans/polished-tumbling-bumblebee.md) for complete roadmap.

### 📊 Overall Progress

**Phase 1 - Foundation & MVP:** 33% complete (2/6 milestones)

| Component | Status | Tests |
|-----------|--------|-------|
| Core Types | ✅ Complete | 10/10 |
| Authentication | ✅ Complete | 24/24 |
| Device Registry | 🚧 In Progress | 0/? |
| Message Router | ⏳ Pending | 0/? |
| REST API | ⏳ Pending | 0/? |
| WebSocket API | ⏳ Pending | 0/? |

**Total Tests:** 34/34 passing ✅

## API Documentation

API documentation will be available at `/api/docs` once the REST API is implemented.

## Security

- **TLS 1.3:** All connections encrypted (production)
- **JWT Authentication:** For AI agents (OAuth 2.0)
- **X.509 Certificates:** For device authentication
- **AES-256-GCM:** End-to-end payload encryption
- **RBAC:** Role-based access control

## Contributing

We welcome contributions! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

By contributing, you agree that your contributions will be licensed under the Apache 2.0 License.

## License

Copyright 2024-2025 Hakille and UAIP Contributors

UAIP (Universal AI Integration Protocol) is created and owned by **Hakille**.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the full license text.

## Authors & Acknowledgments

- **Hakille** - Protocol Creator & Original Author
- **Claude Sonnet 4.5** - AI Development Assistant
- See [CONTRIBUTORS.md](CONTRIBUTORS.md) for all contributors

## Contact

- **GitHub:** [@Hakille](https://github.com/Hakille)
- **Project Repository:** [github.com/Hakille/UAIP](https://github.com/Hakille/UAIP)
- **Issues:** [github.com/Hakille/UAIP/issues](https://github.com/Hakille/UAIP/issues)

---

**Made with ❤️ by Hakille**
