//! Device management handlers with database integration

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use uaip_core::error::UaipError;

use crate::api::rest::{
    ApiResult, AppState, CommandRequest, CommandResponse, DeviceInfo, DeviceListResponse,
    DeviceRegistrationRequest, DeviceRegistrationResponse,
};

/// Query parameters for device listing
#[derive(Debug, Deserialize)]
pub struct DeviceListQuery {
    /// Filter by status (online, offline, error, maintenance, deactivated)
    #[serde(default)]
    pub status: Option<String>,

    /// Filter by manufacturer
    #[serde(default)]
    pub manufacturer: Option<String>,

    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: i64,

    /// Items per page
    #[serde(default = "default_per_page")]
    pub per_page: i64,

    /// Sort by field (id, device_id, status, last_seen, registered_at)
    #[serde(default = "default_sort_by")]
    pub sort_by: String,

    /// Sort order (asc, desc)
    #[serde(default = "default_sort_order")]
    pub sort_order: String,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    50
}

fn default_sort_by() -> String {
    "registered_at".to_string()
}

fn default_sort_order() -> String {
    "desc".to_string()
}

/// Device record from database
#[derive(Debug, sqlx::FromRow)]
struct DeviceRow {
    #[allow(dead_code)]
    id: sqlx::types::Uuid,
    device_id: String,
    manufacturer: String,
    model: String,
    status: String,
    last_seen: Option<chrono::DateTime<chrono::Utc>>,
}

/// List all devices with filtering, pagination, and sorting
pub async fn list_devices(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DeviceListQuery>,
) -> ApiResult<Json<DeviceListResponse>> {
    // Get database pool
    let db_pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| UaipError::InternalError("Database not configured".to_string()))?;

    // Validate pagination parameters
    if query.page < 1 {
        return Err(UaipError::InvalidParameter("page must be >= 1".to_string()).into());
    }
    if query.per_page < 1 || query.per_page > 100 {
        return Err(
            UaipError::InvalidParameter("per_page must be between 1 and 100".to_string()).into(),
        );
    }

    // Validate sort_by field
    let valid_sort_fields = ["id", "device_id", "status", "last_seen", "registered_at"];
    if !valid_sort_fields.contains(&query.sort_by.as_str()) {
        return Err(UaipError::InvalidParameter(format!(
            "sort_by must be one of: {}",
            valid_sort_fields.join(", ")
        ))
        .into());
    }

    // Validate sort_order
    let sort_order = match query.sort_order.to_lowercase().as_str() {
        "asc" => "ASC",
        "desc" => "DESC",
        _ => {
            return Err(UaipError::InvalidParameter(
                "sort_order must be 'asc' or 'desc'".to_string(),
            )
            .into())
        }
    };

    // Build query with filters
    let mut conditions = Vec::new();
    let mut bind_values: Vec<String> = Vec::new();

    if let Some(status) = &query.status {
        conditions.push(format!("status = ${}", conditions.len() + 1));
        bind_values.push(status.clone());
    }

    if let Some(manufacturer) = &query.manufacturer {
        conditions.push(format!("manufacturer = ${}", conditions.len() + 1));
        bind_values.push(manufacturer.clone());
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // Calculate offset
    let offset = (query.page - 1) * query.per_page;

    // Build SQL query - Note: Using format! here for ORDER BY is safe since we've validated the values
    let sql_query = format!(
        "SELECT id, device_id, manufacturer, model, status, last_seen
         FROM devices
         {}
         ORDER BY {} {}
         LIMIT ${} OFFSET ${}",
        where_clause,
        query.sort_by,
        sort_order,
        bind_values.len() + 1,
        bind_values.len() + 2
    );

    // Count query
    let count_query = format!("SELECT COUNT(*) as count FROM devices {}", where_clause);

    // Execute count query
    let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query);
    for value in &bind_values {
        count_query_builder = count_query_builder.bind(value);
    }
    let total = count_query_builder.fetch_one(db_pool).await.map_err(|e| {
        tracing::error!("Failed to count devices: {}", e);
        UaipError::InternalError("Failed to query devices".to_string())
    })?;

    // Execute main query
    let mut query_builder = sqlx::query_as::<_, DeviceRow>(&sql_query);
    for value in &bind_values {
        query_builder = query_builder.bind(value);
    }
    query_builder = query_builder.bind(query.per_page).bind(offset);

    let devices = query_builder.fetch_all(db_pool).await.map_err(|e| {
        tracing::error!("Failed to fetch devices: {}", e);
        UaipError::InternalError("Failed to query devices".to_string())
    })?;

    // Transform to DeviceInfo
    let device_infos: Vec<DeviceInfo> = devices
        .into_iter()
        .map(|d| DeviceInfo {
            device_id: d.device_id,
            name: format!("{} {}", d.manufacturer, d.model),
            device_type: d.manufacturer, // Using manufacturer as type for now
            status: d.status,
            last_seen: d.last_seen.map(|dt| dt.to_rfc3339()),
        })
        .collect();

    tracing::debug!(
        "Listed {} devices (total: {}, page: {}, per_page: {})",
        device_infos.len(),
        total,
        query.page,
        query.per_page
    );

    Ok(Json(DeviceListResponse {
        devices: device_infos,
        total: total as usize,
    }))
}

