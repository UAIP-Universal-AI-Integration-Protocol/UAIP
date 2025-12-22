//! Prometheus Metrics for UAIP Hub
//!
//! This module provides Prometheus metrics collection for monitoring
//! hub performance, device activity, and system health.

use lazy_static::lazy_static;
use prometheus::{
    register_counter_vec, register_gauge_vec, register_histogram_vec, CounterVec, Encoder,
    GaugeVec, HistogramVec, TextEncoder,
};

lazy_static! {
    /// Total number of HTTP requests
    pub static ref HTTP_REQUESTS_TOTAL: CounterVec = register_counter_vec!(
        "uaip_http_requests_total",
        "Total number of HTTP requests",
        &["method", "endpoint", "status"]
    )
    .unwrap();

    /// HTTP request duration in seconds
    pub static ref HTTP_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "uaip_http_request_duration_seconds",
        "HTTP request duration in seconds",
        &["method", "endpoint"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .unwrap();

    /// Total number of device commands
    pub static ref DEVICE_COMMANDS_TOTAL: CounterVec = register_counter_vec!(
        "uaip_device_commands_total",
        "Total number of device commands",
        &["device_id", "action", "status"]
    )
    .unwrap();

    /// Device command execution duration
    pub static ref DEVICE_COMMAND_DURATION: HistogramVec = register_histogram_vec!(
        "uaip_device_command_duration_seconds",
        "Device command execution duration in seconds",
        &["device_id", "action"],
        vec![0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
    )
    .unwrap();

    /// Number of active WebSocket connections
    pub static ref WEBSOCKET_CONNECTIONS: GaugeVec = register_gauge_vec!(
        "uaip_websocket_connections",
        "Number of active WebSocket connections",
        &["status"]
    )
    .unwrap();

    /// Total WebSocket messages sent/received
    pub static ref WEBSOCKET_MESSAGES_TOTAL: CounterVec = register_counter_vec!(
        "uaip_websocket_messages_total",
        "Total number of WebSocket messages",
        &["direction", "message_type"]
    )
    .unwrap();

    /// Number of registered devices by status
    pub static ref DEVICES_COUNT: GaugeVec = register_gauge_vec!(
        "uaip_devices_count",
        "Number of registered devices by status",
        &["status"]
    )
    .unwrap();

    /// Device heartbeat latency
    pub static ref DEVICE_HEARTBEAT_LATENCY: HistogramVec = register_histogram_vec!(
        "uaip_device_heartbeat_latency_seconds",
        "Device heartbeat latency in seconds",
        &["device_id"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]
    )
    .unwrap();

    /// Message queue depth by priority
    pub static ref MESSAGE_QUEUE_DEPTH: GaugeVec = register_gauge_vec!(
        "uaip_message_queue_depth",
        "Message queue depth by priority",
        &["priority"]
    )
    .unwrap();

    /// Message routing errors
    pub static ref MESSAGE_ROUTING_ERRORS: CounterVec = register_counter_vec!(
        "uaip_message_routing_errors_total",
        "Total number of message routing errors",
        &["error_type"]
    )
    .unwrap();

    /// Authentication attempts
    pub static ref AUTH_ATTEMPTS_TOTAL: CounterVec = register_counter_vec!(
        "uaip_auth_attempts_total",
        "Total number of authentication attempts",
        &["method", "status"]
    )
    .unwrap();

    /// Active sessions
    pub static ref ACTIVE_SESSIONS: GaugeVec = register_gauge_vec!(
        "uaip_active_sessions",
        "Number of active sessions",
        &["session_type"]
    )
    .unwrap();

    /// Database query duration
    pub static ref DB_QUERY_DURATION: HistogramVec = register_histogram_vec!(
        "uaip_db_query_duration_seconds",
        "Database query duration in seconds",
        &["query_type"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]
    )
    .unwrap();

    /// Redis operation duration
    pub static ref REDIS_OP_DURATION: HistogramVec = register_histogram_vec!(
        "uaip_redis_operation_duration_seconds",
        "Redis operation duration in seconds",
        &["operation"],
        vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.025, 0.05]
    )
    .unwrap();

    /// Cache hit/miss counters
    pub static ref CACHE_OPERATIONS: CounterVec = register_counter_vec!(
        "uaip_cache_operations_total",
        "Total number of cache operations",
        &["operation", "result"]
    )
    .unwrap();

    /// System resource usage
    pub static ref SYSTEM_RESOURCES: GaugeVec = register_gauge_vec!(
        "uaip_system_resources",
        "System resource usage",
        &["resource"]
    )
    .unwrap();
}

/// Metrics helper functions
pub struct Metrics;

impl Metrics {
    /// Record an HTTP request
    pub fn record_http_request(method: &str, endpoint: &str, status: u16, duration: f64) {
        HTTP_REQUESTS_TOTAL
            .with_label_values(&[method, endpoint, &status.to_string()])
            .inc();

        HTTP_REQUEST_DURATION
            .with_label_values(&[method, endpoint])
            .observe(duration);
    }

    /// Record a device command
    pub fn record_device_command(device_id: &str, action: &str, status: &str, duration: f64) {
        DEVICE_COMMANDS_TOTAL
            .with_label_values(&[device_id, action, status])
            .inc();

        DEVICE_COMMAND_DURATION
            .with_label_values(&[device_id, action])
            .observe(duration);
    }

    /// Update WebSocket connection count
    pub fn update_websocket_connections(status: &str, delta: i64) {
        if delta > 0 {
            WEBSOCKET_CONNECTIONS
                .with_label_values(&[status])
                .add(delta as f64);
        } else {
            WEBSOCKET_CONNECTIONS
                .with_label_values(&[status])
                .sub((-delta) as f64);
        }
    }

    /// Record a WebSocket message
    pub fn record_websocket_message(direction: &str, message_type: &str) {
        WEBSOCKET_MESSAGES_TOTAL
            .with_label_values(&[direction, message_type])
            .inc();
    }

    /// Update device count
    pub fn update_device_count(status: &str, count: f64) {
        DEVICES_COUNT.with_label_values(&[status]).set(count);
    }

    /// Record device heartbeat latency
    pub fn record_heartbeat_latency(device_id: &str, latency: f64) {
        DEVICE_HEARTBEAT_LATENCY
            .with_label_values(&[device_id])
            .observe(latency);
    }

    /// Update message queue depth
    pub fn update_queue_depth(priority: &str, depth: f64) {
        MESSAGE_QUEUE_DEPTH
            .with_label_values(&[priority])
            .set(depth);
    }

    /// Record message routing error
    pub fn record_routing_error(error_type: &str) {
        MESSAGE_ROUTING_ERRORS
            .with_label_values(&[error_type])
            .inc();
    }

    /// Record authentication attempt
    pub fn record_auth_attempt(method: &str, status: &str) {
        AUTH_ATTEMPTS_TOTAL
            .with_label_values(&[method, status])
            .inc();
    }

    /// Update active sessions
    pub fn update_active_sessions(session_type: &str, count: f64) {
        ACTIVE_SESSIONS
            .with_label_values(&[session_type])
            .set(count);
    }

    /// Record database query duration
    pub fn record_db_query(query_type: &str, duration: f64) {
        DB_QUERY_DURATION
            .with_label_values(&[query_type])
            .observe(duration);
    }

    /// Record Redis operation duration
    pub fn record_redis_op(operation: &str, duration: f64) {
        REDIS_OP_DURATION
            .with_label_values(&[operation])
            .observe(duration);
    }

    /// Record cache operation
    pub fn record_cache_op(operation: &str, result: &str) {
        CACHE_OPERATIONS
            .with_label_values(&[operation, result])
            .inc();
    }

    /// Update system resource usage
    pub fn update_system_resource(resource: &str, value: f64) {
        SYSTEM_RESOURCES.with_label_values(&[resource]).set(value);
    }

    /// Gather all metrics and encode as Prometheus text format
    pub fn gather_metrics() -> Result<String, String> {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = vec![];

        encoder
            .encode(&metric_families, &mut buffer)
            .map_err(|e| format!("Failed to encode metrics: {}", e))?;

        String::from_utf8(buffer).map_err(|e| format!("Failed to convert metrics to string: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_http_request() {
        Metrics::record_http_request("GET", "/api/health", 200, 0.05);
        // Verify metric was recorded (basic smoke test)
        let metrics = Metrics::gather_metrics().expect("Should gather metrics");
        assert!(metrics.contains("uaip_http_requests_total"));
    }

    #[test]
    fn test_record_device_command() {
        Metrics::record_device_command("device_001", "turn_on", "success", 0.1);
        let metrics = Metrics::gather_metrics().expect("Should gather metrics");
        assert!(metrics.contains("uaip_device_commands_total"));
    }

    #[test]
    fn test_websocket_metrics() {
        Metrics::update_websocket_connections("active", 1);
        Metrics::record_websocket_message("inbound", "telemetry");
        let metrics = Metrics::gather_metrics().expect("Should gather metrics");
        assert!(metrics.contains("uaip_websocket"));
    }

    #[test]
    fn test_gather_metrics() {
        // Record at least one metric to ensure output is not empty
        Metrics::record_http_request("GET", "/test", 200, 0.01);

        let result = Metrics::gather_metrics();
        assert!(result.is_ok());
        let metrics_text = result.unwrap();

        // Metrics should contain at least some data
        assert!(!metrics_text.is_empty());
        assert!(metrics_text.contains("uaip_http_requests_total"));
    }
}
