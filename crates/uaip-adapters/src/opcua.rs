//! OPC UA Protocol Adapter
//!
//! Provides OPC UA client functionality for industrial automation systems.
//! Supports reading and writing nodes, browsing the address space, and subscribing to data changes.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;
use tracing::{debug, info};

use uaip_core::error::{Result, UaipError};

/// OPC UA security mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SecurityMode {
    None,
    Sign,
    SignAndEncrypt,
}

/// OPC UA security policy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityPolicy {
    None,
    Basic128Rsa15,
    Basic256,
    Basic256Sha256,
    Aes128Sha256RsaOaep,
    Aes256Sha256RsaPss,
}

/// OPC UA node identifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeId {
    /// Namespace index
    pub namespace: u16,
    /// Node identifier (can be numeric, string, GUID, or opaque)
    pub identifier: String,
}

impl NodeId {
    pub fn new(namespace: u16, identifier: impl Into<String>) -> Self {
        Self {
            namespace,
            identifier: identifier.into(),
        }
    }

    /// Create a node ID from string format "ns=X;s=Y" or "ns=X;i=Y"
    pub fn from_string(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(';').collect();
        if parts.len() != 2 {
            return Err(UaipError::InvalidParameter(
                "Invalid NodeId format".to_string(),
            ));
        }

        let namespace = parts[0]
            .strip_prefix("ns=")
            .ok_or_else(|| UaipError::InvalidParameter("Missing ns= prefix".to_string()))?
            .parse::<u16>()
            .map_err(|_| UaipError::InvalidParameter("Invalid namespace".to_string()))?;

        let identifier = if let Some(id) = parts[1].strip_prefix("i=") {
            id.to_string()
        } else if let Some(id) = parts[1].strip_prefix("s=") {
            id.to_string()
        } else {
            return Err(UaipError::InvalidParameter(
                "Missing identifier prefix (i= or s=)".to_string(),
            ));
        };

        Ok(Self {
            namespace,
            identifier,
        })
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Try to parse as number for numeric format
        if let Ok(_num) = self.identifier.parse::<u32>() {
            write!(f, "ns={};i={}", self.namespace, self.identifier)
        } else {
            write!(f, "ns={};s={}", self.namespace, self.identifier)
        }
    }
}

/// OPC UA data value with timestamp and status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataValue {
    /// The value
    pub value: OpcValue,
    /// Source timestamp
    pub source_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    /// Server timestamp
    pub server_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    /// Status code
    pub status_code: u32,
}

/// OPC UA value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum OpcValue {
    Boolean(bool),
    SByte(i8),
    Byte(u8),
    Int16(i16),
    UInt16(u16),
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Float(f32),
    Double(f64),
    String(String),
    ByteString(Vec<u8>),
    Null,
}

