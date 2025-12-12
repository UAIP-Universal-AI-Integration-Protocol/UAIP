//! Database models for device registry

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Device status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "lowercase")]
pub enum DeviceStatus {
    Online,
    Offline,
    Error,
    Maintenance,
    Deactivated,
}

impl std::fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceStatus::Online => write!(f, "online"),
            DeviceStatus::Offline => write!(f, "offline"),
            DeviceStatus::Error => write!(f, "error"),
            DeviceStatus::Maintenance => write!(f, "maintenance"),
            DeviceStatus::Deactivated => write!(f, "deactivated"),
        }
    }
}

/// Device database model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Device {
    pub id: Uuid,
    pub device_id: String,
    pub mac_address: String,
    pub manufacturer: String,
    pub model: String,
    pub firmware_version: Option<String>,
    pub status: DeviceStatus,
    pub last_seen: Option<DateTime<Utc>>,
    pub registered_at: DateTime<Utc>,
    pub certificate_expiry: Option<DateTime<Utc>>,
    pub configuration: serde_json::Value,
    pub capabilities: serde_json::Value,
    pub metadata: serde_json::Value,
}

/// Data for creating a new device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDevice {
    pub device_id: String,
    pub mac_address: String,
    pub manufacturer: String,
    pub model: String,
    pub firmware_version: Option<String>,
    pub capabilities: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
}

/// Data for updating an existing device
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateDevice {
    pub firmware_version: Option<String>,
    pub status: Option<DeviceStatus>,
    pub configuration: Option<serde_json::Value>,
    pub capabilities: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub certificate_expiry: Option<DateTime<Utc>>,
}

/// Device query filters
#[derive(Debug, Clone, Default)]
pub struct DeviceFilter {
    pub status: Option<DeviceStatus>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_status_display() {
        assert_eq!(DeviceStatus::Online.to_string(), "online");
        assert_eq!(DeviceStatus::Offline.to_string(), "offline");
        assert_eq!(DeviceStatus::Error.to_string(), "error");
        assert_eq!(DeviceStatus::Maintenance.to_string(), "maintenance");
        assert_eq!(DeviceStatus::Deactivated.to_string(), "deactivated");
    }

    #[test]
    fn test_device_status_serialization() {
        let status = DeviceStatus::Online;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""online""#);

        let deserialized: DeviceStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, DeviceStatus::Online);
    }

    #[test]
    fn test_create_device_serialization() {
        let create = CreateDevice {
            device_id: "device-123".to_string(),
            mac_address: "00:11:22:33:44:55".to_string(),
            manufacturer: "TestCorp".to_string(),
            model: "Model-X".to_string(),
            firmware_version: Some("1.0.0".to_string()),
            capabilities: serde_json::json!([
                {
                    "name": "temperature",
                    "type": "sensor",
                    "actions": ["read"]
                }
            ]),
            metadata: Some(serde_json::json!({"location": "lab"})),
        };

        let json = serde_json::to_string(&create).unwrap();
        let deserialized: CreateDevice = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.device_id, "device-123");
        assert_eq!(deserialized.mac_address, "00:11:22:33:44:55");
    }
}
