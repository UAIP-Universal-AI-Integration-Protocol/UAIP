//! Network Configuration and Management
//!
//! Provides centralized network configuration, service discovery, and connection management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use crate::error::{Result, UaipError};

/// Network configuration for services and adapters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Service bindings
    pub services: HashMap<String, ServiceConfig>,

    /// Adapter endpoints
    pub adapters: HashMap<String, AdapterEndpoint>,

    /// Network discovery configuration
    pub discovery: DiscoveryConfig,

    /// Connection pooling settings
    pub connection_pool: ConnectionPoolConfig,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        let mut services = HashMap::new();
        services.insert(
            "hub".to_string(),
            ServiceConfig {
                host: "0.0.0.0".to_string(),
                port: 8443,
                protocol: Protocol::Https,
                enabled: true,
                health_check_path: Some("/api/v1/system/health".to_string()),
                timeout_ms: 30000,
            },
        );
        services.insert(
            "metrics".to_string(),
            ServiceConfig {
                host: "0.0.0.0".to_string(),
                port: 9091,
                protocol: Protocol::Http,
                enabled: true,
                health_check_path: Some("/metrics".to_string()),
                timeout_ms: 5000,
            },
        );

        Self {
            services,
            adapters: HashMap::new(),
            discovery: DiscoveryConfig::default(),
            connection_pool: ConnectionPoolConfig::default(),
        }
    }
}

impl NetworkConfig {
    /// Get service address
    pub fn get_service_addr(&self, service: &str) -> Result<SocketAddr> {
        let config = self.services.get(service).ok_or_else(|| {
            UaipError::InvalidConfiguration(format!("Service {} not found", service))
        })?;

        config.socket_addr()
    }

    /// Add adapter endpoint
    pub fn add_adapter(&mut self, name: String, endpoint: AdapterEndpoint) {
        self.adapters.insert(name, endpoint);
    }

    /// Get adapter endpoint
    pub fn get_adapter(&self, name: &str) -> Option<&AdapterEndpoint> {
        self.adapters.get(name)
    }

    /// List all enabled services
    pub fn enabled_services(&self) -> Vec<(&String, &ServiceConfig)> {
        self.services
            .iter()
            .filter(|(_, config)| config.enabled)
            .collect()
    }
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Host address (IP or hostname)
    pub host: String,

    /// Port number
    pub port: u16,

    /// Protocol (HTTP, HTTPS, TCP, etc.)
    pub protocol: Protocol,

    /// Whether service is enabled
    pub enabled: bool,

    /// Health check endpoint path
    pub health_check_path: Option<String>,

    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
}

impl ServiceConfig {
    /// Get socket address
    pub fn socket_addr(&self) -> Result<SocketAddr> {
        let ip: IpAddr = self
            .host
            .parse()
            .map_err(|_| UaipError::InvalidConfiguration(format!("Invalid host: {}", self.host)))?;
        Ok(SocketAddr::new(ip, self.port))
    }

    /// Get full URL
    pub fn url(&self) -> String {
        format!("{}://{}:{}", self.protocol.scheme(), self.host, self.port)
    }

    /// Get timeout duration
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }
}

/// Communication protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Http,
    Https,
    Tcp,
    Udp,
    WebSocket,
    WebSocketSecure,
    Mqtt,
    Mqtts,
    ModbusTcp,
    OpcUa,
}

impl Protocol {
    /// Get protocol scheme
    pub fn scheme(&self) -> &'static str {
        match self {
            Protocol::Http => "http",
            Protocol::Https => "https",
            Protocol::Tcp => "tcp",
            Protocol::Udp => "udp",
            Protocol::WebSocket => "ws",
            Protocol::WebSocketSecure => "wss",
            Protocol::Mqtt => "mqtt",
            Protocol::Mqtts => "mqtts",
            Protocol::ModbusTcp => "modbus+tcp",
            Protocol::OpcUa => "opc.tcp",
        }
    }

    /// Get default port for protocol
    pub fn default_port(&self) -> u16 {
        match self {
            Protocol::Http => 80,
            Protocol::Https => 443,
            Protocol::Tcp => 0,
            Protocol::Udp => 0,
            Protocol::WebSocket => 80,
            Protocol::WebSocketSecure => 443,
            Protocol::Mqtt => 1883,
            Protocol::Mqtts => 8883,
            Protocol::ModbusTcp => 502,
            Protocol::OpcUa => 4840,
        }
    }
}

