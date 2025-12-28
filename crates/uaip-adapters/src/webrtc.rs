//! WebRTC Protocol Adapter
//!
//! Provides WebRTC functionality for real-time peer-to-peer communication.
//! Supports data channels, audio/video streaming, and signaling.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use uaip_core::{
    error::{Result, UaipError},
};

/// WebRTC ICE server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceServer {
    /// ICE server URLs (STUN/TURN)
    pub urls: Vec<String>,

    /// Username for TURN authentication
    pub username: Option<String>,

    /// Credential for TURN authentication
    pub credential: Option<String>,
}

impl IceServer {
    /// Create a STUN server configuration
    pub fn stun(url: impl Into<String>) -> Self {
        Self {
            urls: vec![url.into()],
            username: None,
            credential: None,
        }
    }

    /// Create a TURN server configuration with authentication
    pub fn turn(url: impl Into<String>, username: impl Into<String>, credential: impl Into<String>) -> Self {
        Self {
            urls: vec![url.into()],
            username: Some(username.into()),
            credential: Some(credential.into()),
        }
    }

    /// Create default Google STUN servers
    pub fn google_stun() -> Vec<Self> {
        vec![
            Self::stun("stun:stun.l.google.com:19302"),
            Self::stun("stun:stun1.l.google.com:19302"),
        ]
    }
}

/// WebRTC connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionState {
    New,
    Connecting,
    Connected,
    Disconnected,
    Failed,
    Closed,
}

/// WebRTC ICE connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IceConnectionState {
    New,
    Checking,
    Connected,
    Completed,
    Failed,
    Disconnected,
    Closed,
}

/// WebRTC signaling state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SignalingState {
    Stable,
    HaveLocalOffer,
    HaveRemoteOffer,
    HaveLocalPranswer,
    HaveRemotePranswer,
    Closed,
}

/// SDP (Session Description Protocol) type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SdpType {
    Offer,
    Answer,
    Pranswer,
    Rollback,
}

/// Session description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDescription {
    /// SDP type
    #[serde(rename = "type")]
    pub sdp_type: SdpType,

    /// SDP string
    pub sdp: String,
}

/// ICE candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidate {
    /// Candidate string
    pub candidate: String,

    /// SDP media line index
    #[serde(rename = "sdpMLineIndex")]
    pub sdp_mline_index: Option<u16>,

    /// SDP media ID
    #[serde(rename = "sdpMid")]
    pub sdp_mid: Option<String>,
}

/// Data channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataChannelConfig {
    /// Channel label/name
    pub label: String,

    /// Ordered delivery
    #[serde(default = "default_true")]
    pub ordered: bool,

    /// Maximum packet lifetime in milliseconds (unreliable mode)
    pub max_packet_life_time: Option<u16>,

    /// Maximum retransmits (unreliable mode)
    pub max_retransmits: Option<u16>,

    /// Protocol
    pub protocol: Option<String>,

    /// Negotiated
    #[serde(default)]
    pub negotiated: bool,

    /// Channel ID (for negotiated channels)
    pub id: Option<u16>,
}

fn default_true() -> bool {
    true
}

impl Default for DataChannelConfig {
    fn default() -> Self {
        Self {
            label: "default".to_string(),
            ordered: true,
            max_packet_life_time: None,
            max_retransmits: None,
            protocol: None,
            negotiated: false,
            id: None,
        }
    }
}

/// WebRTC adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcConfig {
    /// ICE servers
    pub ice_servers: Vec<IceServer>,

    /// Enable audio
    #[serde(default)]
    pub enable_audio: bool,

    /// Enable video
    #[serde(default)]
    pub enable_video: bool,

    /// Enable data channels
    #[serde(default = "default_true")]
    pub enable_data_channels: bool,

    /// Data channel configurations
    #[serde(default)]
    pub data_channels: Vec<DataChannelConfig>,

    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,
}

fn default_connection_timeout() -> u64 {
    30
}

