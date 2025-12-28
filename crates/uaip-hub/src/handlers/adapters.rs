//! Protocol Adapters Management Handlers
//!
//! REST API endpoints for managing and interacting with protocol adapters
//! (ModBus, OPC UA, WebRTC, HTTP, MQTT, WebSocket).

use axum::{
    extract::State,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

use uaip_adapters::{
    http::{HttpAdapter, HttpConfig},
    modbus::{ModbusAdapter, ModbusConfig},
    opcua::{NodeId, OpcUaAdapter, OpcUaConfig, OpcValue},
    webrtc::{DataChannelConfig, WebRtcAdapter, WebRtcConfig},
};

use crate::api::rest::{ApiError, ApiResult, AppState};

/// List all available protocol adapters
pub async fn list_adapters(
    State(_state): State<Arc<AppState>>,
) -> ApiResult<Json<AdapterListResponse>> {
    info!("Listing available protocol adapters");

    let adapters = vec![
        AdapterInfo {
            adapter_type: "http".to_string(),
            name: "HTTP/REST Client".to_string(),
            description: "HTTP client for REST API communication".to_string(),
            supported_operations: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
            ],
            status: "available".to_string(),
        },
        AdapterInfo {
            adapter_type: "websocket".to_string(),
            name: "WebSocket Client".to_string(),
            description: "Real-time bidirectional WebSocket communication".to_string(),
            supported_operations: vec!["connect".to_string(), "send".to_string()],
            status: "available".to_string(),
        },
        AdapterInfo {
            adapter_type: "mqtt".to_string(),
            name: "MQTT Client".to_string(),
            description: "MQTT publish/subscribe messaging".to_string(),
            supported_operations: vec![
                "connect".to_string(),
                "subscribe".to_string(),
                "publish".to_string(),
            ],
            status: "available".to_string(),
        },
        AdapterInfo {
            adapter_type: "modbus".to_string(),
            name: "Modbus TCP Client".to_string(),
            description: "Industrial Modbus TCP protocol client".to_string(),
            supported_operations: vec![
                "read_coils".to_string(),
                "read_holding_registers".to_string(),
                "write_single_coil".to_string(),
                "write_single_register".to_string(),
            ],
            status: "available".to_string(),
        },
        AdapterInfo {
            adapter_type: "opcua".to_string(),
            name: "OPC UA Client".to_string(),
            description: "Industrial automation OPC UA protocol client".to_string(),
            supported_operations: vec![
                "connect".to_string(),
                "read_node".to_string(),
                "write_node".to_string(),
                "browse_node".to_string(),
            ],
            status: "available".to_string(),
        },
        AdapterInfo {
            adapter_type: "webrtc".to_string(),
            name: "WebRTC Peer".to_string(),
            description: "Real-time peer-to-peer communication with data channels".to_string(),
            supported_operations: vec![
                "create_offer".to_string(),
                "create_answer".to_string(),
                "create_data_channel".to_string(),
            ],
            status: "available".to_string(),
        },
    ];

    Ok(Json(AdapterListResponse {
        adapters,
        total: 6,
    }))
}

/// Test HTTP adapter connection
pub async fn test_http_adapter(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<HttpTestRequest>,
) -> ApiResult<Json<AdapterTestResponse>> {
    info!("Testing HTTP adapter connection to: {}", request.base_url);

    let config = HttpConfig {
        base_url: request.base_url.clone(),
        timeout_seconds: request.timeout_seconds.unwrap_or(10),
        max_retries: 1,
        retry_delay_ms: 1000,
        default_headers: request.headers.unwrap_or_default(),
        auth: request.auth,
        verify_tls: request.verify_tls.unwrap_or(true),
        pool_max_idle_per_host: 10,
    };

    let adapter = HttpAdapter::new(config).map_err(|e| {
        error!("Failed to create HTTP adapter: {}", e);
        ApiError::from(e)
    })?;

    // Perform health check
    match adapter.health_check().await {
        Ok(_) => Ok(Json(AdapterTestResponse {
            success: true,
            message: format!("Successfully connected to {}", request.base_url),
            details: None,
        })),
        Err(e) => Ok(Json(AdapterTestResponse {
            success: false,
            message: format!("Connection failed: {}", e),
            details: Some(format!("{:?}", e)),
        })),
    }
}

