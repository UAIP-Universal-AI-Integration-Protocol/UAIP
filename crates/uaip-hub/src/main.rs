//! UAIP Hub - Main Hub Service
//!
//! The central orchestration service that coordinates all components.

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod config;
mod handlers;
mod telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "uaip_hub=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("UAIP Hub starting...");

    // TODO: Load configuration
    // TODO: Initialize database connections
    // TODO: Start HTTP/WebSocket servers

    tracing::info!("UAIP Hub initialized successfully");

    Ok(())
}
