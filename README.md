<div align="center">

<!-- Logo will be added here by Hakille -->

# UAIP Hub
### Universal AI Integration Protocol

**The Enterprise-Grade IoT & AI Integration Platform**

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/Tests-159%2F159-brightgreen.svg)](#test-coverage)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Docker](https://img.shields.io/badge/Docker-Ready-blue.svg)](docker-compose.dev.yml)
[![Status](https://img.shields.io/badge/Status-Production%20Ready-success.svg)](#project-status)

**Created by [Hakille](https://github.com/Hakille)** | Built with Google/Apple Engineering Standards

[Features](#-key-features) • [Quick Start](#-quick-start) • [Documentation](#-documentation) • [Architecture](#-architecture) • [Contributing](#-contributing)

</div>

---

## 🌟 What is UAIP?

**UAIP (Universal AI Integration Protocol)** is a production-ready platform that enables AI systems to seamlessly discover, authenticate, control, and monitor IoT devices at scale. Built in Rust for performance and safety, UAIP provides enterprise-grade features for real-time device orchestration.

### Why UAIP?

- 🚀 **Production Ready**: 159 tests, zero warnings, battle-tested code
- 🔒 **Security First**: TLS 1.3, JWT, X.509, AES-256-GCM encryption
- ⚡ **High Performance**: Rust-powered, async/await, horizontal scaling
- 🎯 **Developer Friendly**: 5-minute setup, comprehensive docs, Docker ready
- 🔧 **Complete Solution**: Authentication, routing, orchestration, monitoring
- 📊 **Observable**: Built-in Prometheus metrics, Grafana dashboards

---

## ✨ Key Features

### 🔐 **Security & Authentication**
- **Multi-Method Auth**: JWT (OAuth 2.0) for AI agents, X.509 for devices
- **Enterprise RBAC**: Fine-grained role-based access control
- **AES-256-GCM Encryption**: Authenticated encryption for sensitive data
- **TLS 1.3**: Modern, secure transport layer

### 🤖 **AI Orchestration**
- **Rule Engine**: JSON-based automation with 11 operators
- **Workflow Engine**: State machine for multi-step automation
- **Scenario Engine**: High-level automation patterns
- **Priority Queue**: Critical > High > Normal > Low routing

### 🌐 **Protocol Support**
- **HTTP/REST**: Full REST API with retry and multiple auth methods
- **WebSocket**: Real-time bidirectional communication
- **MQTT**: IoT device communication with QoS 0/1/2
- **NATS**: High-performance message queue

### 📡 **Device Management**
- **Auto Discovery**: Automatic device registration
- **Health Monitoring**: Real-time heartbeat tracking
- **Capability System**: Dynamic device feature queries
- **State Caching**: Redis-backed performance optimization

### 📊 **Observability**
- **Structured Logging**: Request tracing and correlation
- **Prometheus Metrics**: 33-panel Grafana dashboards
- **Health Checks**: Database, Redis, NATS verification
- **Performance Monitoring**: Real-time system insights

---

## 🚀 Quick Start

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **Docker & Docker Compose** - [Install Docker](https://docs.docker.com/get-docker/)

### One-Command Installation

```bash
# Clone the repository
git clone https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP.git
cd UAIP

# Start everything (infrastructure + migrations + hub)
make quick-start
```

**That's it!** 🎉 Your UAIP Hub is now running.

### Access Your Services

| Service | URL | Credentials |
|---------|-----|-------------|
| 🏠 UAIP Hub API | http://localhost:8443 | - |
| 📊 Grafana | http://localhost:3000 | admin / admin |
| 📈 Prometheus | http://localhost:9090 | - |
| 🗄️ PostgreSQL | localhost:5432 | uaip / uaip_password_dev |
| 🔴 Redis | localhost:6379 | - |
| 📨 NATS | localhost:4222 | - |

---

## 📚 Documentation

### Core Concepts

<details>
<summary><b>Authentication Flow</b></summary>

```rust
// AI Agent Authentication (OAuth 2.0)
POST /api/v1/auth/login
{
  "grant_type": "client_credentials",
  "client_id": "ai_agent_001",
  "client_secret": "secret",
  "scope": "devices:read devices:write"
}

// Response
{
  "access_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGc..."
}

// Device Authentication (X.509)
POST /api/v1/devices/register
{
  "device_id": "sensor-001",
  "certificate": "-----BEGIN CERTIFICATE-----..."
}
```

</details>

<details>
<summary><b>Device Control</b></summary>

```rust
// Send Command to Device
POST /api/v1/devices/sensor-001/command
Authorization: Bearer <access_token>
{
  "action": "set_temperature",
  "parameters": {
    "target": 22.5,
    "unit": "celsius"
  }
}

// List Devices with Filtering
GET /api/v1/devices?status=online&manufacturer=acme&limit=50

// Query Device Capabilities
GET /api/v1/devices/sensor-001/capabilities
```

</details>

<details>
<summary><b>Automation Rules</b></summary>

```json
{
  "id": "temp_alert_001",
  "name": "High Temperature Alert",
  "enabled": true,
  "conditions": [
    {
      "field": "temperature",
      "operator": "greater_than",
      "value": 30.0,
      "device_id": "sensor-001"
    }
  ],
  "actions": [
    {
      "type": "send_notification",
      "parameters": {
        "message": "Temperature exceeded threshold!",
        "severity": "high"
      }
    }
  ],
  "priority": 1,
  "cooldown_seconds": 300
}
```

</details>

<details>
<summary><b>WebSocket Real-Time Events</b></summary>

```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:8443/ws');

// Subscribe to device events
ws.send(JSON.stringify({
  type: 'subscribe',
  topics: ['devices.sensor-001.telemetry']
}));

// Receive telemetry
ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  if (message.type === 'telemetry') {
    console.log('Temperature:', message.data.temperature);
  }
};
```

</details>

---

## 🏗️ Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      UAIP Hub Core                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │   REST   │  │WebSocket │  │   MQTT   │  │   HTTP   │  │
│  │   API    │  │  Server  │  │  Broker  │  │  Client  │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  │
│       │             │             │             │         │
│  ┌────┴─────────────┴─────────────┴─────────────┴─────┐  │
│  │           Authentication & Authorization           │  │
│  │        (JWT, X.509, RBAC, OAuth 2.0)              │  │
│  └────────────────────┬───────────────────────────────┘  │
│                       │                                   │
│  ┌────────────────────┴───────────────────────────────┐  │
│  │              Orchestration Layer                   │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐        │  │
│  │  │  Rules   │  │Workflows │  │Scenarios │        │  │
│  │  │  Engine  │  │  Engine  │  │  Engine  │        │  │
│  │  └──────────┘  └──────────┘  └──────────┘        │  │
│  └────────────────────┬───────────────────────────────┘  │
│                       │                                   │
│  ┌────────────────────┴───────────────────────────────┐  │
│  │              Message Router                         │  │
│  │    (Priority Queue, QoS 0/1/2, NATS Pub/Sub)      │  │
│  └────────────────────┬───────────────────────────────┘  │
│                       │                                   │
│  ┌────────────────────┴───────────────────────────────┐  │
│  │              Device Registry                        │  │
│  │         (PostgreSQL + Redis Cache)                 │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Crate Structure

```
uaip-hub/
├── 📦 uaip-core           # Core types, messages, errors
├── 🔐 uaip-auth           # JWT, X.509, RBAC
├── 📋 uaip-registry       # Device registry & discovery
├── 🔀 uaip-router         # Message routing & QoS
├── 🔒 uaip-security       # AES-256-GCM encryption, TLS
├── 🤖 uaip-orchestrator   # Rules, Workflows, Scenarios
├── 🔌 uaip-adapters       # MQTT, HTTP, WebSocket clients
└── 🏠 uaip-hub            # Main hub binary
```

**Design Philosophy:**
- **Modular Monolith**: Start simple, scale when needed
- **Horizontal Scaling**: Stateless design for easy replication
- **Microservices Ready**: Crates can split into services
- **Production First**: Security, observability, reliability built-in

---

## 🧪 Test Coverage

```bash
# Run all tests (159 tests)
make test

# Run specific crate tests
cargo test -p uaip-core
cargo test -p uaip-auth
cargo test -p uaip-orchestrator

# Check code quality
make lint        # Clippy with zero warnings
make format      # rustfmt
```

### Test Breakdown

| Crate | Tests | Status |
|-------|-------|--------|
| uaip-core | 10 | ✅ |
| uaip-auth | 24 | ✅ |
| uaip-registry | 17 | ✅ |
| uaip-router | 17 | ✅ |
| uaip-hub | 35 | ✅ |
| uaip-orchestrator | 23 | ✅ |
| uaip-adapters | 17 | ✅ |
| uaip-security | 16 | ✅ |
| **Total** | **159** | **✅** |

---

## 🛠️ Development

### Available Commands

```bash
# Development
make dev              # Start dev environment
make build            # Build all crates
make test             # Run all tests
make watch            # Watch mode (auto-rebuild)

# Database
make db-migrate       # Run migrations
make db-reset         # Reset database
make db-seed          # Seed test data

# Docker
make docker-build     # Build Docker image
make docker-run       # Run in Docker
make docker-logs      # View logs

# Production
make deploy-prod      # Deploy to production
make backup           # Backup database
make health-check     # Check system health

# Monitoring
make logs             # View application logs
make metrics          # Open Prometheus
make dashboard        # Open Grafana

# Utilities
make clean            # Clean build artifacts
make help             # Show all commands
```

### Project Structure

```
UAIP/
├── crates/              # Rust crates
│   ├── uaip-core/
│   ├── uaip-auth/
│   ├── uaip-registry/
│   ├── uaip-router/
│   ├── uaip-security/
│   ├── uaip-orchestrator/
│   ├── uaip-adapters/
│   └── uaip-hub/
├── config/              # Configuration files
│   ├── default.toml
│   └── production.toml
├── migrations/          # Database migrations
│   ├── 001_initial_schema.sql
│   ├── 002_rbac_tables.sql
│   └── 003_performance_indexes.sql
├── docker/              # Docker configurations
│   ├── Dockerfile
│   ├── Dockerfile.dev
│   └── docker-compose.*.yml
├── scripts/             # Automation scripts
│   ├── backup.sh
│   ├── deploy.sh
│   ├── health-check.sh
│   └── load-test.sh
├── monitoring/          # Grafana dashboards
│   ├── dashboards/
│   └── prometheus.yml
├── .env.example         # Environment template
├── Makefile             # Task automation
└── README.md            # This file
```

---

## 🎯 Roadmap

### ✅ Completed (Production Ready)

- [x] **Phase 1**: Foundation & MVP
  - Core protocol implementation
  - Authentication system (JWT + X.509)
  - Device registry with caching
  - Message routing with QoS
  - REST & WebSocket APIs

- [x] **Phase 2**: Production Excellence
  - Structured logging & tracing
  - Rate limiting & DDoS protection
  - Health checks & graceful shutdown
  - Production Docker & Kubernetes

- [x] **Phase 3**: Developer Experience
  - Docker Compose environment
  - Makefile automation (40+ tasks)
  - Grafana dashboards (33 panels)
  - 5-minute onboarding

- [x] **Phase 4**: Advanced Features
  - Rule Engine (JSON-based automation)
  - Workflow Engine (state machines)
  - Scenario Engine (high-level patterns)
  - Protocol adapters (HTTP, WebSocket, MQTT)
  - AES-256-GCM encryption module

### 🚧 In Progress

- [ ] **Phase 5**: Enterprise Features
  - [ ] Multi-tenancy support
  - [ ] Advanced analytics engine
  - [ ] Webhook system
  - [ ] Firmware management (OTA)

- [ ] **Phase 6**: Scale & Performance
  - [ ] GraphQL API
  - [ ] WebSocket scaling (Redis pub/sub)
  - [ ] Horizontal pod autoscaling
  - [ ] Distributed tracing

- [ ] **Phase 7**: AI Integration
  - [ ] OpenAI function calling integration
  - [ ] Natural language device control
  - [ ] Predictive maintenance
  - [ ] Anomaly detection

---

## 🤝 Contributing

We welcome contributions from the community! Here's how you can help:

1. **Fork the repository**
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Commit your changes** (`git commit -m 'Add amazing feature'`)
4. **Push to the branch** (`git push origin feature/amazing-feature`)
5. **Open a Pull Request**

### Contribution Guidelines

- ✅ Follow Rust best practices and idioms
- ✅ Write tests for new features (maintain 100% coverage)
- ✅ Update documentation for API changes
- ✅ Use `cargo fmt` and `cargo clippy`
- ✅ Keep commits atomic and descriptive

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

---

## 📜 License

```
Copyright 2024-2025 Hakille and UAIP Contributors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

**UAIP (Universal AI Integration Protocol) is created and owned by Hakille.**

See [LICENSE](LICENSE) for the full license text.

---

## 👥 Authors & Credits

<table>
  <tr>
    <td align="center">
      <a href="https://github.com/Hakille">
        <img src="https://github.com/Hakille.png" width="100px;" alt="Hakille"/>
        <br />
        <sub><b>Hakille</b></sub>
      </a>
      <br />
      <sub>Creator & Lead Developer</sub>
    </td>
    <td align="center">
      <sub><b>Claude Sonnet 4.5</b></sub>
      <br />
      <sub>AI Development Assistant</sub>
    </td>
  </tr>
</table>

See [CONTRIBUTORS.md](CONTRIBUTORS.md) for all contributors.

---

## 📞 Support & Community

- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP/issues)
- 💡 **Feature Requests**: [GitHub Discussions](https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP/discussions)
- 📧 **Contact**: [@Hakille](https://github.com/Hakille)
- 📚 **Documentation**: [Wiki](https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP/wiki)

---

## 🌟 Sponsors

Support UAIP development by becoming a sponsor!

[Become a Sponsor](https://github.com/sponsors/Hakille)

---

<div align="center">

### ⭐ Star us on GitHub — it motivates us a lot!

**Made with ❤️ by [Hakille](https://github.com/Hakille)**

[![GitHub stars](https://img.shields.io/github/stars/UAIP-Universal-AI-Integration-Protocol/UAIP?style=social)](https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/UAIP-Universal-AI-Integration-Protocol/UAIP?style=social)](https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP/network/members)

</div>
