// ! UAIP Mock Device Simulator
//!
//! Simulates IoT devices for testing and load testing the UAIP Hub

use anyhow::Result;
use clap::{Parser, Subcommand};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info, warn};

mod device;
mod scenarios;

use device::MockDevice;
use scenarios::Scenario;

#[derive(Parser)]
#[command(name = "device-simulator")]
#[command(about = "UAIP Mock Device Simulator", long_about = None)]
struct Cli {
    /// UAIP Hub URL
    #[arg(short, long, env = "UAIP_HUB_URL", default_value = "http://localhost:8443")]
    hub_url: String,

    /// Number of devices to simulate
    #[arg(short = 'n', long, default_value = "10")]
    num_devices: usize,

    /// Simulation duration in seconds (0 = infinite)
    #[arg(short, long, default_value = "60")]
    duration: u64,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a single device simulation
    Single {
        /// Device type
        #[arg(short, long, default_value = "smart-bulb")]
        device_type: String,

        /// Device name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Run multiple devices simulation
    Multi {
        /// Device types (comma-separated)
        #[arg(short, long, default_value = "smart-bulb,thermostat,camera")]
        types: String,
    },

    /// Run load test scenario
    LoadTest {
        /// Requests per second
        #[arg(short, long, default_value = "100")]
        rps: usize,

        /// Number of concurrent devices
        #[arg(short = 'c', long, default_value = "50")]
        concurrency: usize,
    },

    /// Run specific scenario
    Scenario {
        /// Scenario name
        #[arg(short, long, value_enum)]
        name: ScenarioType,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum ScenarioType {
    /// Home automation scenario
    Home,
    /// Industrial IoT scenario
    Industrial,
    /// Smart city scenario
    SmartCity,
    /// Healthcare scenario
    Healthcare,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    let cli = Cli::parse();

    info!("🚀 UAIP Device Simulator Starting");
    info!("📡 Hub URL: {}", cli.hub_url);
    info!("⏱️  Duration: {}s", cli.duration);

    match cli.command {
        Commands::Single { device_type, name } => {
            run_single_device(&cli.hub_url, &device_type, name, cli.duration).await?;
        }
        Commands::Multi { types } => {
            run_multi_devices(&cli.hub_url, &types, cli.num_devices, cli.duration).await?;
        }
        Commands::LoadTest { rps, concurrency } => {
            run_load_test(&cli.hub_url, rps, concurrency, cli.duration).await?;
        }
        Commands::Scenario { name } => {
            run_scenario(&cli.hub_url, name, cli.duration).await?;
        }
    }

    info!("✅ Simulation completed successfully");
    Ok(())
}

/// Run a single device simulation
async fn run_single_device(
    hub_url: &str,
    device_type: &str,
    name: Option<String>,
    duration: u64,
) -> Result<()> {
    let device_name = name.unwrap_or_else(|| format!("mock-{}", device_type));
    let mut device = MockDevice::new(hub_url, device_type, &device_name)?;

    info!("🔌 Starting device: {} ({})", device_name, device_type);

    // Register device
    device.register().await?;
    info!("✅ Device registered: {}", device.id());

    // Run simulation
    let start = std::time::Instant::now();
    let duration_secs = Duration::from_secs(duration);

    while duration == 0 || start.elapsed() < duration_secs {
        // Send telemetry
        device.send_telemetry().await?;

        // Random command simulation
        if rand::thread_rng().gen_bool(0.1) {
            device.simulate_command().await?;
        }

        // Wait before next cycle
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    // Unregister device
    device.unregister().await?;
    info!("👋 Device unregistered");

    Ok(())
}

/// Run multiple devices simulation
async fn run_multi_devices(
    hub_url: &str,
    types: &str,
    num_devices: usize,
    duration: u64,
) -> Result<()> {
    let device_types: Vec<&str> = types.split(',').collect();
    info!(
        "🔌 Starting {} devices of types: {:?}",
        num_devices, device_types
    );

    let mut handles = vec![];

    for i in 0..num_devices {
        let device_type = device_types[i % device_types.len()];
        let device_name = format!("mock-{}-{}", device_type, i);
        let hub_url = hub_url.to_string();

        let handle = tokio::spawn(async move {
            if let Err(e) = run_device_task(&hub_url, device_type, &device_name, duration).await {
                error!("Device {} failed: {}", device_name, e);
            }
        });

        handles.push(handle);

        // Stagger device starts to avoid thundering herd
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Wait for all devices
    for handle in handles {
        handle.await?;
    }

    Ok(())
}

/// Run a single device task
async fn run_device_task(
    hub_url: &str,
    device_type: &str,
    device_name: &str,
    duration: u64,
) -> Result<()> {
    let mut device = MockDevice::new(hub_url, device_type, device_name)?;

    device.register().await?;

    let start = std::time::Instant::now();
    let duration_secs = Duration::from_secs(duration);

    while duration == 0 || start.elapsed() < duration_secs {
        device.send_telemetry().await?;
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    device.unregister().await?;
    Ok(())
}

/// Run load test
async fn run_load_test(hub_url: &str, rps: usize, concurrency: usize, duration: u64) -> Result<()> {
    info!(
        "🔥 Load Test: {} RPS, {} concurrent devices, {}s duration",
        rps, concurrency, duration
    );

    let interval = Duration::from_secs_f64(1.0 / rps as f64);
    let mut handles = vec![];

    // Create devices
    let mut devices = vec![];
    for i in 0..concurrency {
        let device = MockDevice::new(hub_url, "load-test", &format!("load-{}", i))?;
        devices.push(device);
    }

    // Register all devices
    for device in &mut devices {
        device.register().await?;
    }

    info!("✅ {} devices registered", concurrency);

    // Start load generation
    let start = std::time::Instant::now();
    let duration_secs = Duration::from_secs(duration);
    let mut requests_sent = 0;

    while start.elapsed() < duration_secs {
        let device_idx = requests_sent % concurrency;
        let mut device = devices[device_idx].clone();

        let handle = tokio::spawn(async move {
            if let Err(e) = device.send_telemetry().await {
                warn!("Telemetry failed: {}", e);
            }
        });

        handles.push(handle);
        requests_sent += 1;

        tokio::time::sleep(interval).await;
    }

    // Wait for all requests
    for handle in handles {
        handle.await?;
    }

    // Unregister devices
    for device in &mut devices {
        device.unregister().await?;
    }

    info!("📊 Load test complete: {} requests sent", requests_sent);
    Ok(())
}

/// Run a specific scenario
async fn run_scenario(hub_url: &str, scenario_type: ScenarioType, duration: u64) -> Result<()> {
    info!("🎬 Running scenario: {:?}", scenario_type);

    let scenario = match scenario_type {
        ScenarioType::Home => scenarios::create_home_scenario(hub_url)?,
        ScenarioType::Industrial => scenarios::create_industrial_scenario(hub_url)?,
        ScenarioType::SmartCity => scenarios::create_smart_city_scenario(hub_url)?,
        ScenarioType::Healthcare => scenarios::create_healthcare_scenario(hub_url)?,
    };

    scenario.run(duration).await?;

    Ok(())
}
