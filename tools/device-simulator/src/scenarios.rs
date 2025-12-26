// ! Simulation Scenarios

use anyhow::Result;
use std::time::Duration;
use tracing::info;

use crate::device::MockDevice;

pub struct Scenario {
    name: String,
    devices: Vec<MockDevice>,
}

impl Scenario {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            devices: vec![],
        }
    }

    pub fn add_device(&mut self, device: MockDevice) {
        self.devices.push(device);
    }

    pub async fn run(&mut self, duration: u64) -> Result<()> {
        info!("🎬 Starting scenario: {}", self.name);
        info!("📱 Devices: {}", self.devices.len());

        // Register all devices
        for device in &mut self.devices {
            device.register().await?;
        }

        info!("✅ All devices registered");

        // Run simulation
        let start = std::time::Instant::now();
        let duration_secs = Duration::from_secs(duration);

        while start.elapsed() < duration_secs {
            for device in &mut self.devices {
                device.send_telemetry().await?;
            }
            tokio::time::sleep(Duration::from_secs(10)).await;
        }

        // Unregister all devices
        for device in &mut self.devices {
            device.unregister().await?;
        }

        info!("✅ Scenario completed: {}", self.name);
        Ok(())
    }
}

/// Home automation scenario
pub fn create_home_scenario(hub_url: &str) -> Result<Scenario> {
    let mut scenario = Scenario::new("Home Automation");

    // Living room
    scenario.add_device(MockDevice::new(hub_url, "smart-bulb", "living-room-light")?);
    scenario.add_device(MockDevice::new(hub_url, "thermostat", "living-room-thermostat")?);

    // Bedroom
    scenario.add_device(MockDevice::new(hub_url, "smart-bulb", "bedroom-light")?);
    scenario.add_device(MockDevice::new(hub_url, "thermostat", "bedroom-thermostat")?);

    // Security
    scenario.add_device(MockDevice::new(hub_url, "camera", "front-door-camera")?);
    scenario.add_device(MockDevice::new(hub_url, "camera", "backyard-camera")?);

    Ok(scenario)
}

/// Industrial IoT scenario
pub fn create_industrial_scenario(hub_url: &str) -> Result<Scenario> {
    let mut scenario = Scenario::new("Industrial IoT");

    // Factory sensors
    for i in 1..=10 {
        scenario.add_device(MockDevice::new(
            hub_url,
            "temperature-sensor",
            &format!("factory-temp-{}", i),
        )?);
    }

    // Monitoring cameras
    for i in 1..=5 {
        scenario.add_device(MockDevice::new(
            hub_url,
            "camera",
            &format!("factory-cam-{}", i),
        )?);
    }

    Ok(scenario)
}

/// Smart city scenario
pub fn create_smart_city_scenario(hub_url: &str) -> Result<Scenario> {
    let mut scenario = Scenario::new("Smart City");

    // Street lights
    for i in 1..=20 {
        scenario.add_device(MockDevice::new(
            hub_url,
            "smart-bulb",
            &format!("street-light-{}", i),
        )?);
    }

    // Traffic cameras
    for i in 1..=10 {
        scenario.add_device(MockDevice::new(
            hub_url,
            "camera",
            &format!("traffic-cam-{}", i),
        )?);
    }

    Ok(scenario)
}

/// Healthcare scenario
pub fn create_healthcare_scenario(hub_url: &str) -> Result<Scenario> {
    let mut scenario = Scenario::new("Healthcare Monitoring");

    // Patient rooms
    for i in 1..=15 {
        scenario.add_device(MockDevice::new(
            hub_url,
            "health-monitor",
            &format!("room-{}-monitor", i),
        )?);
        scenario.add_device(MockDevice::new(
            hub_url,
            "thermostat",
            &format!("room-{}-climate", i),
        )?);
    }

    Ok(scenario)
}