/// Adapter endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterEndpoint {
    /// Endpoint type (modbus, opcua, etc.)
    pub adapter_type: String,

    /// Connection string or URL
    pub connection_string: String,

    /// Protocol
    pub protocol: Protocol,

    /// Authentication credentials
    pub auth: Option<AuthConfig>,

    /// Connection timeout in milliseconds
    pub timeout_ms: u64,

    /// Maximum retries
    pub max_retries: u32,

    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,

    /// Custom parameters
    pub parameters: HashMap<String, String>,
}

impl AdapterEndpoint {
    /// Create a Modbus TCP endpoint
    pub fn modbus_tcp(host: &str, port: u16, unit_id: u8) -> Self {
        let mut parameters = HashMap::new();
        parameters.insert("unit_id".to_string(), unit_id.to_string());

        Self {
            adapter_type: "modbus".to_string(),
            connection_string: format!("{}:{}", host, port),
            protocol: Protocol::ModbusTcp,
            auth: None,
            timeout_ms: 10000,
            max_retries: 3,
            retry_delay_ms: 1000,
            parameters,
        }
    }

    /// Create an OPC UA endpoint
    pub fn opcua(url: &str, username: Option<String>, password: Option<String>) -> Self {
        let auth = if let (Some(u), Some(p)) = (username, password) {
            Some(AuthConfig::UsernamePassword {
                username: u,
                password: p,
            })
        } else {
            None
        };

        Self {
            adapter_type: "opcua".to_string(),
            connection_string: url.to_string(),
            protocol: Protocol::OpcUa,
            auth,
            timeout_ms: 10000,
            max_retries: 3,
            retry_delay_ms: 1000,
            parameters: HashMap::new(),
        }
    }

    /// Create an HTTP endpoint
    pub fn http(base_url: &str, auth: Option<AuthConfig>) -> Self {
        let protocol = if base_url.starts_with("https") {
            Protocol::Https
        } else {
            Protocol::Http
        };

        Self {
            adapter_type: "http".to_string(),
            connection_string: base_url.to_string(),
            protocol,
            auth,
            timeout_ms: 30000,
            max_retries: 3,
            retry_delay_ms: 1000,
            parameters: HashMap::new(),
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthConfig {
    None,
    UsernamePassword { username: String, password: String },
    Token { token: String },
    ApiKey { key: String, header: String },
    Certificate { cert_path: String, key_path: String },
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Enable mDNS discovery
    pub enable_mdns: bool,

    /// mDNS service type
    pub mdns_service_type: String,

    /// Discovery interval in seconds
    pub discovery_interval_secs: u64,

    /// Enable network scanning
    pub enable_scan: bool,

    /// IP ranges to scan
    pub scan_ranges: Vec<IpRange>,

    /// Ports to scan for each protocol
    pub scan_ports: HashMap<String, Vec<u16>>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        let mut scan_ports = HashMap::new();
        scan_ports.insert("modbus".to_string(), vec![502, 5020]);
        scan_ports.insert("opcua".to_string(), vec![4840, 4841, 4842]);
        scan_ports.insert("http".to_string(), vec![80, 8080, 8000]);
        scan_ports.insert("mqtt".to_string(), vec![1883, 8883]);

        Self {
            enable_mdns: true,
            mdns_service_type: "_uaip._tcp.local.".to_string(),
            discovery_interval_secs: 300,
            enable_scan: false,
            scan_ranges: vec![],
            scan_ports,
        }
    }
}

/// IP address range for scanning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpRange {
    /// Start IP
    pub start: String,

    /// End IP
    pub end: String,

    /// CIDR notation (alternative to start/end)
    pub cidr: Option<String>,
}

/// Connection pooling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum connections per endpoint
    pub max_connections_per_endpoint: usize,

    /// Maximum idle connections
    pub max_idle_connections: usize,

    /// Connection idle timeout in seconds
    pub idle_timeout_secs: u64,

    /// Connection lifetime in seconds
    pub max_lifetime_secs: u64,

