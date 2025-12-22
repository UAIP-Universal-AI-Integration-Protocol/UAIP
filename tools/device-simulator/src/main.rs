use anyhow::{Context, Result};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DeviceType {
    TemperatureSensor,
    HumiditySensor,
    MotionSensor,
    DoorLock,
    Light,
    Thermostat,
    Camera,
    SmartPlug,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeviceConfig {
    id: String,
    device_type: DeviceType,
    name: String,
    location: String,
    update_interval_secs: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceMessage {
    device_id: String,
    timestamp: String,
    #[serde(flatten)]
    data: DeviceData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum DeviceData {
    Temperature {
        value: f64,
        unit: String,
    },
    Humidity {
        value: f64,
        unit: String,
    },
    Motion {
        detected: bool,
        confidence: f64,
    },
    DoorLock {
        locked: bool,
        battery: u8,
    },
    Light {
        on: bool,
        brightness: u8,
        color: String,
    },
    Thermostat {
        current_temp: f64,
        target_temp: f64,
        mode: String,
    },
    Camera {
        recording: bool,
        motion_detected: bool,
        resolution: String,
    },
    SmartPlug {
        on: bool,
        power_consumption: f64,
        voltage: f64,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct CommandMessage {
    command: String,
    device_id: String,
    #[serde(flatten)]
    params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct CommandResponse {
    device_id: String,
    command: String,
    success: bool,
    message: String,
}

#[derive(Parser, Debug)]
#[command(name = "UAIP Device Simulator")]
#[command(about = "Simulates IoT devices for UAIP Hub testing", long_about = None)]
struct Args {
    /// WebSocket URL of UAIP Hub
    #[arg(short, long, default_value = "ws://localhost:8443/ws/devices")]
    url: String,

    /// Number of devices to simulate
    #[arg(short, long, default_value = "5")]
    count: usize,

    /// Update interval in seconds
    #[arg(short, long, default_value = "10")]
    interval: u64,

    /// Device types (comma-separated: temp,humidity,motion,door,light,thermostat,camera,plug)
    #[arg(short = 't', long, default_value = "temp,humidity,motion")]
    device_types: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

struct DeviceSimulator {
    config: DeviceConfig,
    state: DeviceState,
}

#[derive(Debug, Clone)]
struct DeviceState {
    temperature: f64,
    humidity: f64,
    motion_detected: bool,
    door_locked: bool,
    light_on: bool,
    light_brightness: u8,
    thermostat_target: f64,
    camera_recording: bool,
    plug_on: bool,
}

impl Default for DeviceState {
    fn default() -> Self {
        Self {
            temperature: 20.0,
            humidity: 50.0,
            motion_detected: false,
            door_locked: true,
            light_on: false,
            light_brightness: 100,
            thermostat_target: 22.0,
            camera_recording: false,
            plug_on: true,
        }
    }
}

impl DeviceSimulator {
    fn new(config: DeviceConfig) -> Self {
        Self {
            config,
            state: DeviceState::default(),
        }
    }

    fn generate_data(&mut self) -> DeviceData {
        let mut rng = rand::thread_rng();

        match self.config.device_type {
            DeviceType::TemperatureSensor => {
                // Simulate temperature drift
                self.state.temperature += rng.gen_range(-0.5..0.5);
                self.state.temperature = self.state.temperature.clamp(15.0, 30.0);
                DeviceData::Temperature {
                    value: (self.state.temperature * 10.0).round() / 10.0,
                    unit: "celsius".to_string(),
                }
            }
            DeviceType::HumiditySensor => {
                // Simulate humidity changes
                self.state.humidity += rng.gen_range(-2.0..2.0);
                self.state.humidity = self.state.humidity.clamp(30.0, 70.0);
                DeviceData::Humidity {
                    value: self.state.humidity.round(),
                    unit: "percent".to_string(),
                }
            }
            DeviceType::MotionSensor => {
                // Random motion events
                self.state.motion_detected = rng.gen_bool(0.1); // 10% chance
                DeviceData::Motion {
                    detected: self.state.motion_detected,
                    confidence: if self.state.motion_detected {
                        rng.gen_range(0.8..1.0)
                    } else {
                        0.0
                    },
                }
            }
            DeviceType::DoorLock => DeviceData::DoorLock {
                locked: self.state.door_locked,
                battery: rng.gen_range(70..100),
            },
            DeviceType::Light => DeviceData::Light {
                on: self.state.light_on,
                brightness: self.state.light_brightness,
                color: "#FFFFFF".to_string(),
            },
            DeviceType::Thermostat => {
                // Simulate temperature approaching target
                let diff = self.state.thermostat_target - self.state.temperature;
                self.state.temperature += diff * 0.1;
                DeviceData::Thermostat {
                    current_temp: (self.state.temperature * 10.0).round() / 10.0,
                    target_temp: self.state.thermostat_target,
                    mode: "auto".to_string(),
                }
            }
            DeviceType::Camera => DeviceData::Camera {
                recording: self.state.camera_recording,
                motion_detected: self.state.motion_detected,
                resolution: "1920x1080".to_string(),
            },
            DeviceType::SmartPlug => {
                let power = if self.state.plug_on {
                    rng.gen_range(50.0..150.0)
                } else {
                    0.0
                };
                DeviceData::SmartPlug {
                    on: self.state.plug_on,
                    power_consumption: (power * 10.0).round() / 10.0,
                    voltage: 230.0,
                }
            }
        }
    }

    fn handle_command(&mut self, command: &str, params: &serde_json::Value) -> Result<String> {
        match (command, &self.config.device_type) {
            ("lock", DeviceType::DoorLock) => {
                self.state.door_locked = true;
                Ok("Door locked successfully".to_string())
            }
            ("unlock", DeviceType::DoorLock) => {
                self.state.door_locked = false;
                Ok("Door unlocked successfully".to_string())
            }
            ("turn_on", DeviceType::Light) => {
                self.state.light_on = true;
                Ok("Light turned on".to_string())
            }
            ("turn_off", DeviceType::Light) => {
                self.state.light_on = false;
                Ok("Light turned off".to_string())
            }
            ("set_brightness", DeviceType::Light) => {
                if let Some(brightness) = params.get("brightness").and_then(|v| v.as_u64()) {
                    self.state.light_brightness = brightness.min(100) as u8;
                    Ok(format!("Brightness set to {}", self.state.light_brightness))
                } else {
                    anyhow::bail!("Invalid brightness value")
                }
            }
            ("set_temperature", DeviceType::Thermostat) => {
                if let Some(temp) = params.get("temperature").and_then(|v| v.as_f64()) {
                    self.state.thermostat_target = temp;
                    Ok(format!("Target temperature set to {}°C", temp))
                } else {
                    anyhow::bail!("Invalid temperature value")
                }
            }
            ("start_recording", DeviceType::Camera) => {
                self.state.camera_recording = true;
                Ok("Recording started".to_string())
            }
            ("stop_recording", DeviceType::Camera) => {
                self.state.camera_recording = false;
                Ok("Recording stopped".to_string())
            }
            ("turn_on", DeviceType::SmartPlug) => {
                self.state.plug_on = true;
                Ok("Smart plug turned on".to_string())
            }
            ("turn_off", DeviceType::SmartPlug) => {
                self.state.plug_on = false;
                Ok("Smart plug turned off".to_string())
            }
            _ => anyhow::bail!("Command '{}' not supported for this device type", command),
        }
    }

    async fn run(&mut self, url: String) -> Result<()> {
        info!(
            "Connecting device {} ({:?}) to {}",
            self.config.name, self.config.device_type, url
        );

        let (ws_stream, _) = connect_async(&url)
            .await
            .context("Failed to connect to WebSocket")?;

        let (mut write, mut read) = ws_stream.split();

        // Send initial registration message
        let registration = serde_json::json!({
            "type": "register",
            "device_id": self.config.id,
            "device_type": format!("{:?}", self.config.device_type),
            "name": self.config.name,
            "location": self.config.location,
        });

        write
            .send(Message::Text(registration.to_string()))
            .await
            .context("Failed to send registration")?;

        info!("Device {} registered successfully", self.config.name);

        let mut interval = time::interval(Duration::from_secs(self.config.update_interval_secs));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // Send periodic data
                    let data = self.generate_data();
                    let message = DeviceMessage {
                        device_id: self.config.id.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        data,
                    };

                    let json = serde_json::to_string(&message)?;
                    if let Err(e) = write.send(Message::Text(json)).await {
                        error!("Failed to send data: {}", e);
                        break;
                    }
                }
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            // Handle incoming commands
                            if let Ok(cmd) = serde_json::from_str::<CommandMessage>(&text) {
                                if cmd.device_id == self.config.id {
                                    info!("Received command '{}' for device {}", cmd.command, self.config.name);

                                    let (success, message) = match self.handle_command(&cmd.command, &cmd.params) {
                                        Ok(msg) => (true, msg),
                                        Err(e) => (false, e.to_string()),
                                    };

                                    let response = CommandResponse {
                                        device_id: self.config.id.clone(),
                                        command: cmd.command,
                                        success,
                                        message,
                                    };

                                    let json = serde_json::to_string(&response)?;
                                    if let Err(e) = write.send(Message::Text(json)).await {
                                        error!("Failed to send response: {}", e);
                                    }
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("Server closed connection for device {}", self.config.name);
                            break;
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            warn!("WebSocket stream ended for device {}", self.config.name);
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        info!("Device {} disconnected", self.config.name);
        Ok(())
    }
}

fn parse_device_types(types_str: &str) -> Vec<DeviceType> {
    types_str
        .split(',')
        .filter_map(|s| match s.trim().to_lowercase().as_str() {
            "temp" | "temperature" => Some(DeviceType::TemperatureSensor),
            "humidity" => Some(DeviceType::HumiditySensor),
            "motion" => Some(DeviceType::MotionSensor),
            "door" | "lock" => Some(DeviceType::DoorLock),
            "light" => Some(DeviceType::Light),
            "thermostat" => Some(DeviceType::Thermostat),
            "camera" => Some(DeviceType::Camera),
            "plug" | "smartplug" => Some(DeviceType::SmartPlug),
            _ => None,
        })
        .collect()
}

fn generate_device_configs(count: usize, device_types: Vec<DeviceType>, interval: u64) -> Vec<DeviceConfig> {
    let locations = vec!["Living Room", "Bedroom", "Kitchen", "Bathroom", "Garage", "Garden"];
    let mut configs = Vec::new();

    for i in 0..count {
        let device_type = device_types[i % device_types.len()].clone();
        let location = locations[i % locations.len()];
        let name = format!("{:?} #{}", device_type, i + 1);

        configs.push(DeviceConfig {
            id: Uuid::new_v4().to_string(),
            device_type,
            name,
            location: location.to_string(),
            update_interval_secs: interval,
        });
    }

    configs
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let filter = if args.verbose {
        "device_simulator=debug,info"
    } else {
        "device_simulator=info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    info!("🚀 UAIP Device Simulator starting...");
    info!("Hub URL: {}", args.url);
    info!("Device count: {}", args.count);
    info!("Update interval: {}s", args.interval);

    let device_types = parse_device_types(&args.device_types);
    if device_types.is_empty() {
        error!("No valid device types specified");
        return Ok(());
    }

    info!("Device types: {:?}", device_types);

    let configs = generate_device_configs(args.count, device_types, args.interval);

    // Spawn tasks for each device
    let mut handles = Vec::new();

    for config in configs {
        let url = args.url.clone();
        let handle = tokio::spawn(async move {
            let mut simulator = DeviceSimulator::new(config);
            if let Err(e) = simulator.run(url).await {
                error!("Device simulator error: {}", e);
            }
        });
        handles.push(handle);

        // Stagger device connections
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    info!("✅ All {} devices connected", args.count);
    info!("Press Ctrl+C to stop");

    // Wait for all devices
    for handle in handles {
        let _ = handle.await;
    }

    info!("Device simulator stopped");
    Ok(())
}
