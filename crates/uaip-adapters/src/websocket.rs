//! WebSocket Protocol Adapter
//!
//! Provides WebSocket client functionality for real-time bidirectional communication.
//! Supports text and binary messages with automatic reconnection and ping/pong.

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};
use tracing::{debug, error, info, warn};

use uaip_core::{
    error::{Result, UaipError},
    message::UaipMessage,
};

/// WebSocket adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// WebSocket server URL
    pub url: String,

    /// Reconnection delay in milliseconds
    pub reconnect_delay_ms: u64,

    /// Maximum reconnection attempts (0 = infinite)
    pub max_reconnect_attempts: u32,

    /// Ping interval in seconds
    pub ping_interval_seconds: u64,

    /// Pong timeout in seconds
    pub pong_timeout_seconds: u64,

    /// Message buffer size
    pub message_buffer_size: usize,

    /// Enable TLS certificate verification
    #[serde(default = "default_true")]
    pub verify_tls: bool,
}

fn default_true() -> bool {
    true
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            url: "ws://localhost:8080/ws".to_string(),
            reconnect_delay_ms: 5000,
            max_reconnect_attempts: 0, // Infinite
            ping_interval_seconds: 30,
            pong_timeout_seconds: 10,
            message_buffer_size: 100,
            verify_tls: true,
        }
    }
}

/// WebSocket connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Connected and operational
    Connected,
    /// Connection failed
    Failed,
}

/// WebSocket message types
#[derive(Debug, Clone)]
pub enum WsMessage {
    /// Text message
    Text(String),
    /// Binary message
    Binary(Vec<u8>),
    /// UAIP message (boxed to reduce enum size)
    Uaip(Box<UaipMessage>),
}

/// Message handler callback type
pub type MessageHandler = Arc<dyn Fn(WsMessage) -> Result<()> + Send + Sync>;

/// WebSocket adapter for bidirectional communication
pub struct WebSocketAdapter {
    config: WebSocketConfig,
    state: Arc<RwLock<ConnectionState>>,
    message_tx: Option<mpsc::Sender<Message>>,
    message_handler: Arc<RwLock<Option<MessageHandler>>>,
}

impl WebSocketAdapter {
    /// Create a new WebSocket adapter
    pub fn new(config: WebSocketConfig) -> Self {
        info!("WebSocket adapter created for URL: {}", config.url);

        Self {
            config,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            message_tx: None,
            message_handler: Arc::new(RwLock::new(None)),
        }
    }

    /// Set message handler callback
    pub async fn set_message_handler<F>(&self, handler: F)
    where
        F: Fn(WsMessage) -> Result<()> + Send + Sync + 'static,
    {
        let mut handler_lock = self.message_handler.write().await;
        *handler_lock = Some(Arc::new(handler));
    }

    /// Connect to WebSocket server
    pub async fn connect(&mut self) -> Result<()> {
        *self.state.write().await = ConnectionState::Connecting;

        let (ws_stream, _) = connect_async(&self.config.url)
            .await
            .map_err(|e| UaipError::ConnectionError(format!("WebSocket connection failed: {}", e)))?;

        info!("WebSocket connected to {}", self.config.url);
        *self.state.write().await = ConnectionState::Connected;

        // Create message channel
        let (tx, rx) = mpsc::channel(self.config.message_buffer_size);
        self.message_tx = Some(tx);

        // Spawn handler task
        let state = Arc::clone(&self.state);
        let handler = Arc::clone(&self.message_handler);
        let config = self.config.clone();

        tokio::spawn(async move {
            Self::handle_connection(ws_stream, rx, state, handler, config).await;
        });

        Ok(())
    }

    /// Handle WebSocket connection
    async fn handle_connection(
        ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
        mut outgoing_rx: mpsc::Receiver<Message>,
        state: Arc<RwLock<ConnectionState>>,
        message_handler: Arc<RwLock<Option<MessageHandler>>>,
        config: WebSocketConfig,
    ) {
        let (mut write, mut read) = ws_stream.split();

        // Ping interval
        let mut ping_interval = interval(Duration::from_secs(config.ping_interval_seconds));

        loop {
            tokio::select! {
                // Receive from server
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            debug!("Received text message: {} bytes", text.len());
                            Self::handle_message(WsMessage::Text(text), &message_handler).await;
                        }
                        Some(Ok(Message::Binary(data))) => {
                            debug!("Received binary message: {} bytes", data.len());
                            Self::handle_message(WsMessage::Binary(data), &message_handler).await;
                        }
                        Some(Ok(Message::Ping(_))) => {
                            debug!("Received ping");
                        }
                        Some(Ok(Message::Pong(_))) => {
                            debug!("Received pong");
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("WebSocket connection closed by server");
                            *state.write().await = ConnectionState::Disconnected;
                            break;
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            *state.write().await = ConnectionState::Failed;
                            break;
                        }
                        None => {
                            warn!("WebSocket stream ended");
                            *state.write().await = ConnectionState::Disconnected;
                            break;
                        }
                        _ => {}
                    }
                }

                // Send to server
                Some(msg) = outgoing_rx.recv() => {
                    if let Err(e) = write.send(msg).await {
                        error!("Failed to send message: {}", e);
                        *state.write().await = ConnectionState::Failed;
                        break;
                    }
                }

