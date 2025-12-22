//! UAIP Hub - Main Hub Service
//!
//! The central orchestration service that coordinates all components.

use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use uaip_hub::{
    api::rest::{create_router, AppState},
    health::HealthChecker,
    middleware::RateLimitLayer,
    shutdown::shutdown_signal,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging with tracing
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "uaip_hub=info,tower_http=info,axum=info".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(true)
                .with_thread_names(true),
        )
        .init();

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "UAIP Hub starting");

    // Create application state
    let state = Arc::new(AppState::new());

    // Create health checker
    let _health_checker = Arc::new(HealthChecker::new());

    // Create rate limiter
    let rate_limiter = RateLimitLayer::new(Default::default());

    // Spawn rate limiter cleanup task
    let cleanup_limiter = rate_limiter.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
        loop {
            interval.tick().await;
            cleanup_limiter.cleanup_old_buckets().await;
            tracing::debug!("Rate limiter buckets cleaned up");
        }
    });

    // Create router with all middleware
    let app = create_router(state);

    // Bind to address
    let addr = SocketAddr::from(([127, 0, 0, 1], 8443));
    tracing::info!(
        address = %addr,
        "HTTP server listening"
    );

    // Start server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("UAIP Hub shut down gracefully");

    Ok(())
}
