//! Structured logging middleware with request tracking
//!
//! Provides comprehensive request/response logging with correlation IDs

use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Request ID header name
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Logging middleware that tracks requests with correlation IDs
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();

    // Generate or extract request ID
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Log request start
    info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        "Request started"
    );

    // Process request
    let response = next.run(request).await;

    // Calculate duration
    let duration = start.elapsed();
    let duration_ms = duration.as_secs_f64() * 1000.0;

    // Get status code
    let status = response.status();

    // Log based on status code
    if status.is_success() {
        info!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = %format!("{:.2}", duration_ms),
            "Request completed successfully"
        );
    } else if status.is_client_error() {
        warn!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = %format!("{:.2}", duration_ms),
            "Request failed with client error"
        );
    } else if status.is_server_error() {
        error!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = %format!("{:.2}", duration_ms),
            "Request failed with server error"
        );
    } else {
        info!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = %format!("{:.2}", duration_ms),
            "Request completed"
        );
    }

    // Record metrics
    crate::metrics::Metrics::record_http_request(
        method.as_str(),
        &path,
        status.as_u16(),
        duration.as_secs_f64(),
    );

    response
}

/// Error logging helper
pub fn log_error(error: &dyn std::error::Error, context: &str) {
    error!(
        error = %error,
        context = %context,
        "Error occurred"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_header() {
        assert_eq!(REQUEST_ID_HEADER, "x-request-id");
    }

    #[test]
    fn test_log_error() {
        let error = std::io::Error::other("test error");
        log_error(&error, "test context");
        // Should not panic
    }
}
