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
    // Load environment variables
    dotenvy::dotenv().ok();

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

    // Initialize database connection (optional)
    let db_pool = match std::env::var("DATABASE_URL") {
        Ok(url) => {
            tracing::info!("Connecting to PostgreSQL database...");
            match sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&url)
                .await
            {
                Ok(pool) => {
                    tracing::info!("PostgreSQL connection established");
                    Some(pool)
                }
                Err(e) => {
                    tracing::warn!("Failed to connect to PostgreSQL: {}", e);
                    tracing::warn!("Continuing without database connection");
                    None
                }
            }
        }
        Err(_) => {
            tracing::info!("DATABASE_URL not set, skipping PostgreSQL connection");
            None
        }
    };

    // Initialize Redis connection (optional)
    let redis_client = match std::env::var("REDIS_URL") {
        Ok(url) => {
            tracing::info!("Connecting to Redis...");
            match redis::Client::open(url.as_str()) {
                Ok(client) => {
                    // Test the connection
                    match client.get_multiplexed_async_connection().await {
                        Ok(_) => {
                            tracing::info!("Redis connection established");
                            Some(client)
                        }
                        Err(e) => {
                            tracing::warn!("Failed to connect to Redis: {}", e);
                            tracing::warn!("Continuing without Redis connection");
                            None
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to create Redis client: {}", e);
                    None
                }
            }
        }
        Err(_) => {
            tracing::info!("REDIS_URL not set, skipping Redis connection");
            None
        }
    };

    // Initialize NATS connection (optional)
    let nats_client = match std::env::var("NATS_URL") {
        Ok(url) => {
            tracing::info!("Connecting to NATS...");
            match async_nats::connect(&url).await {
                Ok(client) => {
                    tracing::info!("NATS connection established");
                    Some(client)
                }
                Err(e) => {
                    tracing::warn!("Failed to connect to NATS: {}", e);
                    tracing::warn!("Continuing without NATS connection");
                    None
                }
            }
        }
        Err(_) => {
            tracing::info!("NATS_URL not set, skipping NATS connection");
            None
        }
    };

    // Create application state with connections
    let mut state = AppState::new();
    if let Some(pool) = db_pool.clone() {
        state = state.with_db(pool);
    }
    if let Some(client) = redis_client.clone() {
        state = state.with_redis(client);
    }
    if let Some(client) = nats_client.clone() {
        state = state.with_nats(client);
    }
    let state = Arc::new(state);

    // Create health checker with connections
    let mut health_checker = HealthChecker::new();
    if let Some(pool) = db_pool {
        health_checker = health_checker.with_db(pool);
    }
    if let Some(client) = redis_client {
        health_checker = health_checker.with_redis(client);
    }
    if let Some(client) = nats_client {
        health_checker = health_checker.with_nats(client);
    }
    let health_checker = Arc::new(health_checker);

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
    let app = create_router(state).layer(axum::Extension(health_checker));

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
