//! Device management handlers

use axum::{extract::{Path, State}, Json};
use std::sync::Arc;

use uaip_core::error::UaipError;

use crate::api::rest::{
    ApiResult, AppState, CommandRequest, CommandResponse, DeviceListResponse,
    DeviceRegistrationRequest, DeviceRegistrationResponse,
};

/// List all devices
pub async fn list_devices(
    State(_state): State<Arc<AppState>>,
) -> ApiResult<Json<DeviceListResponse>> {
    // TODO: Query devices from database using uaip-registry
    // For now, return empty list

    Ok(Json(DeviceListResponse {
        devices: vec![],
        total: 0,
    }))
}

/// Register a new device
pub async fn register_device(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<DeviceRegistrationRequest>,
) -> ApiResult<Json<DeviceRegistrationResponse>> {
    // Validate device_id
    if request.device_id.is_empty() {
        return Err(UaipError::InvalidParameter(
            "device_id cannot be empty".to_string(),
        )
        .into());
    }

    // Validate name
    if request.name.is_empty() {
        return Err(UaipError::InvalidParameter(
            "name cannot be empty".to_string(),
        )
        .into());
    }

    // TODO: Use uaip-registry to initiate registration challenge
    // For now, return placeholder challenge

    let challenge = format!("challenge_{}", uuid::Uuid::new_v4());
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);

    Ok(Json(DeviceRegistrationResponse {
        device_id: request.device_id,
        challenge,
        expires_at: expires_at.to_rfc3339(),
    }))
}

/// Send command to a device
pub async fn send_command(
    State(_state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
    Json(request): Json<CommandRequest>,
) -> ApiResult<Json<CommandResponse>> {
    // Validate device_id
    if device_id.is_empty() {
        return Err(UaipError::InvalidParameter(
            "device_id cannot be empty".to_string(),
        )
        .into());
    }

    // Validate action
    if request.action.is_empty() {
        return Err(UaipError::InvalidParameter(
            "action cannot be empty".to_string(),
        )
        .into());
    }

    // TODO: Validate device exists using uaip-registry
    // TODO: Create UaipMessage and route using uaip-router
    // For now, return placeholder response

    let message_id = format!("msg_{}", uuid::Uuid::new_v4());

    Ok(Json(CommandResponse {
        message_id,
        status: "queued".to_string(),
        queued_at: chrono::Utc::now().to_rfc3339(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_devices() {
        let state = Arc::new(AppState::new());
        let result = list_devices(State(state)).await;

        assert!(result.is_ok());
        let response = result.unwrap().0;
        assert_eq!(response.total, 0);
        assert_eq!(response.devices.len(), 0);
    }

    #[tokio::test]
    async fn test_register_device_success() {
        let state = Arc::new(AppState::new());
        let request = DeviceRegistrationRequest {
            device_id: "device-001".to_string(),
            device_type: "sensor".to_string(),
            name: "Temperature Sensor".to_string(),
            manufacturer: Some("ACME".to_string()),
            model: Some("TEMP-100".to_string()),
            capabilities: vec!["temperature".to_string()],
        };

        let result = register_device(State(state), Json(request)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.device_id, "device-001");
        assert!(response.challenge.starts_with("challenge_"));
        assert!(!response.expires_at.is_empty());
    }

    #[tokio::test]
    async fn test_register_device_empty_id() {
        let state = Arc::new(AppState::new());
        let request = DeviceRegistrationRequest {
            device_id: "".to_string(),
            device_type: "sensor".to_string(),
            name: "Test".to_string(),
            manufacturer: None,
            model: None,
            capabilities: vec![],
        };

        let result = register_device(State(state), Json(request)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_command_success() {
        let state = Arc::new(AppState::new());
        let request = CommandRequest {
            action: "read_temperature".to_string(),
            parameters: Some(serde_json::json!({"unit": "celsius"})),
            priority: Some("normal".to_string()),
        };

        let result = send_command(State(state), Path("device-001".to_string()), Json(request))
            .await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert!(response.message_id.starts_with("msg_"));
        assert_eq!(response.status, "queued");
    }

    #[tokio::test]
    async fn test_send_command_empty_action() {
        let state = Arc::new(AppState::new());
        let request = CommandRequest {
            action: "".to_string(),
            parameters: None,
            priority: None,
        };

        let result = send_command(State(state), Path("device-001".to_string()), Json(request))
            .await;
        assert!(result.is_err());
    }
}
