//! Metrics handler for Prometheus endpoint

use axum::{http::StatusCode, response::IntoResponse};

/// GET /metrics - Prometheus metrics endpoint
pub async fn metrics_handler() -> impl IntoResponse {
    match crate::metrics::Metrics::gather_metrics() {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to gather metrics: {}", e),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_handler() {
        let response = metrics_handler().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