                // Send periodic ping
                _ = ping_interval.tick() => {
                    debug!("Sending ping");
                    if let Err(e) = write.send(Message::Ping(vec![])).await {
                        error!("Failed to send ping: {}", e);
                        *state.write().await = ConnectionState::Failed;
                        break;
                    }
                }
            }
        }

        info!("WebSocket handler task exited");
    }

    /// Handle incoming message
    async fn handle_message(
        msg: WsMessage,
        handler: &Arc<RwLock<Option<MessageHandler>>>,
    ) {
        let handler_lock = handler.read().await;
        if let Some(h) = handler_lock.as_ref() {
            if let Err(e) = h(msg) {
                error!("Message handler error: {}", e);
            }
        } else {
            warn!("Received message but no handler is set");
        }
    }

    /// Send text message
    pub async fn send_text(&self, text: String) -> Result<()> {
        if let Some(tx) = &self.message_tx {
            tx.send(Message::Text(text))
                .await
                .map_err(|e| UaipError::ConnectionError(format!("Failed to send text: {}", e)))?;
            Ok(())
        } else {
            Err(UaipError::InvalidState("Not connected".to_string()))
        }
    }

    /// Send binary message
    pub async fn send_binary(&self, data: Vec<u8>) -> Result<()> {
        if let Some(tx) = &self.message_tx {
            tx.send(Message::Binary(data))
                .await
                .map_err(|e| UaipError::ConnectionError(format!("Failed to send binary: {}", e)))?;
            Ok(())
        } else {
            Err(UaipError::InvalidState("Not connected".to_string()))
        }
    }

    /// Send UAIP message
    pub async fn send_uaip_message(&self, message: &UaipMessage) -> Result<()> {
        let json = serde_json::to_string(message)
            .map_err(|e| UaipError::InvalidMessage(format!("Failed to serialize message: {}", e)))?;
        self.send_text(json).await
    }

    /// Disconnect from server
    pub async fn disconnect(&self) -> Result<()> {
        if let Some(tx) = &self.message_tx {
            tx.send(Message::Close(None))
                .await
                .map_err(|e| UaipError::ConnectionError(format!("Failed to send close: {}", e)))?;
        }
        *self.state.write().await = ConnectionState::Disconnected;
        info!("WebSocket disconnected");
        Ok(())
    }

    /// Get current connection state
    pub async fn get_state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        *self.state.read().await == ConnectionState::Connected
    }

    /// Get configuration
    pub fn get_config(&self) -> &WebSocketConfig {
        &self.config
    }

    /// Connect with automatic reconnection
    pub async fn connect_with_retry(&mut self) -> Result<()> {
        let mut attempts = 0;
        let max_attempts = self.config.max_reconnect_attempts;

        loop {
            match self.connect().await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    attempts += 1;
                    if max_attempts > 0 && attempts >= max_attempts {
                        return Err(UaipError::MaxRetriesExceeded(format!(
                            "Failed to connect after {} attempts",
                            attempts
                        )));
                    }

                    error!(
                        "Connection attempt {} failed: {}, retrying in {}ms",
                        attempts, e, self.config.reconnect_delay_ms
                    );

                    sleep(Duration::from_millis(self.config.reconnect_delay_ms)).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_config_default() {
        let config = WebSocketConfig::default();
        assert_eq!(config.url, "ws://localhost:8080/ws");
        assert_eq!(config.reconnect_delay_ms, 5000);
        assert_eq!(config.max_reconnect_attempts, 0);
        assert_eq!(config.ping_interval_seconds, 30);
        assert!(config.verify_tls);
        assert_eq!(config.message_buffer_size, 100);
    }

    #[test]
    fn test_websocket_config_custom() {
        let config = WebSocketConfig {
            url: "wss://secure.example.com/ws".to_string(),
            reconnect_delay_ms: 10000,
            max_reconnect_attempts: 5,
            ping_interval_seconds: 60,
            pong_timeout_seconds: 15,
            message_buffer_size: 200,
            verify_tls: false,
        };

        assert_eq!(config.url, "wss://secure.example.com/ws");
        assert_eq!(config.reconnect_delay_ms, 10000);
        assert_eq!(config.max_reconnect_attempts, 5);
        assert!(!config.verify_tls);
    }

    #[test]
    fn test_connection_state() {
        assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_websocket_adapter_creation() {
        let config = WebSocketConfig::default();
        let adapter = WebSocketAdapter::new(config);

        assert_eq!(adapter.get_state().await, ConnectionState::Disconnected);
        assert!(!adapter.is_connected().await);
    }

    #[tokio::test]
    async fn test_websocket_adapter_with_custom_config() {
        let config = WebSocketConfig {
            url: "wss://test.com/ws".to_string(),
            reconnect_delay_ms: 1000,
            max_reconnect_attempts: 3,
            ping_interval_seconds: 15,
            pong_timeout_seconds: 5,
            message_buffer_size: 50,
            verify_tls: true,
        };

        let adapter = WebSocketAdapter::new(config.clone());
        assert_eq!(adapter.get_config().url, "wss://test.com/ws");
        assert_eq!(adapter.get_config().reconnect_delay_ms, 1000);
    }

    #[tokio::test]
    async fn test_send_without_connection() {
        let config = WebSocketConfig::default();
        let adapter = WebSocketAdapter::new(config);

        let result = adapter.send_text("test".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_message_handler_set() {
        let config = WebSocketConfig::default();
        let adapter = WebSocketAdapter::new(config);

        adapter.set_message_handler(|_msg| Ok(())).await;

        // Handler is set internally, we can't directly test it without a connection
        // but we can verify the adapter was created successfully
        assert_eq!(adapter.get_state().await, ConnectionState::Disconnected);
    }
}