    /// Enable connection pooling
    pub enabled: bool,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections_per_endpoint: 10,
            max_idle_connections: 5,
            idle_timeout_secs: 300,
            max_lifetime_secs: 1800,
            enabled: true,
        }
    }
}

/// Network endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointInfo {
    /// Endpoint address
    pub address: SocketAddr,

    /// Protocol detected
    pub protocol: Protocol,

    /// Service name (if discovered via mDNS)
    pub service_name: Option<String>,

    /// Device information (if available)
    pub device_info: Option<String>,

    /// Last seen timestamp
    pub last_seen: chrono::DateTime<chrono::Utc>,

    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert!(config.services.contains_key("hub"));
        assert!(config.services.contains_key("metrics"));
        assert!(config.discovery.enable_mdns);
        assert!(config.connection_pool.enabled);
    }

    #[test]
    fn test_service_config_socket_addr() {
        let config = ServiceConfig {
            host: "127.0.0.1".to_string(),
            port: 8443,
            protocol: Protocol::Https,
            enabled: true,
            health_check_path: None,
            timeout_ms: 5000,
        };

        let addr = config.socket_addr().unwrap();
        assert_eq!(addr.port(), 8443);
    }

    #[test]
    fn test_service_config_url() {
        let config = ServiceConfig {
            host: "localhost".to_string(),
            port: 8443,
            protocol: Protocol::Https,
            enabled: true,
            health_check_path: None,
            timeout_ms: 5000,
        };

        assert_eq!(config.url(), "https://localhost:8443");
    }

    #[test]
    fn test_protocol_scheme() {
        assert_eq!(Protocol::Http.scheme(), "http");
        assert_eq!(Protocol::Https.scheme(), "https");
        assert_eq!(Protocol::ModbusTcp.scheme(), "modbus+tcp");
        assert_eq!(Protocol::OpcUa.scheme(), "opc.tcp");
    }

    #[test]
    fn test_protocol_default_port() {
        assert_eq!(Protocol::Http.default_port(), 80);
        assert_eq!(Protocol::Https.default_port(), 443);
        assert_eq!(Protocol::ModbusTcp.default_port(), 502);
        assert_eq!(Protocol::OpcUa.default_port(), 4840);
        assert_eq!(Protocol::Mqtt.default_port(), 1883);
    }

    #[test]
    fn test_adapter_endpoint_modbus() {
        let endpoint = AdapterEndpoint::modbus_tcp("192.168.1.100", 502, 1);
        assert_eq!(endpoint.adapter_type, "modbus");
        assert_eq!(endpoint.connection_string, "192.168.1.100:502");
        assert_eq!(endpoint.protocol, Protocol::ModbusTcp);
        assert_eq!(endpoint.parameters.get("unit_id"), Some(&"1".to_string()));
    }

    #[test]
    fn test_adapter_endpoint_opcua() {
        let endpoint = AdapterEndpoint::opcua(
            "opc.tcp://localhost:4840",
            Some("admin".to_string()),
            Some("password".to_string()),
        );

        assert_eq!(endpoint.adapter_type, "opcua");
        assert_eq!(endpoint.protocol, Protocol::OpcUa);
        assert!(endpoint.auth.is_some());
    }

    #[test]
    fn test_adapter_endpoint_http() {
        let endpoint = AdapterEndpoint::http("https://api.example.com", None);
        assert_eq!(endpoint.adapter_type, "http");
        assert_eq!(endpoint.protocol, Protocol::Https);
    }

    #[test]
    fn test_auth_config_serialization() {
        let auth = AuthConfig::UsernamePassword {
            username: "user".to_string(),
            password: "pass".to_string(),
        };

        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("username_password"));
        assert!(json.contains("user"));
    }

    #[test]
    fn test_connection_pool_config_default() {
        let config = ConnectionPoolConfig::default();
        assert_eq!(config.max_connections_per_endpoint, 10);
        assert_eq!(config.max_idle_connections, 5);
        assert!(config.enabled);
    }

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert!(config.enable_mdns);
        assert!(!config.enable_scan);
        assert!(config.scan_ports.contains_key("modbus"));
        assert!(config.scan_ports.contains_key("opcua"));
    }
}
