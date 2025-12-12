//! Device Information and Capabilities
//!
//! This module defines device metadata, capabilities, and status information.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Device information structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceInfo {
    /// Unique device identifier
    pub device_id: String,
    /// MAC address
    pub mac_address: String,
    /// Manufacturer name
    pub manufacturer: String,
    /// Model name/number
    pub model: String,
    /// Firmware version
    pub firmware_version: String,
    /// Device status
    pub status: DeviceStatus,
    /// Device capabilities
    pub capabilities: Vec<Capability>,
    /// Registration timestamp
    pub registered_at: DateTime<Utc>,
    /// Last seen timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<DateTime<Utc>>,
    /// Device configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<HashMap<String, serde_json::Value>>,
    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Device status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceStatus {
    Online,
    Offline,
    Error,
    Maintenance,
    Deactivated,
}

/// Device capability definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Capability {
    /// Capability name
    pub name: String,
    /// Capability type
    pub capability_type: CapabilityType,
    /// Whether this is a primary capability
    pub is_primary: bool,
    /// Supported actions for this capability
    pub supported_actions: Vec<String>,
    /// Parameters for this capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, ParameterSpec>>,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Capability types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityType {
    /// Sensor capabilities (read-only)
    Sensor,
    /// Actuator capabilities (write/control)
    Actuator,
    /// Video streaming
    VideoStream,
    /// Audio streaming
    AudioStream,
    /// Two-way communication
    TwoWayAudio,
    /// Pan/Tilt control
    PanTilt,
    /// Configuration
    Configuration,
    /// Custom capability
    Custom,
}

/// Parameter specification for capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParameterSpec {
    /// Parameter data type
    pub param_type: ParameterType,
    /// Whether this parameter is required
    pub required: bool,
    /// Default value (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// Minimum value (for numeric types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Maximum value (for numeric types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Allowed values (for enum types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<Vec<String>>,
    /// Unit of measurement (for sensor data)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Parameter types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Object,
    Array,
}

/// Device registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRegistrationRequest {
    pub device_id: String,
    pub mac_address: String,
    pub manufacturer: String,
    pub model: String,
    pub firmware_version: String,
    pub capabilities: Vec<Capability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<String>,
}

/// Device registration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRegistrationResponse {
    pub success: bool,
    pub device_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hub_certificate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Heartbeat message from device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub status: DeviceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<HealthCheck>,
}

/// Health check data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub connectivity: HealthStatus,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub storage_usage: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_metrics: Option<HashMap<String, serde_json::Value>>,
}

/// Health status for individual checks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Ok,
    Warning,
    Error,
}

impl DeviceInfo {
    /// Create a new device info
    pub fn new(
        device_id: String,
        mac_address: String,
        manufacturer: String,
        model: String,
        firmware_version: String,
    ) -> Self {
        Self {
            device_id,
            mac_address,
            manufacturer,
            model,
            firmware_version,
            status: DeviceStatus::Offline,
            capabilities: Vec::new(),
            registered_at: Utc::now(),
            last_seen: None,
            configuration: None,
            metadata: None,
        }
    }

    /// Add a capability to the device
    pub fn add_capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Update device status
    pub fn with_status(mut self, status: DeviceStatus) -> Self {
        self.status = status;
        self
    }

    /// Update last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = Some(Utc::now());
    }

    /// Check if device supports a specific capability
    pub fn has_capability(&self, capability_name: &str) -> bool {
        self.capabilities.iter().any(|c| c.name == capability_name)
    }

    /// Get a specific capability by name
    pub fn get_capability(&self, capability_name: &str) -> Option<&Capability> {
        self.capabilities.iter().find(|c| c.name == capability_name)
    }
}

impl Capability {
    /// Create a new capability
    pub fn new(name: String, capability_type: CapabilityType, is_primary: bool) -> Self {
        Self {
            name,
            capability_type,
            is_primary,
            supported_actions: Vec::new(),
            parameters: None,
            description: None,
        }
    }

    /// Add a supported action
    pub fn add_action(mut self, action: String) -> Self {
        self.supported_actions.push(action);
        self
    }

    /// Add a parameter
    pub fn add_parameter(mut self, param_name: String, param_spec: ParameterSpec) -> Self {
        if self.parameters.is_none() {
            self.parameters = Some(HashMap::new());
        }
        if let Some(ref mut params) = self.parameters {
            params.insert(param_name, param_spec);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_info_creation() {
        let device = DeviceInfo::new(
            "device_001".to_string(),
            "00:11:22:33:44:55".to_string(),
            "Acme Corp".to_string(),
            "SmartCam 3000".to_string(),
            "1.2.3".to_string(),
        );

        assert_eq!(device.device_id, "device_001");
        assert_eq!(device.status, DeviceStatus::Offline);
        assert_eq!(device.capabilities.len(), 0);
    }

    #[test]
    fn test_capability_creation() {
        let capability = Capability::new(
            "temperature".to_string(),
            CapabilityType::Sensor,
            true,
        )
        .add_action("read".to_string())
        .add_parameter(
            "unit".to_string(),
            ParameterSpec {
                param_type: ParameterType::String,
                required: false,
                default: Some(serde_json::json!("celsius")),
                min: None,
                max: None,
                allowed_values: Some(vec!["celsius".to_string(), "fahrenheit".to_string()]),
                unit: None,
                description: Some("Temperature unit".to_string()),
            },
        );

        assert_eq!(capability.name, "temperature");
        assert_eq!(capability.supported_actions.len(), 1);
        assert!(capability.parameters.is_some());
    }

    #[test]
    fn test_device_has_capability() {
        let device = DeviceInfo::new(
            "device_001".to_string(),
            "00:11:22:33:44:55".to_string(),
            "Acme Corp".to_string(),
            "SmartCam 3000".to_string(),
            "1.2.3".to_string(),
        )
        .add_capability(
            Capability::new(
                "video_stream".to_string(),
                CapabilityType::VideoStream,
                true,
            )
        );

        assert!(device.has_capability("video_stream"));
        assert!(!device.has_capability("audio_stream"));
    }
}
