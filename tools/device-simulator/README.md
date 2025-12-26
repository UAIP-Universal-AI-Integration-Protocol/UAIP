# UAIP Device Simulator 🤖

A comprehensive mock IoT device simulator for testing and load testing the UAIP Hub.

## Features

- ✅ Realistic device simulation (smart bulbs, thermostats, cameras, sensors)
- ✅ Multiple operational modes (single, multi, load test, scenario)
- ✅ Configurable telemetry generation
- ✅ Load testing capabilities (1000+ concurrent devices)
- ✅ Pre-built realistic scenarios
- ✅ CLI interface with rich configuration

## Installation

```bash
cd tools/device-simulator
cargo build --release
```

The binary will be available at `../../target/release/device-simulator`.

## Quick Start

### Single Device

Simulate a single smart bulb:

```bash
device-simulator single \
  --device-type smart-bulb \
  --name living-room-light \
  --duration 60
```

### Multiple Devices

Simulate 10 mixed devices:

```bash
device-simulator multi \
  --types "smart-bulb,thermostat,camera" \
  --num-devices 10 \
  --duration 300
```

### Load Test

Stress test with 100 concurrent devices:

```bash
device-simulator load-test \
  --rps 500 \
  --concurrency 100 \
  --duration 600
```

### Scenario

Run a pre-built home automation scenario:

```bash
device-simulator scenario --name home
```

## Device Types

### Smart Bulb
- **Capabilities:** power, brightness, color
- **Telemetry:** on/off state, brightness level (0-100), RGB color
- **Use Case:** Smart home lighting

### Thermostat
- **Capabilities:** temperature, target temp, mode
- **Telemetry:** current temp, target temp, humidity, mode (heat/cool/auto)
- **Use Case:** Climate control

### Camera
- **Capabilities:** streaming, motion detection
- **Telemetry:** streaming state, motion events, FPS
- **Use Case:** Security and surveillance

### Generic Sensor
- **Capabilities:** read
- **Telemetry:** numeric value (0-100)
- **Use Case:** Custom sensors

## Scenarios

### Home Automation (6 devices)
- 2× Smart bulbs (living room, bedroom)
- 2× Thermostats (living room, bedroom)
- 2× Cameras (front door, backyard)

```bash
device-simulator scenario --name home
```

### Industrial IoT (15 devices)
- 10× Temperature sensors
- 5× Monitoring cameras

```bash
device-simulator scenario --name industrial
```

### Smart City (30 devices)
- 20× Street lights
- 10× Traffic cameras

```bash
device-simulator scenario --name smart-city
```

### Healthcare (30 devices)
- 15× Health monitors
- 15× Room climate controls

```bash
device-simulator scenario --name healthcare
```

## Configuration

### Environment Variables

```bash
# UAIP Hub URL (default: http://localhost:8443)
export UAIP_HUB_URL=https://uaip.example.com

# Run simulator
device-simulator single --device-type thermostat
```

### Command Line Options

```
OPTIONS:
  -h, --hub-url <URL>          UAIP Hub URL [env: UAIP_HUB_URL] [default: http://localhost:8443]
  -n, --num-devices <N>        Number of devices [default: 10]
  -d, --duration <SECONDS>     Simulation duration (0 = infinite) [default: 60]
```

## Commands

### `single`

Run a single device simulation.

```bash
device-simulator single [OPTIONS]

OPTIONS:
  -t, --device-type <TYPE>    Device type [default: smart-bulb]
  -n, --name <NAME>           Device name
```

**Example:**
```bash
device-simulator single -t thermostat -n bedroom-thermostat -d 120
```

### `multi`

Run multiple devices simulation.

```bash
device-simulator multi [OPTIONS]

OPTIONS:
  -t, --types <TYPES>         Device types (comma-separated) [default: smart-bulb,thermostat,camera]
```

**Example:**
```bash
device-simulator multi -t "smart-bulb,thermostat" -n 20 -d 300
```

### `load-test`

Run load testing scenario.

```bash
device-simulator load-test [OPTIONS]

OPTIONS:
  -r, --rps <RPS>             Requests per second [default: 100]
  -c, --concurrency <N>       Number of concurrent devices [default: 50]
```

