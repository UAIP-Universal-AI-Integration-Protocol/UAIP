# UAIP Hub Integration Tests

Comprehensive integration test suite for the UAIP Hub, providing end-to-end testing of all major components.

## 📋 Overview

The integration test harness provides:
- **Test Helpers**: Common utilities and fixtures
- **API Client**: Type-safe HTTP client wrapper
- **Database Helpers**: Easy database setup and cleanup
- **Custom Assertions**: Expressive test assertions
- **Fixtures**: Pre-built test data generators

## 🚀 Quick Start

### Run All Integration Tests

```bash
# From project root
cargo test --test '*' --workspace

# Run specific test suite
cargo test --test api_integration
cargo test --test device_integration
cargo test --test auth_integration
cargo test --test messaging_integration
```

### Run with Docker Compose

```bash
# Start test environment
docker-compose -f docker-compose.dev.yml up -d

# Run tests
cargo test --test '*' --workspace

# Cleanup
docker-compose -f docker-compose.dev.yml down
```

## 📁 Structure

```
tests/integration/
├── Cargo.toml                    # Test dependencies
├── README.md                     # This file
├── helpers/
│   ├── mod.rs                    # Main helper module
│   ├── client.rs                 # API client wrapper
│   ├── fixtures.rs               # Test data generators
│   ├── assertions.rs             # Custom assertions
│   └── database.rs               # Database helpers
├── api_integration.rs            # API endpoint tests
├── device_integration.rs         # Device management tests
├── auth_integration.rs           # Authentication tests
└── messaging_integration.rs      # Message routing tests
```

## 🧰 Test Helpers

### API Client

Type-safe HTTP client with authentication support:

```rust
use integration_tests::*;

#[tokio::test]
async fn test_example() {
    let ctx = TestContext::new().await.unwrap();
    let mut client = ctx.client();

    // Login
    client.login("admin", "password").await.unwrap();

    // Make authenticated requests
    let devices: DeviceListResponse = client
        .get_json("/api/v1/devices")
        .await
        .unwrap();

    assert!(devices.total > 0);
}
```

### Fixtures

Pre-built test data generators:

```rust
use integration_tests::*;

#[tokio::test]
async fn test_with_fixtures() {
    let ctx = TestContext::new().await.unwrap();
    let mut client = ctx.client();
    client.login("admin", "password").await.unwrap();

    // Create device fixtures
    let temp_sensor = temperature_sensor_fixture("Sensor 1");
    let smart_light = smart_light_fixture("Light 1");
    let thermostat = thermostat_fixture("Thermostat 1");

    // Register devices
    let response = client.post("/api/v1/devices", &temp_sensor).await.unwrap();
    assert_status(response, StatusCode::CREATED).await;
}
```

### Custom Assertions

Expressive assertions for common patterns:

```rust
use integration_tests::*;

#[tokio::test]
async fn test_with_assertions() {
    let ctx = TestContext::new().await.unwrap();
    let client = ctx.client();

    // Assert status code
    let response = client.get("/api/v1/system/health").await.unwrap();
    let response = assert_status(response, StatusCode::OK).await;

    // Assert JSON deserialization
    let health: HealthResponse = assert_json(response).await;

    // Assert approximate equality (floats)
    assert_approx_eq(health.uptime_seconds as f64, 100.0, 50.0);

    // Assert vector contents
    let devices = vec!["device1", "device2", "device3"];
    assert_length(&devices, 3);
    assert_contains_element(&devices, &"device1");
}
```

### Database Helpers

Easy database manipulation:

```rust
use integration_tests::*;

#[tokio::test]
async fn test_with_database() {
    let ctx = TestContext::new().await.unwrap();
    let db = DatabaseHelper::new(&ctx.db_pool);

    // Insert test data
    let user_id = db.insert_user("testuser", "test@example.com", "hash").await.unwrap();
    let device_id = db.insert_device("Sensor 1", "temperature", "online").await.unwrap();

    // Query data
    let count = db.count_devices().await.unwrap();
    assert_eq!(count, 1);

    // Cleanup
    db.cleanup().await.unwrap();
}
```

## 📝 Test Suites

### 1. API Integration Tests (`api_integration.rs`)

Tests core API functionality:
- Health checks (health, liveness, readiness)
- Metrics endpoint
- CORS headers
- Request ID tracking
- Error handling (404, 405)

**Example:**
```rust
#[tokio::test]
async fn test_health_check() {
    let ctx = TestContext::new().await.unwrap();
    let client = ctx.client();

    let response = client.get("/api/v1/system/health").await.unwrap();
    let response = assert_status(response, StatusCode::OK).await;
    let health: HealthResponse = assert_json(response).await;

    assert_eq!(health.status, "healthy");
}
```

### 2. Device Integration Tests (`device_integration.rs`)

Tests device management:
- Device registration
- Device listing and filtering
- Device updates
- Device deletion
- Device commands

**Example:**
```rust
#[tokio::test]
async fn test_device_registration() {
    let ctx = TestContext::new().await.unwrap();
    let mut client = ctx.client();
    client.login("admin", "password").await.unwrap();

    let device = temperature_sensor_fixture("Test Sensor");
    let response = client.post("/api/v1/devices", &device).await.unwrap();

    let response = assert_status(response, StatusCode::CREATED).await;
    let registered: Device = assert_json(response).await;

    assert_eq!(registered.name, "Test Sensor");
    assert_eq!(registered.device_type, "temperature_sensor");
}
```