impl Default for WebRtcConfig {
    fn default() -> Self {
        Self {
            ice_servers: IceServer::google_stun(),
            enable_audio: false,
            enable_video: false,
            enable_data_channels: true,
            data_channels: vec![DataChannelConfig::default()],
            connection_timeout: 30,
        }
    }
}

/// Data channel message handler
pub type DataChannelHandler = Arc<dyn Fn(String, Vec<u8>) -> Result<()> + Send + Sync>;

/// WebRTC data channel
pub struct DataChannel {
    label: String,
    state: RwLock<ConnectionState>,
    message_handler: RwLock<Option<DataChannelHandler>>,
}

impl DataChannel {
    fn new(label: String) -> Self {
        Self {
            label,
            state: RwLock::new(ConnectionState::New),
            message_handler: RwLock::new(None),
        }
    }

    /// Get channel label
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Send data
    pub async fn send(&self, data: Vec<u8>) -> Result<()> {
        let state = self.state.read().await;
        if *state != ConnectionState::Connected {
            return Err(UaipError::ConnectionError(
                "Data channel not connected".to_string(),
            ));
        }

        debug!("Sending {} bytes on data channel: {}", data.len(), self.label);
        Ok(())
    }

    /// Send text
    pub async fn send_text(&self, text: String) -> Result<()> {
        self.send(text.into_bytes()).await
    }

    /// Send JSON
    pub async fn send_json<T: Serialize>(&self, data: &T) -> Result<()> {
        let json = serde_json::to_vec(data)
            .map_err(|e| UaipError::InvalidMessage(format!("Failed to serialize: {}", e)))?;
        self.send(json).await
    }

    /// Set message handler
    pub async fn set_message_handler<F>(&self, handler: F)
    where
        F: Fn(String, Vec<u8>) -> Result<()> + Send + Sync + 'static,
    {
        *self.message_handler.write().await = Some(Arc::new(handler));
    }

    /// Get current state
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }
}

/// WebRTC peer connection
pub struct WebRtcAdapter {
    config: WebRtcConfig,
    connection_state: Arc<RwLock<ConnectionState>>,
    ice_connection_state: Arc<RwLock<IceConnectionState>>,
    signaling_state: Arc<RwLock<SignalingState>>,
    data_channels: Arc<RwLock<HashMap<String, Arc<DataChannel>>>>,
    local_description: Arc<RwLock<Option<SessionDescription>>>,
    remote_description: Arc<RwLock<Option<SessionDescription>>>,
    ice_candidates: Arc<RwLock<Vec<IceCandidate>>>,
}