/// Register a new device (initiates 3-step challenge)
pub async fn register_device(
    State(state): State<Arc<AppState>>,
    Json(request): Json<DeviceRegistrationRequest>,
) -> ApiResult<Json<DeviceRegistrationResponse>> {
    // Validate device_id
    if request.device_id.is_empty() {
        return Err(UaipError::InvalidParameter("device_id cannot be empty".to_string()).into());
    }

    // Validate name
    if request.name.is_empty() {
        return Err(UaipError::InvalidParameter("name cannot be empty".to_string()).into());
    }

    // Get database pool
    let db_pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| UaipError::InternalError("Database not configured".to_string()))?;

    // Check if device already exists
    let existing =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM devices WHERE device_id = $1")
            .bind(&request.device_id)
            .fetch_one(db_pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to check device existence: {}", e);
                UaipError::InternalError("Failed to check device".to_string())
            })?;

    if existing > 0 {
        return Err(UaipError::InvalidParameter(format!(
            "Device with ID '{}' already exists",
            request.device_id
        ))
        .into());
    }

    // Generate registration challenge
    let challenge = format!("challenge_{}", uuid::Uuid::new_v4());
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);

    // TODO: Step 1 of 3-step challenge:
    // - Store challenge in temporary table with expiry
    // - Return challenge to device
    // Step 2: Device signs challenge with private key
    // Step 3: Hub verifies signature and creates certificate

    // For now, just insert the device directly (simplified registration)
    let device_uuid = uuid::Uuid::new_v4();

    // Generate a placeholder MAC address
    let mac_address = format!(
        "00:00:00:{:02x}:{:02x}:{:02x}",
        device_uuid.as_bytes()[0],
        device_uuid.as_bytes()[1],
        device_uuid.as_bytes()[2]
    );

    sqlx::query(
        "INSERT INTO devices (id, device_id, mac_address, manufacturer, model, firmware_version, status, capabilities, metadata)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind(device_uuid)
    .bind(&request.device_id)
    .bind(&mac_address)
    .bind(request.manufacturer.as_ref().unwrap_or(&"Unknown".to_string()))
    .bind(request.model.as_ref().unwrap_or(&"Unknown".to_string()))
    .bind("1.0.0") // Default firmware version
    .bind("offline") // Initially offline until first heartbeat
    .bind(serde_json::to_value(&request.capabilities).unwrap_or(serde_json::json!([])))
    .bind(serde_json::json!({
        "name": request.name,
        "device_type": request.device_type
    }))
    .execute(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to register device: {}", e);
        UaipError::InternalError("Failed to register device".to_string())
    })?;

    tracing::info!(
        "Device registered: {} ({})",
        request.device_id,
        request.name
    );

    Ok(Json(DeviceRegistrationResponse {
        device_id: request.device_id,
        challenge,
        expires_at: expires_at.to_rfc3339(),
    }))
}

/// Send command to a device
pub async fn send_command(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
    Json(request): Json<CommandRequest>,
) -> ApiResult<Json<CommandResponse>> {
    // Validate device_id
    if device_id.is_empty() {
        return Err(UaipError::InvalidParameter("device_id cannot be empty".to_string()).into());
    }

    // Validate action
    if request.action.is_empty() {
        return Err(UaipError::InvalidParameter("action cannot be empty".to_string()).into());
    }

    // Get database pool
    let db_pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| UaipError::InternalError("Database not configured".to_string()))?;

    // Verify device exists and get its UUID
    let device_uuid: Option<sqlx::types::Uuid> =
        sqlx::query_scalar("SELECT id FROM devices WHERE device_id = $1")
            .bind(&device_id)
            .fetch_optional(db_pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to query device: {}", e);
                UaipError::InternalError("Failed to verify device".to_string())
            })?;

    let _device_uuid = device_uuid
        .ok_or_else(|| UaipError::DeviceNotFound(format!("Device '{}' not found", device_id)))?;

    // Determine priority
    let priority = request.priority.as_deref().unwrap_or("normal");
    let priority_level = match priority {
        "low" => "low",
        "normal" => "normal",
        "high" => "high",
        "critical" => "critical",
        _ => "normal",
    };

    // Create message in message_log table
    let message_id = format!("msg_{}", uuid::Uuid::new_v4());
    let correlation_id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO message_log (
            id, message_id, correlation_id, sender_id, recipient_id,
            action, qos_level, priority, status, payload
         )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(uuid::Uuid::new_v4())
    .bind(&message_id)
    .bind(&correlation_id)
    .bind("hub") // sender is the hub
    .bind(&device_id) // recipient is the device
    .bind(&request.action)
    .bind(1_i16) // QoS level 1 (at least once)
    .bind(priority_level)
    .bind("pending")
    .bind(request.parameters.unwrap_or(serde_json::json!({})))
    .execute(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create message: {}", e);
        UaipError::InternalError("Failed to queue command".to_string())
    })?;

    tracing::info!(
        "Command queued: {} for device {} (message_id: {})",
        request.action,
        device_id,
        message_id
    );

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
    async fn test_list_devices_no_database() {
        let state = Arc::new(AppState::new());
        let query = DeviceListQuery {
            status: None,
            manufacturer: None,
            page: 1,
            per_page: 50,
            sort_by: "registered_at".to_string(),
            sort_order: "desc".to_string(),
        };

        let result = list_devices(State(state), Query(query)).await;
        assert!(result.is_err());
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
    async fn test_send_command_empty_action() {
        let state = Arc::new(AppState::new());
        let request = CommandRequest {
            action: "".to_string(),
            parameters: None,
            priority: None,
        };

        let result =
            send_command(State(state), Path("device-001".to_string()), Json(request)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_device_list_query_defaults() {
        let query = DeviceListQuery {
            status: None,
            manufacturer: None,
            page: default_page(),
            per_page: default_per_page(),
            sort_by: default_sort_by(),
            sort_order: default_sort_order(),
        };

        assert_eq!(query.page, 1);
        assert_eq!(query.per_page, 50);
        assert_eq!(query.sort_by, "registered_at");
        assert_eq!(query.sort_order, "desc");
    }
}