/// Test Modbus adapter connection
pub async fn test_modbus_adapter(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<ModbusTestRequest>,
) -> ApiResult<Json<AdapterTestResponse>> {
    info!("Testing Modbus adapter connection to: {}", request.server_address);

    let config = ModbusConfig {
        server_address: request.server_address.clone(),
        unit_id: request.unit_id.unwrap_or(1),
        connection_timeout: request.connection_timeout.unwrap_or(10),
        read_timeout: 5,
        write_timeout: 5,
        max_retries: 1,
        retry_delay_ms: 1000,
    };

    let adapter = ModbusAdapter::new(config).map_err(|e| {
        error!("Failed to create Modbus adapter: {}", e);
        ApiError::from(e)
    })?;

    // Perform health check (attempts to read a register)
    match adapter.health_check().await {
        Ok(_) => Ok(Json(AdapterTestResponse {
            success: true,
            message: format!("Successfully connected to Modbus server {}", request.server_address),
            details: None,
        })),
        Err(e) => Ok(Json(AdapterTestResponse {
            success: false,
            message: format!("Connection failed: {}", e),
            details: Some(format!("{:?}", e)),
        })),
    }
}

/// Read Modbus holding registers
pub async fn read_modbus_registers(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<ModbusReadRequest>,
) -> ApiResult<Json<ModbusReadResponse>> {
    info!(
        "Reading Modbus holding registers from {} at address {} (count: {})",
        request.server_address, request.address, request.count
    );

    let config = ModbusConfig {
        server_address: request.server_address.clone(),
        unit_id: request.unit_id.unwrap_or(1),
        connection_timeout: 10,
        read_timeout: 5,
        write_timeout: 5,
        max_retries: 3,
        retry_delay_ms: 1000,
    };

    let adapter = ModbusAdapter::new(config).map_err(ApiError::from)?;

    let values = adapter
        .read_holding_registers(request.address, request.count)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(ModbusReadResponse { values }))
}

/// Test OPC UA adapter connection
pub async fn test_opcua_adapter(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<OpcUaTestRequest>,
) -> ApiResult<Json<AdapterTestResponse>> {
    info!("Testing OPC UA adapter connection to: {}", request.endpoint_url);

    let config = OpcUaConfig {
        endpoint_url: request.endpoint_url.clone(),
        application_name: "UAIP Hub".to_string(),
        application_uri: "urn:uaip:hub".to_string(),
        security_mode: request.security_mode.unwrap_or(uaip_adapters::opcua::SecurityMode::None),
        security_policy: request.security_policy.unwrap_or(uaip_adapters::opcua::SecurityPolicy::None),
        username: request.username,
        password: request.password,
        connection_timeout: 10,
        session_timeout: 60,
        request_timeout: 5,
        max_retries: 1,
        retry_delay_ms: 1000,
    };

    let mut adapter = OpcUaAdapter::new(config).map_err(|e| {
        error!("Failed to create OPC UA adapter: {}", e);
        ApiError::from(e)
    })?;

    // Perform health check
    match adapter.health_check().await {
        Ok(_) => Ok(Json(AdapterTestResponse {
            success: true,
            message: format!("Successfully connected to OPC UA server {}", request.endpoint_url),
            details: adapter.get_session_id().map(|s| s.to_string()),
        })),
        Err(e) => Ok(Json(AdapterTestResponse {
            success: false,
            message: format!("Connection failed: {}", e),
            details: Some(format!("{:?}", e)),
        })),
    }
}

/// Read OPC UA node value
pub async fn read_opcua_node(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<OpcUaReadRequest>,
) -> ApiResult<Json<OpcUaReadResponse>> {
    info!(
        "Reading OPC UA node from {} : {}",
        request.endpoint_url, request.node_id
    );

    let config = OpcUaConfig {
        endpoint_url: request.endpoint_url.clone(),
        application_name: "UAIP Hub".to_string(),
        application_uri: "urn:uaip:hub".to_string(),
        security_mode: uaip_adapters::opcua::SecurityMode::None,
        security_policy: uaip_adapters::opcua::SecurityPolicy::None,
        username: request.username,
        password: request.password,
        connection_timeout: 10,
        session_timeout: 60,
        request_timeout: 5,
        max_retries: 3,
        retry_delay_ms: 1000,
    };

    let mut adapter = OpcUaAdapter::new(config).map_err(ApiError::from)?;

    let node_id = NodeId::from_string(&request.node_id).map_err(|e| {
        ApiError::bad_request(format!("Invalid node ID format: {}", e))
    })?;

    let data_value = adapter.read_node(&node_id).await.map_err(ApiError::from)?;

    Ok(Json(OpcUaReadResponse {
        node_id: request.node_id,
        value: data_value.value,
        source_timestamp: data_value.source_timestamp,
        server_timestamp: data_value.server_timestamp,
        status_code: data_value.status_code,
    }))
}