/// OPC UA adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpcUaConfig {
    /// OPC UA server endpoint URL
    pub endpoint_url: String,

    /// Application name
    pub application_name: String,

    /// Application URI
    pub application_uri: String,

    /// Security mode
    pub security_mode: SecurityMode,

    /// Security policy
    pub security_policy: SecurityPolicy,

    /// Username for authentication
    pub username: Option<String>,

    /// Password for authentication
    pub password: Option<String>,

    /// Connection timeout in seconds
    pub connection_timeout: u64,

    /// Session timeout in seconds
    pub session_timeout: u64,

    /// Request timeout in seconds
    pub request_timeout: u64,

    /// Maximum retries for failed operations
    pub max_retries: u32,

    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for OpcUaConfig {
    fn default() -> Self {
        Self {
            endpoint_url: "opc.tcp://localhost:4840".to_string(),
            application_name: "UAIP OPC UA Client".to_string(),
            application_uri: "urn:uaip:opcua:client".to_string(),
            security_mode: SecurityMode::None,
            security_policy: SecurityPolicy::None,
            username: None,
            password: None,
            connection_timeout: 10,
            session_timeout: 60,
            request_timeout: 5,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// OPC UA adapter for industrial automation communication
pub struct OpcUaAdapter {
    config: OpcUaConfig,
    session_id: Option<String>,
    connected: bool,
}

impl OpcUaAdapter {
    /// Create a new OPC UA adapter
    pub fn new(config: OpcUaConfig) -> Result<Self> {
        info!(
            "OPC UA adapter created for endpoint: {}",
            config.endpoint_url
        );

        Ok(Self {
            config,
            session_id: None,
            connected: false,
        })
    }

    /// Connect to OPC UA server and create session
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to OPC UA server: {}", self.config.endpoint_url);

        // Simulate connection (in real implementation, use opcua crate)
        tokio::time::sleep(Duration::from_millis(100)).await;

        self.session_id = Some(format!("session-{}", uuid::Uuid::new_v4()));
        self.connected = true;

        info!(
            "Connected to OPC UA server (session: {})",
            self.session_id.as_ref().unwrap()
        );
        Ok(())
    }

    /// Disconnect from OPC UA server
    pub async fn disconnect(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }

        info!("Disconnecting from OPC UA server");
        self.connected = false;
        self.session_id = None;

        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Ensure connection is active
    async fn ensure_connected(&mut self) -> Result<()> {
        if !self.connected {
            self.connect().await?;
        }
        Ok(())
    }

    /// Read a single node value
    pub async fn read_node(&mut self, node_id: &NodeId) -> Result<DataValue> {
        self.ensure_connected().await?;

        debug!("Reading node: {}", node_id.to_string());

        // Simulate read operation
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Return mock data
        Ok(DataValue {
            value: OpcValue::Double(42.5),
            source_timestamp: Some(chrono::Utc::now()),
            server_timestamp: Some(chrono::Utc::now()),
            status_code: 0, // Good
        })
    }

    /// Read multiple node values
    pub async fn read_nodes(&mut self, node_ids: &[NodeId]) -> Result<Vec<DataValue>> {
        self.ensure_connected().await?;

        debug!("Reading {} nodes", node_ids.len());

        let mut results = Vec::new();
        for node_id in node_ids {
            results.push(self.read_node(node_id).await?);
        }

        Ok(results)
    }

    /// Write a value to a node
    pub async fn write_node(&mut self, node_id: &NodeId, value: OpcValue) -> Result<()> {
        self.ensure_connected().await?;

        debug!("Writing to node: {} = {:?}", node_id.to_string(), value);

        // Simulate write operation
        tokio::time::sleep(Duration::from_millis(50)).await;

        Ok(())
    }

    /// Write multiple values to nodes
    pub async fn write_nodes(&mut self, writes: &[(NodeId, OpcValue)]) -> Result<()> {
        self.ensure_connected().await?;

        debug!("Writing to {} nodes", writes.len());

        for (node_id, value) in writes {
            self.write_node(node_id, value.clone()).await?;
        }

        Ok(())
    }

    /// Browse children of a node
    pub async fn browse_node(&mut self, node_id: &NodeId) -> Result<Vec<NodeId>> {
        self.ensure_connected().await?;

        debug!("Browsing node: {}", node_id.to_string());

        // Simulate browse operation
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Return mock children
        Ok(vec![
            NodeId::new(node_id.namespace, format!("{}.child1", node_id.identifier)),
            NodeId::new(node_id.namespace, format!("{}.child2", node_id.identifier)),
        ])
    }

    /// Call a method on a node
    pub async fn call_method(
        &mut self,
        object_id: &NodeId,
        method_id: &NodeId,
        input_arguments: Vec<OpcValue>,
    ) -> Result<Vec<OpcValue>> {
        self.ensure_connected().await?;

        debug!(
            "Calling method {} on object {} with {} arguments",
            method_id.to_string(),
            object_id.to_string(),
            input_arguments.len()
        );

        // Simulate method call
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Return mock output
        Ok(vec![OpcValue::Int32(0)])
    }

    /// Get the OPC UA configuration
    pub fn get_config(&self) -> &OpcUaConfig {
        &self.config
    }

    /// Get session ID
    pub fn get_session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Health check - try to read server status
    pub async fn health_check(&mut self) -> Result<()> {
        self.ensure_connected().await?;

        // Read ServerStatus node (standard node)
        let server_status_node = NodeId::new(0, "i=2256");
        self.read_node(&server_status_node).await?;

        Ok(())
    }
}

/// Helper function to create common OPC UA node IDs
pub mod well_known_nodes {
    use super::NodeId;

    /// Root folder node
    pub fn root_folder() -> NodeId {
        NodeId::new(0, "84")
    }

    /// Objects folder node
    pub fn objects_folder() -> NodeId {
        NodeId::new(0, "85")
    }

    /// Server node
    pub fn server() -> NodeId {
        NodeId::new(0, "2253")
    }

    /// Server status node
    pub fn server_status() -> NodeId {
        NodeId::new(0, "2256")
    }

    /// Server current time node
    pub fn server_current_time() -> NodeId {
        NodeId::new(0, "2258")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcua_config_default() {
        let config = OpcUaConfig::default();
        assert_eq!(config.endpoint_url, "opc.tcp://localhost:4840");
        assert_eq!(config.security_mode, SecurityMode::None);
        assert_eq!(config.connection_timeout, 10);
    }

    #[test]
    fn test_node_id_creation() {
        let node_id = NodeId::new(2, "MyVariable");
        assert_eq!(node_id.namespace, 2);
        assert_eq!(node_id.identifier, "MyVariable");
    }

    #[test]
    fn test_node_id_from_string() {
        let node_id = NodeId::from_string("ns=2;s=MyVariable").unwrap();
        assert_eq!(node_id.namespace, 2);
        assert_eq!(node_id.identifier, "MyVariable");

        let numeric = NodeId::from_string("ns=0;i=2253").unwrap();
        assert_eq!(numeric.namespace, 0);
        assert_eq!(numeric.identifier, "2253");
    }

    #[test]
    fn test_node_id_to_string() {
        let node_id = NodeId::new(2, "123");
        assert_eq!(node_id.to_string(), "ns=2;i=123");

        let string_id = NodeId::new(2, "MyVariable");
        assert_eq!(string_id.to_string(), "ns=2;s=MyVariable");
    }

    #[test]
    fn test_opc_value_serialization() {
        let value = OpcValue::Double(42.5);
        let json = serde_json::to_string(&value).unwrap();
        assert!(json.contains("Double"));
        assert!(json.contains("42.5"));
    }

    #[test]
    fn test_security_mode() {
        let config = OpcUaConfig {
            security_mode: SecurityMode::SignAndEncrypt,
            ..Default::default()
        };
        assert_eq!(config.security_mode, SecurityMode::SignAndEncrypt);
    }

    #[tokio::test]
    async fn test_opcua_adapter_creation() {
        let config = OpcUaConfig::default();
        let result = OpcUaAdapter::new(config);
        assert!(result.is_ok());
        let adapter = result.unwrap();
        assert!(!adapter.is_connected());
    }

    #[tokio::test]
    async fn test_opcua_connect_disconnect() {
        let config = OpcUaConfig::default();
        let mut adapter = OpcUaAdapter::new(config).unwrap();

        assert!(!adapter.is_connected());

        adapter.connect().await.unwrap();
        assert!(adapter.is_connected());
        assert!(adapter.get_session_id().is_some());

        adapter.disconnect().await.unwrap();
        assert!(!adapter.is_connected());
    }

    #[tokio::test]
    async fn test_opcua_read_node() {
        let config = OpcUaConfig::default();
        let mut adapter = OpcUaAdapter::new(config).unwrap();

        let node_id = NodeId::new(2, "Temperature");
        let result = adapter.read_node(&node_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_well_known_nodes() {
        use well_known_nodes::*;

        let root = root_folder();
        assert_eq!(root.to_string(), "ns=0;i=84");

        let objects = objects_folder();
        assert_eq!(objects.to_string(), "ns=0;i=85");

        let server = server();
        assert_eq!(server.to_string(), "ns=0;i=2253");
    }
}
