# UAIP Device Simulator

A comprehensive IoT device simulator for testing and developing with UAIP Hub. Simulates realistic device behavior including sensor readings, actuator control, and command responses.

## Features

### Supported Device Types

1. **Temperature Sensor** - Simulates temperature readings with realistic drift
2. **Humidity Sensor** - Simulates humidity percentage changes
3. **Motion Sensor** - Random motion detection events
4. **Door Lock** - Smart lock with lock/unlock commands
5. **Light** - Smart light with on/off and brightness control
6. **Thermostat** - Smart thermostat with temperature control
7. **Camera** - Security camera with recording control
8. **Smart Plug** - Smart power outlet with power consumption tracking

### Key Features

- ✅ **Realistic Data Generation** - Simulates actual sensor behavior with drift and noise
- ✅ **Command Support** - Responds to commands (lock/unlock, on/off, set values)
- ✅ **WebSocket Communication** - Connects to UAIP Hub via WebSocket
- ✅ **Multiple Devices** - Simulate hundreds of devices simultaneously
- ✅ **Configurable** - Adjust update intervals, device types, and counts
- ✅ **Production-Ready** - Built with Tokio for high performance
- ✅ **Easy CLI** - Simple command-line interface

## Installation

### Prerequisites

- Rust 1.70+ (install from https://rustup.rs)
- UAIP Hub running (see main README)

### Build

```bash
cd tools/device-simulator
cargo build --release
```

The binary will be at `target/release/device-simulator`

## Quick Start

### Basic Usage

```bash
# Simulate 5 devices (default mix of sensors)
cargo run

# Or use the binary directly
./target/release/device-simulator
```

### Custom Configuration

```bash
# Simulate 10 temperature sensors
cargo run -- -c 10 -t temp

# Multiple device types
cargo run -- -c 20 -t "temp,humidity,motion,door,light"

# Connect to remote hub
cargo run -- -u ws://uaip-hub.example.com:8443/ws/devices -c 10

# Fast updates (every 2 seconds)
cargo run -- -c 5 -i 2

# Verbose logging
cargo run -- -v -c 5
```

## Command-Line Options

```
Usage: device-simulator [OPTIONS]

Options:
  -u, --url <URL>
          WebSocket URL of UAIP Hub
          [default: ws://localhost:8443/ws/devices]

  -c, --count <COUNT>
          Number of devices to simulate
          [default: 5]

  -i, --interval <INTERVAL>
          Update interval in seconds
          [default: 10]

  -t, --device-types <DEVICE_TYPES>
          Device types (comma-separated)
          Available types: temp, humidity, motion, door, light, thermostat, camera, plug
          [default: temp,humidity,motion]

  -v, --verbose
          Enable verbose logging

  -h, --help
          Print help

  -V, --version
          Print version
```

## Device Types Reference

### Temperature Sensor (`temp`, `temperature`)

**Data Format:**
```json
{
  "device_id": "uuid",
  "timestamp": "2025-01-22T14:30:00Z",
  "type": "temperature",
  "value": 22.5,
  "unit": "celsius"
}
```

**Behavior:**
- Temperature drifts slowly (-0.5°C to +0.5°C per update)
- Range: 15°C to 30°C
- Simulates realistic room temperature changes

### Humidity Sensor (`humidity`)

**Data Format:**
```json
{
  "device_id": "uuid",
  "timestamp": "2025-01-22T14:30:00Z",
  "type": "humidity",
  "value": 55.0,
  "unit": "percent"
}
```

**Behavior:**
- Humidity changes gradually (-2% to +2% per update)
- Range: 30% to 70%
- Realistic indoor humidity simulation

### Motion Sensor (`motion`)

**Data Format:**
```json
{
  "device_id": "uuid",
  "timestamp": "2025-01-22T14:30:00Z",
  "type": "motion",
  "detected": true,
  "confidence": 0.95
}
```

**Behavior:**
- Random motion events (10% probability per update)
- Confidence: 0.8-1.0 when motion detected
- Useful for security system testing

### Door Lock (`door`, `lock`)

**Data Format:**
```json
{
  "device_id": "uuid",
  "timestamp": "2025-01-22T14:30:00Z",
  "type": "door_lock",
  "locked": true,
  "battery": 85
}
```

**Supported Commands:**
- `lock` - Lock the door
- `unlock` - Unlock the door

**Example Command:**
```json
{
  "command": "lock",
  "device_id": "uuid"
}
```

### Light (`light`)

**Data Format:**
```json
{
  "device_id": "uuid",
  "timestamp": "2025-01-22T14:30:00Z",
  "type": "light",
  "on": true,
  "brightness": 75,
  "color": "#FFFFFF"
}
```

**Supported Commands:**
- `turn_on` - Turn light on
- `turn_off` - Turn light off
- `set_brightness` - Set brightness (0-100)

**Example Command:**
```json
{
  "command": "set_brightness",
  "device_id": "uuid",
  "brightness": 50
}
```

### Thermostat (`thermostat`)

**Data Format:**
```json
{
  "device_id": "uuid",
  "timestamp": "2025-01-22T14:30:00Z",
  "type": "thermostat",
  "current_temp": 21.5,
  "target_temp": 22.0,
  "mode": "auto"
}
```

**Supported Commands:**
- `set_temperature` - Set target temperature

**Example Command:**
```json
{
  "command": "set_temperature",
  "device_id": "uuid",
  "temperature": 23.5
}
```

**Behavior:**
- Current temperature gradually approaches target
- Simulates realistic HVAC behavior

### Camera (`camera`)

**Data Format:**
```json
{
  "device_id": "uuid",
  "timestamp": "2025-01-22T14:30:00Z",
  "type": "camera",
  "recording": false,
  "motion_detected": false,
  "resolution": "1920x1080"
}
```

**Supported Commands:**
- `start_recording` - Start recording
- `stop_recording` - Stop recording

### Smart Plug (`plug`, `smartplug`)

**Data Format:**
```json
{
  "device_id": "uuid",
  "timestamp": "2025-01-22T14:30:00Z",
  "type": "smart_plug",
  "on": true,
  "power_consumption": 125.5,
  "voltage": 230.0
}
```

**Supported Commands:**
- `turn_on` - Turn plug on
- `turn_off` - Turn plug off

**Behavior:**
- Power consumption: 50-150W when on, 0W when off
- Simulates realistic power usage

## Usage Examples

### Development Testing

```bash
# Start hub in development mode
cd ../../
make dev

# In another terminal, start simulator
cd tools/device-simulator
cargo run -- -c 10 -i 5
```

### Load Testing

```bash
# Simulate 100 devices with 2-second updates
cargo run --release -- -c 100 -i 2 -t "temp,humidity,motion,door,light"

# Simulate 1000 devices (stress test)
cargo run --release -- -c 1000 -i 10
```

### Specific Scenarios

#### Smart Home Scenario
```bash
# Simulate a smart home with various devices
cargo run -- -c 20 -t "temp,humidity,motion,door,light,thermostat,camera,plug" -i 10
```

#### Security System Scenario
```bash
# Focus on security devices
cargo run -- -c 15 -t "motion,door,camera" -i 5
```

#### Energy Monitoring Scenario
```bash
# Multiple smart plugs for energy tracking
cargo run -- -c 10 -t "plug" -i 1
```

## Integration with UAIP Hub

### Message Flow

```
Device Simulator → WebSocket → UAIP Hub
                                    ↓
                            Message Router
                                    ↓
                        Device Registry (Redis)
                                    ↓
                            Database (PostgreSQL)
```

### Registration Process

1. Device connects to WebSocket endpoint
2. Sends registration message with device info
3. Hub validates and registers device
4. Device starts sending periodic updates
5. Device listens for incoming commands

### Command Handling

1. Hub sends command to device via WebSocket
2. Device executes command (updates internal state)
3. Device sends command response
4. Next data update reflects new state

## Monitoring Simulated Devices

### View in Grafana

1. Start simulator: `cargo run -- -c 10`
2. Open Grafana: http://localhost:3000
3. Navigate to "UAIP Hub Overview" dashboard
4. See device metrics in real-time

### Query Device Data

```bash
# View metrics
curl http://localhost:8443/metrics | grep uaip_devices

# Health check
curl http://localhost:8443/api/v1/system/health

# Device list (requires authentication)
curl -H "Authorization: Bearer <token>" \
     http://localhost:8443/api/v1/devices
```

## Advanced Usage

### Environment Variables

```bash
# Set via environment
export DEVICE_SIMULATOR_URL="ws://hub.example.com:8443/ws/devices"
export DEVICE_SIMULATOR_COUNT=50
export DEVICE_SIMULATOR_INTERVAL=5

cargo run
```

### Programmatic Usage

You can also use the simulator as a library:

```rust
use uaip_device_simulator::{DeviceSimulator, DeviceConfig, DeviceType};

#[tokio::main]
async fn main() {
    let config = DeviceConfig {
        id: "my-device-1".to_string(),
        device_type: DeviceType::TemperatureSensor,
        name: "Living Room Temp".to_string(),
        location: "Living Room".to_string(),
        update_interval_secs: 10,
    };

    let mut simulator = DeviceSimulator::new(config);
    simulator.run("ws://localhost:8443/ws/devices".to_string()).await.unwrap();
}
```

## Troubleshooting

### Connection Failed

**Problem:** `Failed to connect to WebSocket`

**Solutions:**
1. Ensure UAIP Hub is running: `make health`
2. Check WebSocket endpoint: `curl http://localhost:8443/health`
3. Verify URL is correct (default: `ws://localhost:8443/ws/devices`)

### High CPU Usage

**Problem:** Simulator using too much CPU

**Solutions:**
1. Reduce device count: `-c 10` instead of `-c 1000`
2. Increase update interval: `-i 30` instead of `-i 1`
3. Use release build: `cargo build --release`

### Devices Not Appearing

**Problem:** Devices connect but don't appear in hub

**Solutions:**
1. Check hub logs: `make logs-hub`
2. Verify authentication if required
3. Check device registration format matches hub expectations

## Performance Benchmarks

### Single Device
- Memory: ~100KB per device
- CPU: <0.1% per device (10s interval)
- Network: ~100 bytes/update

### 100 Devices
- Memory: ~10MB total
- CPU: ~5% (10s interval)
- Network: ~10KB/s

### 1000 Devices (Stress Test)
- Memory: ~100MB total
- CPU: ~30% (10s interval)
- Network: ~100KB/s

*Benchmarks on: Intel i7, 16GB RAM, Ubuntu 22.04*

## Development

### Run Tests

```bash
cargo test
```

### Run with Logging

```bash
RUST_LOG=debug cargo run -- -c 5
```

### Format Code

```bash
cargo fmt
```

### Lint Code

```bash
cargo clippy
```

## CI/CD Integration

```yaml
# Example GitHub Actions
- name: Test Device Simulator
  run: |
    cd tools/device-simulator
    cargo test
    cargo build --release
```

## Contributing

When adding new device types:

1. Add enum variant to `DeviceType`
2. Add data structure to `DeviceData`
3. Implement data generation in `generate_data()`
4. Implement command handling in `handle_command()`
5. Add parser case in `parse_device_types()`
6. Update this README with examples

## License

Apache 2.0 - See main project LICENSE

## Support

- Issues: https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP/issues
- Documentation: Main UAIP README
- Examples: See `examples/` directory (coming soon)

---

**Built with ❤️ for UAIP Hub testing and development**
