//! Request handlers module

use axum::{Extension, Json};
use std::sync::Arc;

pub mod auth;
pub mod commands;
pub mod devices;
pub mod metrics;

use crate::health::{HealthCheckResponse, HealthChecker};

/// Health check handler
pub async fn health_check(
    Extension(checker): Extension<Arc<HealthChecker>>,
) -> Json<HealthCheckResponse> {
    let health = checker.check_health().await;
    Json(health)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let checker = Arc::new(HealthChecker::new());
        let response = health_check(Extension(checker)).await;

        assert!(!response.0.version.is_empty());
        assert!(!response.0.timestamp.is_empty());
        assert_eq!(response.0.dependencies.len(), 3); // PostgreSQL, Redis, NATS
    }
}
