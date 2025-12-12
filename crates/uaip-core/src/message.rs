//! UAIP Message Format
//!
//! This module defines the core message structure for the Universal AI Integration Protocol.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Root message structure for UAIP
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UaipMessage {
    pub header: Header,
    pub security: Security,
    pub payload: Payload,
    pub metadata: Metadata,
}

/// Message header containing routing and identification information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Header {
    /// Protocol version
    pub version: String,
    /// Unique message identifier
    pub message_id: String,
    /// Correlation ID for request-response patterns (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
    /// Time-to-live in milliseconds
    pub ttl: u64,
    /// Message priority
    pub priority: Priority,
    /// Sender information
    pub sender: Entity,
    /// Recipient information
    pub recipient: Entity,
    /// Routing information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<Routing>,
}

/// Entity (sender or recipient) information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entity {
    pub id: String,
    #[serde(rename = "type")]
    pub entity_type: EntityType,
}

/// Entity type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Device,
    AiAgent,
    User,
    System,
    Broadcast,
}

/// Message priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

/// Routing information for multi-hop scenarios
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Routing {
    pub hop_count: u32,
    pub max_hops: u32,
    pub path: Vec<String>,
}

/// Security information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Security {
    pub authentication: Authentication,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption: Option<Encryption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<Signature>,
}

/// Authentication information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Authentication {
    pub method: AuthMethod,
    pub token: String,
}

/// Authentication methods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    Jwt,
    Certificate,
    ApiKey,
}

/// Encryption information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Encryption {
    pub enabled: bool,
    pub algorithm: String,
    pub key_id: String,
}

/// Digital signature information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Signature {
    pub algorithm: String,
    pub value: String,
}

/// Message payload
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Payload {
    /// Action to perform
    pub action: Action,
    /// Device type (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_type: Option<DeviceType>,
    /// Capability being used (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability: Option<String>,
    /// Payload data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Data>,
    /// Additional parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

/// Action types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Read,
    Write,
    Stream,
    Execute,
    Subscribe,
    Notify,
}

/// Device types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Sensor,
    Actuator,
    Camera,
    Audio,
    Hybrid,
}

/// Data payload structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Data {
    pub format: DataFormat,
    pub encoding: DataEncoding,
    pub compression: CompressionType,
    pub content: serde_json::Value,
}

/// Data format types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DataFormat {
    Json,
    Binary,
    Stream,
}

/// Data encoding types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DataEncoding {
    Utf8,
    Base64,
}

/// Compression types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CompressionType {
    None,
    Gzip,
    Zstd,
}

/// Message metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
    /// Whether acknowledgment is required
    pub requires_ack: bool,
    /// Acknowledgment timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ack_timeout: Option<u64>,
    /// Retry policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<RetryPolicy>,
    /// Quality of Service level
    pub qos: QosLevel,
    /// Content type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// Custom user data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_data: Option<HashMap<String, serde_json::Value>>,
}

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryPolicy {
    pub enabled: bool,
    pub max_retries: u32,
    pub backoff: BackoffStrategy,
}

/// Backoff strategies for retries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BackoffStrategy {
    Linear,
    Exponential,
}

/// Quality of Service levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QosLevel {
    AtMostOnce,  // QoS 0: Fire and forget
    AtLeastOnce, // QoS 1: At least once delivery
    ExactlyOnce, // QoS 2: Exactly once delivery
}

impl UaipMessage {
    /// Create a new UAIP message with default values
    pub fn new(
        sender_id: String,
        sender_type: EntityType,
        recipient_id: String,
        recipient_type: EntityType,
    ) -> Self {
        Self {
            header: Header {
                version: "1.0".to_string(),
                message_id: format!("msg_{}", Uuid::new_v4().simple()),
                correlation_id: None,
                timestamp: Utc::now(),
                ttl: 5000, // 5 seconds default
                priority: Priority::Normal,
                sender: Entity {
                    id: sender_id,
                    entity_type: sender_type,
                },
                recipient: Entity {
                    id: recipient_id,
                    entity_type: recipient_type,
                },
                routing: None,
            },
            security: Security {
                authentication: Authentication {
                    method: AuthMethod::Jwt,
                    token: String::new(),
                },
                encryption: None,
                signature: None,
            },
            payload: Payload {
                action: Action::Read,
                device_type: None,
                capability: None,
                data: None,
                parameters: None,
            },
            metadata: Metadata {
                requires_ack: false,
                ack_timeout: None,
                retry_policy: None,
                qos: QosLevel::AtMostOnce,
                content_type: Some("application/json".to_string()),
                user_data: None,
            },
        }
    }

    /// Set correlation ID for request-response patterns
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.header.correlation_id = Some(correlation_id);
        self
    }

    /// Set message priority
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.header.priority = priority;
        self
    }

    /// Set QoS level
    pub fn with_qos(mut self, qos: QosLevel) -> Self {
        self.metadata.qos = qos;
        self
    }

    /// Set payload action
    pub fn with_action(mut self, action: Action) -> Self {
        self.payload.action = action;
        self
    }

    /// Set authentication token
    pub fn with_token(mut self, token: String) -> Self {
        self.security.authentication.token = token;
        self
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = UaipMessage::new(
            "device_001".to_string(),
            EntityType::Device,
            "ai_agent_001".to_string(),
            EntityType::AiAgent,
        );

        assert_eq!(msg.header.version, "1.0");
        assert_eq!(msg.header.sender.id, "device_001");
        assert_eq!(msg.header.recipient.id, "ai_agent_001");
        assert_eq!(msg.header.priority, Priority::Normal);
    }

    #[test]
    fn test_message_serialization() {
        let msg = UaipMessage::new(
            "device_001".to_string(),
            EntityType::Device,
            "ai_agent_001".to_string(),
            EntityType::AiAgent,
        );

        let json = msg.to_json().expect("Should serialize");
        assert!(json.contains("\"version\": \"1.0\""));
        assert!(json.contains("\"sender\""));
        assert!(json.contains("\"recipient\""));
    }

    #[test]
    fn test_message_deserialization() {
        let msg = UaipMessage::new(
            "device_001".to_string(),
            EntityType::Device,
            "ai_agent_001".to_string(),
            EntityType::AiAgent,
        );

        let json = msg.to_json().expect("Should serialize");
        let deserialized = UaipMessage::from_json(&json).expect("Should deserialize");

        assert_eq!(deserialized.header.sender.id, msg.header.sender.id);
        assert_eq!(deserialized.header.recipient.id, msg.header.recipient.id);
    }

    #[test]
    fn test_message_builder() {
        let msg = UaipMessage::new(
            "device_001".to_string(),
            EntityType::Device,
            "ai_agent_001".to_string(),
            EntityType::AiAgent,
        )
        .with_priority(Priority::High)
        .with_qos(QosLevel::ExactlyOnce)
        .with_action(Action::Execute)
        .with_correlation_id("corr_123".to_string());

        assert_eq!(msg.header.priority, Priority::High);
        assert_eq!(msg.metadata.qos, QosLevel::ExactlyOnce);
        assert_eq!(msg.payload.action, Action::Execute);
        assert_eq!(msg.header.correlation_id, Some("corr_123".to_string()));
    }
}
