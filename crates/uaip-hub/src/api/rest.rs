//! REST API endpoints

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use uaip_core::error::{ErrorResponse, UaipError};

/// Result type for API handlers
pub type ApiResult<T> = Result<T, ApiError>;

use crate::api::websocket;
use crate::handlers;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db_pool: Option<sqlx::PgPool>,
    pub redis_client: Option<redis::Client>,
    pub nats_client: Option<async_nats::Client>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            db_pool: None,
            redis_client: None,
            nats_client: None,
        }
    }

    pub fn with_db(mut self, pool: sqlx::PgPool) -> Self {
        self.db_pool = Some(pool);
        self
    }

    pub fn with_redis(mut self, client: redis::Client) -> Self {
        self.redis_client = Some(client);
        self
    }

    pub fn with_nats(mut self, client: async_nats::Client) -> Self {
        self.nats_client = Some(client);
        self
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create the REST API router
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health check
        .route("/api/v1/system/health", get(handlers::health_check))
        // Metrics endpoint for Prometheus
        .route("/metrics", get(handlers::metrics::metrics_handler))
        // Authentication
        .route("/api/v1/auth/login", post(handlers::auth::login))
        .route("/api/v1/auth/register", post(handlers::auth::register))
        .route("/api/v1/auth/change-password", post(handlers::auth::change_password))
        // User Management
        .route("/api/v1/users", get(handlers::users::list_users))
        .route("/api/v1/users/register", post(handlers::users::create_user))
        .route("/api/v1/users/:id", delete(handlers::users::delete_user))
        .route("/api/v1/users/:id", axum::routing::put(handlers::users::update_user))
        .route("/api/v1/users/:id/password", axum::routing::put(handlers::users::admin_reset_password))
        .route("/api/v1/users/:id/status", post(handlers::users::update_user_status))
        // Devices
        .route("/api/v1/devices", get(handlers::devices::list_devices))
        .route(
            "/api/v1/devices/register",
            post(handlers::devices::register_device),
        )
        .route(
            "/api/v1/devices/:id/command",
            post(handlers::devices::send_command),
        )
        // Protocol Adapters
        .route("/api/v1/adapters", get(handlers::adapters::list_adapters))
        .route(
            "/api/v1/adapters/http/test",
            post(handlers::adapters::test_http_adapter),
        )
        .route(
            "/api/v1/adapters/modbus/test",
            post(handlers::adapters::test_modbus_adapter),
        )
        .route(
            "/api/v1/adapters/modbus/read",
            post(handlers::adapters::read_modbus_registers),
        )
        .route(
            "/api/v1/adapters/opcua/test",
            post(handlers::adapters::test_opcua_adapter),
        )
        .route(
            "/api/v1/adapters/opcua/read",
            post(handlers::adapters::read_opcua_node),
        )
        .route(
            "/api/v1/adapters/webrtc/offer",
            post(handlers::adapters::create_webrtc_offer),
        )
        // AI Agents
        .route(
            "/api/v1/ai/agents/register",
            post(handlers::ai::register_ai_agent),
        )
        .route("/api/v1/ai/agents", get(handlers::ai::list_ai_agents))
        .route("/api/v1/ai/sessions", post(handlers::ai::create_ai_session))
        .route(
            "/api/v1/ai/sessions/:session_id",
            get(handlers::ai::get_ai_session),
        )
        // Media Management
        .route("/api/v1/media/upload", post(handlers::media::upload_media))
        .route("/api/v1/media", get(handlers::media::list_media))
        .route("/api/v1/media/:id", get(handlers::media::get_media))
        .route("/api/v1/media/:id", delete(handlers::media::delete_media))
        // Streaming
        .route(
            "/api/v1/streaming/sessions",
            post(handlers::media::create_stream_session),
        )
        .route(
            "/api/v1/streaming/sessions/:id",
            get(handlers::media::get_stream_session),
        )
        // WebSocket
        .route("/ws", get(websocket::ws_handler))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(state)
}

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: String,
}

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub grant_type: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub scope: Option<String>,
}

/// Registration request
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub require_password_change: bool,
}

/// Device registration request
#[derive(Debug, Deserialize)]
pub struct DeviceRegistrationRequest {
    pub device_id: String,
    pub device_type: String,
    pub name: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub capabilities: Vec<String>,
}

/// Device registration response
#[derive(Debug, Serialize)]
pub struct DeviceRegistrationResponse {
    pub device_id: String,
    pub challenge: String,
    pub expires_at: String,
}

/// Device list response
#[derive(Debug, Serialize)]
pub struct DeviceListResponse {
    pub devices: Vec<DeviceInfo>,
    pub total: usize,
}

/// Device information
#[derive(Debug, Serialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub name: String,
    pub device_type: String,
    pub status: String,
    pub last_seen: Option<String>,
}

/// Command request
#[derive(Debug, Deserialize)]
pub struct CommandRequest {
    pub action: String,
    pub parameters: Option<serde_json::Value>,
    pub priority: Option<String>,
}

/// Command response
#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub message_id: String,
    pub status: String,
    pub queued_at: String,
}

/// Error response wrapper for HTTP responses
#[derive(Debug)]
pub struct ApiError(pub UaipError);

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let error_response: ErrorResponse = self.0.into();

        let status = match error_response.code {
            uaip_core::error::ErrorCode::AuthenticationFailed => StatusCode::UNAUTHORIZED,
            uaip_core::error::ErrorCode::AuthorizationFailed => StatusCode::FORBIDDEN,
            uaip_core::error::ErrorCode::DeviceNotFound => StatusCode::NOT_FOUND,
            uaip_core::error::ErrorCode::InvalidParameter => StatusCode::BAD_REQUEST,
            uaip_core::error::ErrorCode::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(error_response)).into_response()
    }
}

impl From<UaipError> for ApiError {
    fn from(error: UaipError) -> Self {
        ApiError(error)
    }
}

impl ApiError {
    /// Create a bad request error
    pub fn bad_request(message: String) -> Self {
        ApiError(UaipError::InvalidParameter(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();
        assert!(state.db_pool.is_none());
        assert!(state.redis_client.is_none());
        assert!(state.nats_client.is_none());
    }

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("1.0.0"));
    }

    #[test]
    fn test_login_request_deserialization() {
        let json =
            r#"{"grant_type":"client_credentials","client_id":"test","scope":"device:read"}"#;
        let request: LoginRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.grant_type, "client_credentials");
        assert_eq!(request.client_id, "test");
        assert_eq!(request.scope, Some("device:read".to_string()));
    }
}
