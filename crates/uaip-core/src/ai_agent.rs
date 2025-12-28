//! AI Agent Integration Module
//!
//! Provides types and functionality for AI agents to interact with devices and the UAIP protocol.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::device::{Capability, DeviceId};

/// AI Agent information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAgent {
    /// Unique agent identifier
    pub id: Uuid,

    /// Agent name/identifier
    pub name: String,

    /// Agent type/category
    pub agent_type: AgentType,

    /// Capabilities the agent can handle
    pub supported_capabilities: Vec<Capability>,

    /// Agent version
    pub version: String,

    /// Agent provider/organization
    pub provider: String,

    /// Custom metadata
    pub metadata: HashMap<String, String>,

    /// Agent configuration
    pub config: AgentConfig,
}

impl AiAgent {
    /// Create a new AI agent
    pub fn new(name: String, agent_type: AgentType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            agent_type,
            supported_capabilities: Vec::new(),
            version: "1.0.0".to_string(),
            provider: "UAIP".to_string(),
            metadata: HashMap::new(),
            config: AgentConfig::default(),
        }
    }

    /// Add supported capability
    pub fn add_capability(&mut self, capability: Capability) {
        if !self.supported_capabilities.contains(&capability) {
            self.supported_capabilities.push(capability);
        }
    }

    /// Check if agent supports a capability
    pub fn supports_capability(&self, capability: &Capability) -> bool {
        self.supported_capabilities.contains(capability)
    }

    /// Check if agent can handle a device
    pub fn can_handle_device(&self, device_capabilities: &[Capability]) -> bool {
        device_capabilities
            .iter()
            .any(|cap| self.supports_capability(cap))
    }
}

/// AI Agent type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    /// Conversational AI (ChatGPT, Claude, etc.)
    Conversational,

    /// Automation AI (workflow orchestration)
    Automation,

    /// Monitoring AI (anomaly detection, analytics)
    Monitoring,

    /// Control AI (device control and optimization)
    Control,

    /// Diagnostic AI (troubleshooting, maintenance)
    Diagnostic,

    /// Predictive AI (forecasting, prediction)
    Predictive,

    /// Multi-purpose AI (combination of above)
    MultiPurpose,

    /// Custom agent type
    Custom,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Maximum concurrent device connections
    pub max_concurrent_devices: usize,

    /// Command rate limit per minute
    pub command_rate_limit: u32,

    /// Enable caching
    pub enable_caching: bool,

    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,

    /// Priority level for agent commands
    pub default_priority: CommandPriority,

    /// Timeout for device operations in milliseconds
    pub operation_timeout_ms: u64,

    /// Enable learning/adaptation
    pub enable_learning: bool,

    /// Preferred communication protocol
    pub preferred_protocol: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_concurrent_devices: 100,
            command_rate_limit: 60,
            enable_caching: true,
            cache_ttl_secs: 300,
            default_priority: CommandPriority::Normal,
            operation_timeout_ms: 30000,
            enable_learning: false,
            preferred_protocol: None,
        }
    }
}

/// Command priority for AI-initiated commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// AI Session - represents an active interaction session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSession {
    /// Unique session ID
    pub id: Uuid,

    /// Agent ID
    pub agent_id: Uuid,

    /// Connected devices in this session
    pub devices: Vec<DeviceId>,

    /// Session state
    pub state: SessionState,

    /// Session start time
    pub started_at: chrono::DateTime<chrono::Utc>,

    /// Session last activity time
    pub last_activity_at: chrono::DateTime<chrono::Utc>,

    /// Session context/state data
    pub context: HashMap<String, serde_json::Value>,

    /// Session statistics
    pub stats: SessionStats,
}

impl AiSession {
    /// Create a new session
    pub fn new(agent_id: Uuid) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            agent_id,
            devices: Vec::new(),
            state: SessionState::Active,
            started_at: now,
            last_activity_at: now,
            context: HashMap::new(),
            stats: SessionStats::default(),
        }
    }

    /// Add device to session
    pub fn add_device(&mut self, device_id: DeviceId) {
        if !self.devices.contains(&device_id) {
            self.devices.push(device_id);
            self.last_activity_at = chrono::Utc::now();
        }
    }

    /// Remove device from session
    pub fn remove_device(&mut self, device_id: &DeviceId) {
        self.devices.retain(|d| d != device_id);
        self.last_activity_at = chrono::Utc::now();
    }

    /// Update session activity
    pub fn update_activity(&mut self) {
        self.last_activity_at = chrono::Utc::now();
    }

    /// Check if session is expired
    pub fn is_expired(&self, timeout_secs: u64) -> bool {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(self.last_activity_at);
        duration.num_seconds() as u64 > timeout_secs
    }

    /// Get session duration in seconds
    pub fn duration_secs(&self) -> i64 {
        let now = chrono::Utc::now();
        now.signed_duration_since(self.started_at).num_seconds()
    }
}

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionState {
    /// Session is active
    Active,

    /// Session is paused
    Paused,

    /// Session is ending
    Terminating,

    /// Session has ended
    Terminated,

    /// Session encountered an error
    Error,
}

/// Session statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionStats {
    /// Number of commands sent
    pub commands_sent: u64,

    /// Number of responses received
    pub responses_received: u64,

    /// Number of errors encountered
    pub errors_count: u64,

    /// Total bytes sent
    pub bytes_sent: u64,

    /// Total bytes received
    pub bytes_received: u64,

    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
}

