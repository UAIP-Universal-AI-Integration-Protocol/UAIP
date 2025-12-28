//! Real-time Streaming Module
//!
//! Provides streaming session management and real-time data delivery for media files.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::media::{StreamConfig, StreamQuality};

/// Active streaming session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingSession {
    /// Unique session ID
    pub id: Uuid,

    /// Media file being streamed
    pub media_id: Uuid,

    /// Stream configuration
    pub config: StreamConfig,

    /// Session state
    pub state: SessionState,

    /// Connected clients/viewers
    pub clients: Vec<StreamClient>,

    /// Session started timestamp
    pub started_at: DateTime<Utc>,

    /// Last activity timestamp
    pub last_activity_at: DateTime<Utc>,

    /// Session statistics
    pub stats: StreamingStats,
}

impl StreamingSession {
    /// Create a new streaming session
    pub fn new(media_id: Uuid, config: StreamConfig) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            media_id,
            config,
            state: SessionState::Initializing,
            clients: Vec::new(),
            started_at: now,
            last_activity_at: now,
            stats: StreamingStats::default(),
        }
    }

    /// Add a client to the session
    pub fn add_client(&mut self, client: StreamClient) {
        self.clients.push(client);
        self.stats.total_clients += 1;
        self.stats.current_clients = self.clients.len() as u32;
        self.last_activity_at = Utc::now();
    }

    /// Remove a client from the session
    pub fn remove_client(&mut self, client_id: &Uuid) {
        self.clients.retain(|c| c.id != *client_id);
        self.stats.current_clients = self.clients.len() as u32;
        self.last_activity_at = Utc::now();
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        matches!(self.state, SessionState::Streaming)
    }
}

/// Streaming session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionState {
    /// Session is being initialized
    Initializing,
    /// Session is ready to stream
    Ready,
    /// Currently streaming
    Streaming,
    /// Stream is paused
    Paused,
    /// Stream is buffering
    Buffering,
    /// Stream ended normally
    Ended,
    /// Stream encountered an error
    Error,
}

/// Connected streaming client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamClient {
    /// Unique client ID
    pub id: Uuid,

    /// Client IP address
    pub ip_address: String,

    /// User agent string
    pub user_agent: Option<String>,

    /// Requested quality level
    pub quality: StreamQuality,

    /// Client connection timestamp
    pub connected_at: DateTime<Utc>,

    /// Last heartbeat/activity
    pub last_heartbeat: DateTime<Utc>,

    /// Bytes transferred to this client
    pub bytes_transferred: u64,

    /// Client-specific metadata
    pub metadata: HashMap<String, String>,
}

impl StreamClient {
    /// Create a new streaming client
    pub fn new(ip_address: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            ip_address,
            user_agent: None,
            quality: StreamQuality::Auto,
            connected_at: now,
            last_heartbeat: now,
            bytes_transferred: 0,
            metadata: HashMap::new(),
        }
    }

    /// Check if client connection is alive
    pub fn is_alive(&self, timeout_secs: u64) -> bool {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.last_heartbeat);
        duration.num_seconds() as u64 <= timeout_secs
    }
}

/// Streaming session statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StreamingStats {
    /// Total number of clients that connected
    pub total_clients: u32,

    /// Currently connected clients
    pub current_clients: u32,

    /// Peak concurrent clients
    pub peak_clients: u32,

    /// Total bytes streamed
    pub total_bytes: u64,

    /// Average bitrate in kbps
    pub avg_bitrate_kbps: u32,

    /// Total stream duration in seconds
    pub duration_secs: f64,

    /// Number of buffer events
    pub buffer_events: u32,

    /// Number of errors
    pub error_count: u32,
}

impl StreamingStats {
    /// Update peak clients if current exceeds it
    pub fn update_peak(&mut self, current: u32) {
        if current > self.peak_clients {
            self.peak_clients = current;
        }
    }

    /// Record bytes sent
    pub fn record_bytes(&mut self, bytes: u64) {
        self.total_bytes += bytes;
    }

    /// Record a buffer event
    pub fn record_buffer(&mut self) {
        self.buffer_events += 1;
    }

    /// Record an error
    pub fn record_error(&mut self) {
        self.error_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_session_creation() {
        let media_id = Uuid::new_v4();
        let config = StreamConfig::new(media_id, StreamProtocol::Hls);
        let session = StreamingSession::new(media_id, config);

        assert_eq!(session.media_id, media_id);
        assert_eq!(session.state, SessionState::Initializing);
        assert_eq!(session.clients.len(), 0);
    }

    #[test]
    fn test_client_management() {
        let media_id = Uuid::new_v4();
        let config = StreamConfig::new(media_id, StreamProtocol::Hls);
        let mut session = StreamingSession::new(media_id, config);

        let client = StreamClient::new("192.168.1.100".to_string());
        let client_id = client.id;

        session.add_client(client);
        assert_eq!(session.clients.len(), 1);
        assert_eq!(session.stats.total_clients, 1);
        assert_eq!(session.stats.current_clients, 1);

        session.remove_client(&client_id);
        assert_eq!(session.clients.len(), 0);
        assert_eq!(session.stats.current_clients, 0);
    }

    #[test]
    fn test_streaming_stats() {
        let mut stats = StreamingStats::default();

        stats.record_bytes(1024);
        assert_eq!(stats.total_bytes, 1024);

        stats.record_buffer();
        assert_eq!(stats.buffer_events, 1);

        stats.record_error();
        assert_eq!(stats.error_count, 1);

        stats.update_peak(10);
        assert_eq!(stats.peak_clients, 10);

        stats.update_peak(5); // Should not update
        assert_eq!(stats.peak_clients, 10);
    }

    #[test]
    fn test_stream_client_creation() {
        let client = StreamClient::new("10.0.0.1".to_string());
        assert_eq!(client.ip_address, "10.0.0.1");
        assert_eq!(client.quality, StreamQuality::Auto);
        assert_eq!(client.bytes_transferred, 0);
    }
}
