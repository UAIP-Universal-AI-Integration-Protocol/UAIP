//! UAIP Hub - Main Hub Service
//!
//! The central orchestration service that coordinates all components.

use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use uaip_hub::api::rest::{create_router, AppState};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "uaip_hub=debug,tower_http=debug,axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("UAIP Hub starting...");

    // Create application state
    let state = Arc::new(AppState::new());

    // Create router
    let app = create_router(state);

    // Bind to address
    let addr = SocketAddr::from(([127, 0, 0, 1], 8443));
    tracing::info!("Starting HTTP server on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    tracing::info!("UAIP Hub shut down gracefully");

    Ok(())
}