/// Create WebRTC offer
pub async fn create_webrtc_offer(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<WebRtcOfferRequest>,
) -> ApiResult<Json<WebRtcOfferResponse>> {
    info!("Creating WebRTC offer");

    let config = WebRtcConfig {
        ice_servers: request.ice_servers.unwrap_or_else(|| {
            uaip_adapters::webrtc::IceServer::google_stun()
        }),
        enable_audio: request.enable_audio.unwrap_or(false),
        enable_video: request.enable_video.unwrap_or(false),
        enable_data_channels: request.enable_data_channels.unwrap_or(true),
        data_channels: request.data_channels.unwrap_or_else(|| {
            vec![DataChannelConfig::default()]
        }),
        connection_timeout: 30,
    };

    let adapter = WebRtcAdapter::new(config).map_err(|e| {
        error!("Failed to create WebRTC adapter: {}", e);
        ApiError::from(e)
    })?;

    let offer = adapter.create_offer().await.map_err(ApiError::from)?;

    Ok(Json(WebRtcOfferResponse {
        sdp_type: format!("{:?}", offer.sdp_type),
        sdp: offer.sdp,
    }))
}

// ===== Request/Response Types =====

#[derive(Debug, Serialize)]
pub struct AdapterListResponse {
    pub adapters: Vec<AdapterInfo>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct AdapterInfo {
    pub adapter_type: String,
    pub name: String,
    pub description: String,
    pub supported_operations: Vec<String>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct AdapterTestResponse {
    pub success: bool,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HttpTestRequest {
    pub base_url: String,
    pub timeout_seconds: Option<u64>,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub auth: Option<uaip_adapters::http::HttpAuth>,
    pub verify_tls: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ModbusTestRequest {
    pub server_address: String,
    pub unit_id: Option<u8>,
    pub connection_timeout: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ModbusReadRequest {
    pub server_address: String,
    pub unit_id: Option<u8>,
    pub address: u16,
    pub count: u16,
}

#[derive(Debug, Serialize)]
pub struct ModbusReadResponse {
    pub values: Vec<u16>,
}

#[derive(Debug, Deserialize)]
pub struct OpcUaTestRequest {
    pub endpoint_url: String,
    pub security_mode: Option<uaip_adapters::opcua::SecurityMode>,
    pub security_policy: Option<uaip_adapters::opcua::SecurityPolicy>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OpcUaReadRequest {
    pub endpoint_url: String,
    pub node_id: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OpcUaReadResponse {
    pub node_id: String,
    pub value: OpcValue,
    pub source_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub server_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub status_code: u32,
}

#[derive(Debug, Deserialize)]
pub struct WebRtcOfferRequest {
    pub ice_servers: Option<Vec<uaip_adapters::webrtc::IceServer>>,
    pub enable_audio: Option<bool>,
    pub enable_video: Option<bool>,
    pub enable_data_channels: Option<bool>,
    pub data_channels: Option<Vec<DataChannelConfig>>,
}

#[derive(Debug, Serialize)]
pub struct WebRtcOfferResponse {
    pub sdp_type: String,
    pub sdp: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_info_serialization() {
        let info = AdapterInfo {
            adapter_type: "modbus".to_string(),
            name: "Modbus TCP".to_string(),
            description: "Industrial protocol".to_string(),
            supported_operations: vec!["read".to_string()],
            status: "available".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("modbus"));
        assert!(json.contains("available"));
    }

    #[test]
    fn test_modbus_read_request() {
        let request = ModbusReadRequest {
            server_address: "127.0.0.1:502".to_string(),
            unit_id: Some(1),
            address: 100,
            count: 10,
        };

        assert_eq!(request.server_address, "127.0.0.1:502");
        assert_eq!(request.unit_id, Some(1));
        assert_eq!(request.address, 100);
        assert_eq!(request.count, 10);
    }
}