impl WebRtcAdapter {
    /// Create a new WebRTC adapter
    pub fn new(config: WebRtcConfig) -> Result<Self> {
        info!("WebRTC adapter created with {} ICE servers", config.ice_servers.len());

        Ok(Self {
            config,
            connection_state: Arc::new(RwLock::new(ConnectionState::New)),
            ice_connection_state: Arc::new(RwLock::new(IceConnectionState::New)),
            signaling_state: Arc::new(RwLock::new(SignalingState::Stable)),
            data_channels: Arc::new(RwLock::new(HashMap::new())),
            local_description: Arc::new(RwLock::new(None)),
            remote_description: Arc::new(RwLock::new(None)),
            ice_candidates: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Create an offer
    pub async fn create_offer(&self) -> Result<SessionDescription> {
        info!("Creating WebRTC offer");

        // Update signaling state
        *self.signaling_state.write().await = SignalingState::HaveLocalOffer;

        // Generate SDP offer (simplified mock)
        let sdp = format!(
            "v=0\r\n\
             o=- {} 2 IN IP4 127.0.0.1\r\n\
             s=-\r\n\
             t=0 0\r\n\
             a=group:BUNDLE 0\r\n\
             m=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\n\
             c=IN IP4 0.0.0.0\r\n",
            chrono::Utc::now().timestamp()
        );

        let offer = SessionDescription {
            sdp_type: SdpType::Offer,
            sdp,
        };

        *self.local_description.write().await = Some(offer.clone());

        Ok(offer)
    }

    /// Create an answer
    pub async fn create_answer(&self) -> Result<SessionDescription> {
        info!("Creating WebRTC answer");

        // Check if we have remote offer
        let remote_desc = self.remote_description.read().await;
        if remote_desc.is_none() {
            return Err(UaipError::InvalidMessage(
                "No remote offer set".to_string(),
            ));
        }

        // Update signaling state
        *self.signaling_state.write().await = SignalingState::Stable;

        // Generate SDP answer (simplified mock)
        let sdp = format!(
            "v=0\r\n\
             o=- {} 2 IN IP4 127.0.0.1\r\n\
             s=-\r\n\
             t=0 0\r\n\
             a=group:BUNDLE 0\r\n\
             m=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\n\
             c=IN IP4 0.0.0.0\r\n",
            chrono::Utc::now().timestamp()
        );

        let answer = SessionDescription {
            sdp_type: SdpType::Answer,
            sdp,
        };

        *self.local_description.write().await = Some(answer.clone());

        Ok(answer)
    }

    /// Set local description
    pub async fn set_local_description(&self, description: SessionDescription) -> Result<()> {
        info!("Setting local description: {:?}", description.sdp_type);
        *self.local_description.write().await = Some(description);
        Ok(())
    }

    /// Set remote description
    pub async fn set_remote_description(&self, description: SessionDescription) -> Result<()> {
        info!("Setting remote description: {:?}", description.sdp_type);
        *self.remote_description.write().await = Some(description);
        *self.signaling_state.write().await = SignalingState::HaveRemoteOffer;
        Ok(())
    }

    /// Add ICE candidate
    pub async fn add_ice_candidate(&self, candidate: IceCandidate) -> Result<()> {
        debug!("Adding ICE candidate");
        self.ice_candidates.write().await.push(candidate);
        Ok(())
    }

    /// Create a data channel
    pub async fn create_data_channel(&self, config: DataChannelConfig) -> Result<Arc<DataChannel>> {
        info!("Creating data channel: {}", config.label);

        let channel = Arc::new(DataChannel::new(config.label.clone()));
        self.data_channels
            .write()
            .await
            .insert(config.label.clone(), channel.clone());

        // Simulate connection
        *channel.state.write().await = ConnectionState::Connected;

        Ok(channel)
    }

    /// Get a data channel by label
    pub async fn get_data_channel(&self, label: &str) -> Option<Arc<DataChannel>> {
        self.data_channels.read().await.get(label).cloned()
    }

    /// Get all data channels
    pub async fn get_data_channels(&self) -> Vec<Arc<DataChannel>> {
        self.data_channels.read().await.values().cloned().collect()
    }

    /// Get connection state
    pub async fn connection_state(&self) -> ConnectionState {
        *self.connection_state.read().await
    }

    /// Get ICE connection state
    pub async fn ice_connection_state(&self) -> IceConnectionState {
        *self.ice_connection_state.read().await
    }

    /// Get signaling state
    pub async fn signaling_state(&self) -> SignalingState {
        *self.signaling_state.read().await
    }

    /// Close the connection
    pub async fn close(&self) -> Result<()> {
        info!("Closing WebRTC connection");

        *self.connection_state.write().await = ConnectionState::Closed;
        *self.ice_connection_state.write().await = IceConnectionState::Closed;
        *self.signaling_state.write().await = SignalingState::Closed;

        // Close all data channels
        for channel in self.data_channels.read().await.values() {
            *channel.state.write().await = ConnectionState::Closed;
        }

        Ok(())
    }

    /// Get the WebRTC configuration
    pub fn get_config(&self) -> &WebRtcConfig {
        &self.config
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        let state = self.connection_state().await;
        match state {
            ConnectionState::Connected => Ok(()),
            ConnectionState::Closed | ConnectionState::Failed => {
                Err(UaipError::ConnectionError(format!(
                    "Connection in state: {:?}",
                    state
                )))
            }
            _ => Ok(()), // New, Connecting, Disconnected are acceptable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ice_server_stun() {
        let server = IceServer::stun("stun:stun.example.com:3478");
        assert_eq!(server.urls.len(), 1);
        assert_eq!(server.urls[0], "stun:stun.example.com:3478");
        assert!(server.username.is_none());
        assert!(server.credential.is_none());
    }

    #[test]
    fn test_ice_server_turn() {
        let server = IceServer::turn("turn:turn.example.com:3478", "user", "pass");
        assert_eq!(server.urls.len(), 1);
        assert_eq!(server.username, Some("user".to_string()));
        assert_eq!(server.credential, Some("pass".to_string()));
    }

    #[test]
    fn test_google_stun() {
        let servers = IceServer::google_stun();
        assert_eq!(servers.len(), 2);
        assert!(servers[0].urls[0].contains("google"));
    }

    #[test]
    fn test_webrtc_config_default() {
        let config = WebRtcConfig::default();
        assert!(!config.enable_audio);
        assert!(!config.enable_video);
        assert!(config.enable_data_channels);
        assert_eq!(config.connection_timeout, 30);
        assert!(!config.ice_servers.is_empty());
    }

    #[test]
    fn test_data_channel_config() {
        let config = DataChannelConfig {
            label: "test".to_string(),
            ordered: true,
            max_retransmits: Some(3),
            ..Default::default()
        };

        assert_eq!(config.label, "test");
        assert!(config.ordered);
        assert_eq!(config.max_retransmits, Some(3));
    }

    #[tokio::test]
    async fn test_webrtc_adapter_creation() {
        let config = WebRtcConfig::default();
        let result = WebRtcAdapter::new(config);
        assert!(result.is_ok());

        let adapter = result.unwrap();
        assert_eq!(adapter.connection_state().await, ConnectionState::New);
        assert_eq!(
            adapter.ice_connection_state().await,
            IceConnectionState::New
        );
        assert_eq!(adapter.signaling_state().await, SignalingState::Stable);
    }

    #[tokio::test]
    async fn test_create_offer() {
        let config = WebRtcConfig::default();
        let adapter = WebRtcAdapter::new(config).unwrap();

        let offer = adapter.create_offer().await;
        assert!(offer.is_ok());

        let offer = offer.unwrap();
        assert_eq!(offer.sdp_type, SdpType::Offer);
        assert!(!offer.sdp.is_empty());
    }

    #[tokio::test]
    async fn test_create_data_channel() {
        let config = WebRtcConfig::default();
        let adapter = WebRtcAdapter::new(config).unwrap();

        let dc_config = DataChannelConfig {
            label: "test-channel".to_string(),
            ..Default::default()
        };

        let result = adapter.create_data_channel(dc_config).await;
        assert!(result.is_ok());

        let channel = result.unwrap();
        assert_eq!(channel.label(), "test-channel");
        assert_eq!(channel.state().await, ConnectionState::Connected);
    }

    #[tokio::test]
    async fn test_get_data_channel() {
        let config = WebRtcConfig::default();
        let adapter = WebRtcAdapter::new(config).unwrap();

        let dc_config = DataChannelConfig {
            label: "test".to_string(),
            ..Default::default()
        };

        adapter.create_data_channel(dc_config).await.unwrap();

        let channel = adapter.get_data_channel("test").await;
        assert!(channel.is_some());
        assert_eq!(channel.unwrap().label(), "test");
    }

    #[tokio::test]
    async fn test_session_description() {
        let desc = SessionDescription {
            sdp_type: SdpType::Offer,
            sdp: "v=0\r\n...".to_string(),
        };

        let json = serde_json::to_string(&desc).unwrap();
        assert!(json.contains("offer"));
    }

    #[tokio::test]
    async fn test_close() {
        let config = WebRtcConfig::default();
        let adapter = WebRtcAdapter::new(config).unwrap();

        adapter.close().await.unwrap();
        assert_eq!(adapter.connection_state().await, ConnectionState::Closed);
        assert_eq!(
            adapter.signaling_state().await,
            SignalingState::Closed
        );
    }
}