**Example:**
```bash
device-simulator load-test -r 1000 -c 100 -d 600
```

### `scenario`

Run a pre-defined scenario.

```bash
device-simulator scenario [OPTIONS]

OPTIONS:
  -n, --name <NAME>           Scenario name [values: home, industrial, smart-city, healthcare]
```

**Example:**
```bash
device-simulator scenario -n smart-city -d 300
```

## Performance

### Benchmark Results

| Metric | Value |
|--------|-------|
| Max Concurrent Devices | 1000+ |
| Max Requests/Second | 10,000+ |
| Memory per Device | ~1MB |
| Telemetry Interval | 5 seconds (configurable) |

### Load Test Example

```bash
# Test with 500 devices, 5000 RPS for 10 minutes
device-simulator load-test \
  --concurrency 500 \
  --rps 5000 \
  --duration 600 \
  --hub-url http://uaip-hub:8443
```

**Expected Output:**
```
🚀 UAIP Device Simulator Starting
📡 Hub URL: http://uaip-hub:8443
⏱️  Duration: 600s
🔥 Load Test: 5000 RPS, 500 concurrent devices, 600s duration
✅ 500 devices registered
... (telemetry sending)
📊 Load test complete: 3,000,000 requests sent
```

## Development

### Adding Custom Device Types

Edit `src/device.rs` and add a new capability set:

```rust
fn default_capabilities_for_type(device_type: &str) -> Vec<Capability> {
    match device_type {
        "my-custom-device" => vec![
            Capability {
                name: "my_capability".to_string(),
                capability_type: "my_type".to_string(),
                actions: vec!["action1".to_string(), "action2".to_string()],
            },
        ],
        // ... existing types
    }
}
```

### Adding Custom Scenarios

Edit `src/scenarios.rs`:

```rust
pub fn create_my_scenario(hub_url: &str) -> Result<Scenario> {
    let mut scenario = Scenario::new("My Custom Scenario");

    // Add devices
    scenario.add_device(MockDevice::new(hub_url, "smart-bulb", "bulb-1")?);
    scenario.add_device(MockDevice::new(hub_url, "thermostat", "thermostat-1")?);

    Ok(scenario)
}
```

## Testing

### Unit Tests

```bash
cargo test
```

### Integration Test

```bash
# Start UAIP Hub
docker-compose -f docker-compose.dev.yml up -d

# Run simulator
cargo run -- multi -n 10 -d 60

# Verify in Grafana
open http://localhost:3000
```

## Troubleshooting

### Connection Refused

```
Error: Connection refused (os error 111)
```

**Solution:** Ensure UAIP Hub is running:
```bash
docker-compose -f docker-compose.dev.yml ps
```

### Too Many Devices

```
Error: Too many open files
```

**Solution:** Increase file descriptor limit:
```bash
ulimit -n 10000
```

### Out of Memory

```
Error: Cannot allocate memory
```

**Solution:** Reduce concurrency or use staggered starts:
```bash
device-simulator multi -n 100  # Instead of 1000
```

## Use Cases

### 1. Development Testing

Test your changes with realistic device traffic:

```bash
device-simulator multi -n 10 -d 300
```

### 2. Load Testing

Stress test before production:

```bash
device-simulator load-test -c 500 -r 5000 -d 600
```

### 3. Demo Preparation

Run a realistic scenario for demos:

```bash
device-simulator scenario -n home -d 0  # Runs indefinitely
```

### 4. Integration Testing

Automated testing in CI/CD:

```bash
#!/bin/bash
device-simulator multi -n 50 -d 120 &
SIMULATOR_PID=$!

# Run tests
pytest tests/

# Cleanup
kill $SIMULATOR_PID
```

### 5. Performance Benchmarking

Measure hub performance:

```bash
# Baseline
device-simulator load-test -c 100 -r 1000 -d 300

# After optimization
device-simulator load-test -c 100 -r 1000 -d 300

# Compare Grafana metrics
```

## License

Apache 2.0 - See [LICENSE](../../LICENSE)

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md)

## Support

- **Issues:** [GitHub Issues](https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP/issues)
- **Documentation:** [Main README](../../README.md)
- **Discussions:** [GitHub Discussions](https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP/discussions)