impl SessionStats {
    /// Record a command
    pub fn record_command(&mut self, bytes_sent: usize) {
        self.commands_sent += 1;
        self.bytes_sent += bytes_sent as u64;
    }

    /// Record a response
    pub fn record_response(&mut self, bytes_received: usize, response_time_ms: u64) {
        self.responses_received += 1;
        self.bytes_received += bytes_received as u64;

        // Update running average
        let total_responses = self.responses_received as f64;
        self.avg_response_time_ms = (self.avg_response_time_ms * (total_responses - 1.0)
            + response_time_ms as f64)
            / total_responses;
    }

    /// Record an error
    pub fn record_error(&mut self) {
        self.errors_count += 1;
    }
}

/// AI-Device interaction request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInteraction {
    /// Session ID
    pub session_id: Uuid,

    /// Target device ID
    pub device_id: DeviceId,

    /// Interaction type
    pub interaction_type: InteractionType,

    /// Command/query parameters
    pub parameters: HashMap<String, serde_json::Value>,

    /// Expected response format
    pub response_format: Option<String>,

    /// Timeout for this interaction
    pub timeout_ms: Option<u64>,
}

/// Type of interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractionType {
    /// Query device state/data
    Query,

    /// Send command to device
    Command,

    /// Subscribe to device events
    Subscribe,

    /// Unsubscribe from device events
    Unsubscribe,

    /// Configure device
    Configure,

    /// Request device metadata
    Metadata,
}

/// Device interaction response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionResponse {
    /// Request ID
    pub request_id: Uuid,

    /// Device ID
    pub device_id: DeviceId,

    /// Success status
    pub success: bool,

    /// Response data
    pub data: Option<serde_json::Value>,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Response timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// AI learning/feedback data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFeedback {
    /// Agent ID
    pub agent_id: Uuid,

    /// Session ID
    pub session_id: Option<Uuid>,

    /// Device ID
    pub device_id: Option<DeviceId>,

    /// Feedback type
    pub feedback_type: FeedbackType,

    /// Feedback data
    pub data: HashMap<String, serde_json::Value>,

    /// Feedback timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Type of feedback
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackType {
    /// Successful interaction
    Success,

    /// Failed interaction
    Failure,

    /// Performance metric
    Performance,

    /// User satisfaction
    Satisfaction,

    /// Anomaly detected
    Anomaly,

    /// Pattern learned
    Pattern,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_agent_creation() {
        let agent = AiAgent::new("TestAgent".to_string(), AgentType::Conversational);
        assert_eq!(agent.name, "TestAgent");
        assert_eq!(agent.agent_type, AgentType::Conversational);
        assert!(agent.supported_capabilities.is_empty());
    }

    #[test]
    fn test_ai_agent_capabilities() {
        let mut agent = AiAgent::new("TestAgent".to_string(), AgentType::Control);
        agent.add_capability(Capability::Sensor);
        agent.add_capability(Capability::Actuator);

        assert_eq!(agent.supported_capabilities.len(), 2);
        assert!(agent.supports_capability(&Capability::Sensor));
        assert!(agent.supports_capability(&Capability::Actuator));
        assert!(!agent.supports_capability(&Capability::VideoStream));
    }

    #[test]
    fn test_ai_agent_can_handle_device() {
        let mut agent = AiAgent::new("TestAgent".to_string(), AgentType::Control);
        agent.add_capability(Capability::Sensor);

        let device_caps = vec![Capability::Sensor, Capability::Actuator];
        assert!(agent.can_handle_device(&device_caps));

        let other_caps = vec![Capability::VideoStream];
        assert!(!agent.can_handle_device(&other_caps));
    }

    #[test]
    fn test_ai_session_creation() {
        let agent_id = Uuid::new_v4();
        let session = AiSession::new(agent_id);

        assert_eq!(session.agent_id, agent_id);
        assert_eq!(session.state, SessionState::Active);
        assert!(session.devices.is_empty());
    }

    #[test]
    fn test_ai_session_device_management() {
        let agent_id = Uuid::new_v4();
        let mut session = AiSession::new(agent_id);

        let device_id = "device-123".to_string();
        session.add_device(device_id.clone());

        assert_eq!(session.devices.len(), 1);
        assert!(session.devices.contains(&device_id));

        session.remove_device(&device_id);
        assert!(session.devices.is_empty());
    }

    #[test]
    fn test_session_stats() {
        let mut stats = SessionStats::default();

        stats.record_command(100);
        assert_eq!(stats.commands_sent, 1);
        assert_eq!(stats.bytes_sent, 100);

        stats.record_response(200, 50);
        assert_eq!(stats.responses_received, 1);
        assert_eq!(stats.bytes_received, 200);
        assert_eq!(stats.avg_response_time_ms, 50.0);

        stats.record_response(300, 100);
        assert_eq!(stats.responses_received, 2);
        assert_eq!(stats.avg_response_time_ms, 75.0);

        stats.record_error();
        assert_eq!(stats.errors_count, 1);
    }

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.max_concurrent_devices, 100);
        assert_eq!(config.command_rate_limit, 60);
        assert!(config.enable_caching);
        assert_eq!(config.default_priority, CommandPriority::Normal);
    }

    #[test]
    fn test_interaction_type_serialization() {
        let interaction = InteractionType::Query;
        let json = serde_json::to_string(&interaction).unwrap();
        assert!(json.contains("query"));
    }

    #[test]
    fn test_feedback_type() {
        let feedback_type = FeedbackType::Success;
        let json = serde_json::to_string(&feedback_type).unwrap();
        assert!(json.contains("success"));
    }
}
