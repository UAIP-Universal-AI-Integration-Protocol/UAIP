// API Integration Tests

use integration_tests::*;
use reqwest::StatusCode;

#[tokio::test]
async fn test_health_check() {
    let ctx = TestContext::new().await.expect("Failed to create context");
    let client = ctx.client();

    let response = client.get("/api/v1/system/health").await.expect("Request failed");

    let response = assert_status(response, StatusCode::OK).await;
    let health: HealthResponse = assert_json(response).await;

    assert_eq!(health.status, "healthy");
    assert!(!health.version.is_empty());
}

#[tokio::test]
async fn test_liveness_probe() {
    let ctx = TestContext::new().await.expect("Failed to create context");
    let client = ctx.client();

    let response = client
        .get("/api/v1/system/health/liveness")
        .await
        .expect("Request failed");

    assert_status(response, StatusCode::OK).await;
}

#[tokio::test]
async fn test_readiness_probe() {
    let ctx = TestContext::new().await.expect("Failed to create context");
    let client = ctx.client();

    let response = client
        .get("/api/v1/system/health/readiness")
        .await
        .expect("Request failed");

    // Should be OK if all dependencies are healthy
    let status = response.status();
    assert!(
        status == StatusCode::OK || status == StatusCode::SERVICE_UNAVAILABLE,
        "Expected 200 or 503, got {}",
        status
    );
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let ctx = TestContext::new().await.expect("Failed to create context");
    let client = ctx.client();

    let response = client.get("/metrics").await.expect("Request failed");

    let response = assert_status(response, StatusCode::OK).await;
    let body = response.text().await.expect("Failed to read body");

    // Check for Prometheus metrics format
    assert!(body.contains("# HELP"));
    assert!(body.contains("# TYPE"));
}

#[tokio::test]
async fn test_cors_headers() {
    let ctx = TestContext::new().await.expect("Failed to create context");
    let client = ctx.client();

    let response = client.get("/api/v1/system/health").await.expect("Request failed");

    // Check CORS headers are present
    let headers = response.headers();
    assert!(headers.contains_key("access-control-allow-origin"));
}

#[tokio::test]
async fn test_request_id_header() {
    let ctx = TestContext::new().await.expect("Failed to create context");
    let client = ctx.client();

    let response = client.get("/api/v1/system/health").await.expect("Request failed");

    // Check request ID header is present
    let headers = response.headers();
    assert!(
        headers.contains_key("x-request-id"),
        "Expected x-request-id header"
    );
}

#[tokio::test]
async fn test_404_not_found() {
    let ctx = TestContext::new().await.expect("Failed to create context");
    let client = ctx.client();

    let response = client
        .get("/api/v1/nonexistent")
        .await
        .expect("Request failed");

    assert_status(response, StatusCode::NOT_FOUND).await;
}

#[tokio::test]
async fn test_method_not_allowed() {
    let ctx = TestContext::new().await.expect("Failed to create context");
    let client = ctx.client();

    // Try POST on a GET-only endpoint
    let response = client
        .post("/api/v1/system/health", &serde_json::json!({}))
        .await
        .expect("Request failed");

    let status = response.status();
    assert!(
        status == StatusCode::METHOD_NOT_ALLOWED || status == StatusCode::NOT_FOUND,
        "Expected 405 or 404, got {}",
        status
    );
}
