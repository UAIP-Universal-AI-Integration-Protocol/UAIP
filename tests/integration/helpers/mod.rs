// Integration Test Helpers for UAIP Hub
// Provides common test utilities, fixtures, and assertions

pub mod fixtures;
pub mod assertions;
pub mod client;
pub mod database;

use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize tracing for tests (call once)
pub static TRACING: Lazy<()> = Lazy::new(|| {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "integration_tests=debug,uaip=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_test_writer())
        .init();
});

/// Test context with shared resources
pub struct TestContext {
    pub base_url: String,
    pub db_pool: sqlx::PgPool,
    pub redis_client: redis::Client,
    pub http_client: reqwest::Client,
}

impl TestContext {
    /// Create a new test context
    pub async fn new() -> anyhow::Result<Self> {
        Lazy::force(&TRACING);

        let base_url = std::env::var("TEST_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8443".to_string());

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://uaip:uaip_test_password@localhost:5432/uaip_test".to_string());

        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let db_pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        let redis_client = redis::Client::open(redis_url)?;

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            base_url,
            db_pool,
            redis_client,
            http_client,
        })
    }

    /// Clean up test data
    pub async fn cleanup(&self) -> anyhow::Result<()> {
        // Clean database
        sqlx::query("TRUNCATE TABLE messages, devices, user_roles, users, roles CASCADE")
            .execute(&self.db_pool)
            .await?;

        // Clean Redis
        let mut conn = self.redis_client.get_connection()?;
        redis::cmd("FLUSHDB").execute(&mut conn);

        Ok(())
    }

    /// Get HTTP client with base URL
    pub fn client(&self) -> client::ApiClient {
        client::ApiClient::new(self.http_client.clone(), self.base_url.clone())
    }
}

/// Shared test context (use for parallel tests)
pub static TEST_CONTEXT: Lazy<Arc<Mutex<Option<TestContext>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

/// Get or create test context
pub async fn get_test_context() -> Arc<TestContext> {
    let mut guard = TEST_CONTEXT.lock().await;
    if guard.is_none() {
        *guard = Some(TestContext::new().await.expect("Failed to create test context"));
    }
    Arc::new(guard.as_ref().unwrap().clone())
}

// Re-export common items
pub use assertions::*;
pub use fixtures::*;
pub use client::*;
