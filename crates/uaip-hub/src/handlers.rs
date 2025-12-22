//! Request handlers module

use axum::Json;

pub mod auth;
pub mod commands;
pub mod devices;
pub mod metrics;

use crate::api::rest::HealthResponse;

/// Health check handler
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        assert_eq!(response.0.status, "healthy");
        assert!(!response.0.version.is_empty());
        assert!(!response.0.timestamp.is_empty());
    }
}