### 3. Authentication Tests (`auth_integration.rs`)

Tests authentication and authorization:
- User login
- Token validation
- Token refresh
- Permission checks
- Unauthorized access

**Example:**
```rust
#[tokio::test]
async fn test_login() {
    let ctx = TestContext::new().await.unwrap();
    let mut client = ctx.client();

    let response = client.login("admin", "password").await;
    assert!(response.is_ok());

    let login_resp = response.unwrap();
    assert!(!login_resp.token.is_empty());
    assert!(login_resp.expires_in > 0);
}
```

### 4. Messaging Tests (`messaging_integration.rs`)

Tests message routing:
- Message sending
- Message delivery
- QoS levels (0, 1, 2)
- Priority handling
- Message status tracking

**Example:**
```rust
#[tokio::test]
async fn test_send_message() {
    let ctx = TestContext::new().await.unwrap();
    let mut client = ctx.client();
    client.login("admin", "password").await.unwrap();

    let message = message_fixture("device1", "device2");
    let response = client.post("/api/v1/messages", &message").await.unwrap();

    let response = assert_status(response, StatusCode::ACCEPTED).await;
    let msg_resp: MessageResponse = assert_json(response).await;

    assert_eq!(msg_resp.status, "queued");
}
```

## 🔧 Configuration

### Environment Variables

```bash
# Test environment configuration
export TEST_BASE_URL="http://localhost:8443"
export DATABASE_URL="postgresql://uaip:uaip_test_password@localhost:5432/uaip_test"
export REDIS_URL="redis://localhost:6379"
export NATS_URL="nats://localhost:4222"
```

### Test Database Setup

```bash
# Create test database
createdb uaip_test

# Run migrations
psql -U uaip -d uaip_test -f migrations/001_initial_schema.sql
psql -U uaip -d uaip_test -f migrations/002_rbac_tables.sql
```

## 🎯 Best Practices

### 1. Use Test Context

Always create a test context for consistent setup:

```rust
#[tokio::test]
async fn my_test() {
    let ctx = TestContext::new().await.unwrap();
    // Use ctx.client(), ctx.db_pool, etc.
}
```

### 2. Clean Up After Tests

Use cleanup helpers to avoid test pollution:

```rust
#[tokio::test]
async fn my_test() {
    let ctx = TestContext::new().await.unwrap();

    // Test code...

    // Cleanup
    ctx.cleanup().await.unwrap();
}
```

### 3. Use Fixtures

Use fixtures instead of hardcoded test data:

```rust
// ❌ Bad
let device = DeviceRegistration {
    name: "Test Device".to_string(),
    device_type: "sensor".to_string(),
    // ...
};

// ✅ Good
let device = temperature_sensor_fixture("Test Device");
```

### 4. Use Custom Assertions

Use custom assertions for clearer test failures:

```rust
// ❌ Bad
assert_eq!(response.status(), StatusCode::OK);

// ✅ Good
assert_status(response, StatusCode::OK).await;
```

### 5. Test Error Cases

Don't just test happy paths:

```rust
#[tokio::test]
async fn test_invalid_device_registration() {
    let ctx = TestContext::new().await.unwrap();
    let mut client = ctx.client();
    client.login("admin", "password").await.unwrap();

    // Missing required fields
    let invalid_device = serde_json::json!({
        "name": "Test"
        // Missing device_type
    });

    let response = client.post("/api/v1/devices", &invalid_device).await.unwrap();
    assert_client_error(response).await;
}
```

## 🔍 Debugging Tests

### Enable Logging

```bash
# Run with debug logging
RUST_LOG=debug cargo test --test api_integration

# Run with trace logging
RUST_LOG=trace cargo test --test device_integration
```

### Run Single Test

```bash
# Run specific test
cargo test test_health_check

# Run with output
cargo test test_health_check -- --nocapture
```

### Inspect Test Database

```bash
# Connect to test database
psql -U uaip -d uaip_test

# View test data
SELECT * FROM devices;
SELECT * FROM messages;
```

## 📊 Coverage

Generate test coverage report:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --test '*' --out Html

# Open report
open tarpaulin-report.html
```

## 🚨 Troubleshooting

### Tests Fail to Connect

**Problem:** Tests can't connect to services

**Solutions:**
1. Ensure services are running: `docker-compose ps`
2. Check environment variables: `echo $DATABASE_URL`
3. Verify ports are not in use: `lsof -i :8443`

### Database Errors

**Problem:** Database errors during tests

**Solutions:**
1. Run migrations: `psql -U uaip -d uaip_test -f migrations/*.sql`
2. Clean test database: `dropdb uaip_test && createdb uaip_test`
3. Check permissions: `psql -U uaip -d uaip_test -c '\du'`

### Flaky Tests

**Problem:** Tests pass/fail intermittently

**Solutions:**
1. Add proper cleanup between tests
2. Increase timeouts for slow operations
3. Use test isolation (separate databases)
4. Add retry logic for network operations

## 📚 Additional Resources

- [Main README](../../README.md)
- [API Documentation](../../docs/api/README.md)
- [Database Optimization](../../docs/DATABASE_OPTIMIZATION.md)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)

## 🤝 Contributing

When adding new tests:

1. Follow existing patterns and naming conventions
2. Add fixtures for common test data
3. Include both success and error cases
4. Add documentation for complex test scenarios
5. Ensure tests are isolated and can run in parallel

## 📞 Support

- GitHub Issues: https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP/issues
- Documentation: Main project README

---

**Write tests, not bugs!** 🧪
