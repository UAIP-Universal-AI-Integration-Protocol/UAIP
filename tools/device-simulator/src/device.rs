// ! Mock Device Implementation

use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

#[derive(Clone)]
pub struct MockDevice {
    id: String,
    name: String,
    device_type: String,
    hub_url: String,
    client: reqwest::Client,
    capabilities: Vec<Capability>,
    state: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub capability_type: String,
    pub actions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub device_type: String,
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TelemetryData {
    pub device_id: String,
    pub timestamp: String,
    pub data: HashMap<String, serde_json::Value>,
}

impl MockDevice {
    pub fn new(hub_url: &str, device_type: &str, name: &str) -> Result<Self> {
        let id = Uuid::new_v4().to_string();
        let capabilities = Self::default_capabilities_for_type(device_type);

        Ok(Self {
            id: id.clone(),
            name: name.to_string(),
            device_type: device_type.to_string(),
            hub_url: hub_url.to_string(),
            client: reqwest::Client::new(),
            capabilities,
            state: HashMap::new(),
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    fn default_capabilities_for_type(device_type: &str) -> Vec<Capability> {
        match device_type {
            "smart-bulb" => vec![
                Capability {
                    name: "power".to_string(),
                    capability_type: "switch".to_string(),
                    actions: vec!["on".to_string(), "off".to_string()],
                },
                Capability {
                    name: "brightness".to_string(),
                    capability_type: "level".to_string(),
                    actions: vec!["set".to_string()],
                },
                Capability {
                    name: "color".to_string(),
                    capability_type: "color".to_string(),
                    actions: vec!["set".to_string()],
                },
            ],
            "thermostat" => vec![
                Capability {
                    name: "temperature".to_string(),
                    capability_type: "sensor".to_string(),
                    actions: vec!["read".to_string()],
                },
                Capability {
                    name: "target_temp".to_string(),
                    capability_type: "setpoint".to_string(),
                    actions: vec!["set".to_string()],
                },
                Capability {
                    name: "mode".to_string(),
                    capability_type: "mode".to_string(),
                    actions: vec!["heat".to_string(), "cool".to_string(), "auto".to_string()],
                },
            ],
            "camera" => vec![
                Capability {
                    name: "stream".to_string(),
                    capability_type: "video".to_string(),
                    actions: vec!["start".to_string(), "stop".to_string()],
                },
                Capability {
                    name: "motion".to_string(),
                    capability_type: "sensor".to_string(),
                    actions: vec!["detect".to_string()],
                },
            ],
            _ => vec![Capability {
                name: "generic".to_string(),
                capability_type: "sensor".to_string(),
                actions: vec!["read".to_string()],
            }],
        }
    }

    /// Register device with UAIP Hub
    pub async fn register(&mut self) -> Result<()> {
        let url = format!("{}/api/v1/devices/register", self.hub_url);
        let request = RegisterRequest {
            name: self.name.clone(),
            device_type: self.device_type.clone(),
            capabilities: self.capabilities.clone(),
        };

        debug!("Registering device: {}", self.name);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        info!("Device registered: {}", self.id);
        Ok(())
    }

    /// Unregister device from UAIP Hub
    pub async fn unregister(&self) -> Result<()> {
        let url = format!("{}/api/v1/devices/{}", self.hub_url, self.id);

        debug!("Unregistering device: {}", self.name);

        self.client
            .delete(&url)
            .send()
            .await?
            .error_for_status()?;

        info!("Device unregistered: {}", self.id);
        Ok(())
    }

    /// Send telemetry data
    pub async fn send_telemetry(&mut self) -> Result<()> {
        let data = self.generate_telemetry_data();
        let url = format!("{}/api/v1/devices/{}/telemetry", self.hub_url, self.id);

        let telemetry = TelemetryData {
            device_id: self.id.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data,
        };

        debug!("Sending telemetry for device: {}", self.name);

        self.client
            .post(&url)
            .json(&telemetry)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Generate random telemetry data based on device type
    fn generate_telemetry_data(&self) -> HashMap<String, serde_json::Value> {
        let mut rng = rand::thread_rng();
        let mut data = HashMap::new();

        match self.device_type.as_str() {
            "smart-bulb" => {
                data.insert(
                    "power".to_string(),
                    serde_json::json!(rng.gen_bool(0.7)),
                );
                data.insert(
                    "brightness".to_string(),
                    serde_json::json!(rng.gen_range(0..100)),
                );
                data.insert(
                    "color".to_string(),
                    serde_json::json!(format!("#{:06x}", rng.gen_range(0..0xFFFFFF))),
                );
            }
            "thermostat" => {
                data.insert(
                    "temperature".to_string(),
                    serde_json::json!(rng.gen_range(18.0..28.0)),
                );
                data.insert(
                    "target_temp".to_string(),
                    serde_json::json!(rng.gen_range(20.0..25.0)),
                );
                data.insert(
                    "humidity".to_string(),
                    serde_json::json!(rng.gen_range(30..70)),
                );
            }
            "camera" => {
                data.insert(
                    "streaming".to_string(),
                    serde_json::json!(rng.gen_bool(0.5)),
                );
                data.insert(
                    "motion_detected".to_string(),
                    serde_json::json!(rng.gen_bool(0.1)),
                );
                data.insert(
                    "fps".to_string(),
                    serde_json::json!(rng.gen_range(20..30)),
                );
            }
            _ => {
                data.insert(
                    "value".to_string(),
                    serde_json::json!(rng.gen_range(0..100)),
                );
            }
        }

        data
    }

    /// Simulate receiving and executing a command
    pub async fn simulate_command(&mut self) -> Result<()> {
        debug!("Simulating command for device: {}", self.name);
        // In a real implementation, this would listen for commands from the hub
        // For now, we just simulate random state changes
        Ok(())
    }
}
