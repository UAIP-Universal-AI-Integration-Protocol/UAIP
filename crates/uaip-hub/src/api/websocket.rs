//! WebSocket API endpoints

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

use crate::api::rest::AppState;

/// WebSocket session ID
pub type SessionId = String;

/// WebSocket session manager
pub struct SessionManager {
    /// Active sessions (session_id -> sender)
    sessions: Arc<RwLock<HashMap<SessionId, broadcast::Sender<WsMessage>>>>,
    /// Broadcast channel for global events
    broadcast_tx: broadcast::Sender<WsMessage>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
        }
    }

    /// Register a new session
    pub async fn register(&self, session_id: SessionId) -> broadcast::Receiver<WsMessage> {
        let mut sessions = self.sessions.write().await;
        let (tx, rx) = broadcast::channel(100);
        sessions.insert(session_id.clone(), tx);
        debug!("Registered WebSocket session: {}", session_id);
        rx
    }

    /// Unregister a session
    pub async fn unregister(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        debug!("Unregistered WebSocket session: {}", session_id);
    }

    /// Broadcast message to all sessions
    pub async fn broadcast(&self, message: WsMessage) {
        let _ = self.broadcast_tx.send(message);
    }

    /// Send message to specific session
    pub async fn send_to_session(&self, session_id: &str, message: WsMessage) {
        let sessions = self.sessions.read().await;
        if let Some(tx) = sessions.get(session_id) {
            let _ = tx.send(message);
        }
    }

    /// Get number of active sessions
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// Subscribe to device events
    Subscribe {
        device_id: String,
    },
    /// Unsubscribe from device events
    Unsubscribe {
        device_id: String,
    },
    /// Device telemetry data
    Telemetry {
        device_id: String,
        timestamp: String,
        data: serde_json::Value,
    },
    /// Device command
    Command {
        device_id: String,
        action: String,
        parameters: Option<serde_json::Value>,
    },
    /// Device event notification
    Event {
        device_id: String,
        event_type: String,
        data: serde_json::Value,
    },
    /// Heartbeat/ping
    Ping,
    /// Heartbeat/pong
    Pong,
    /// Error message
    Error {
        code: String,
        message: String,
    },
    /// Success acknowledgment
    Ack {
        request_id: Option<String>,
        message: String,
    },
}

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket) {
    let session_id = uuid::Uuid::new_v4().to_string();
    info!("New WebSocket connection: {}", session_id);

    // Create session manager (in production, this would be shared via AppState)
    let session_manager = Arc::new(SessionManager::new());
    let mut rx = session_manager.register(session_id.clone()).await;

    let (mut sender, mut receiver) = socket.split();

    // Task for sending messages to client
    let session_id_clone = session_id.clone();
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let text = match serde_json::to_string(&msg) {
                Ok(json) => json,
                Err(e) => {
                    error!("Failed to serialize message: {}", e);
                    continue;
                }
            };

            if sender.send(Message::Text(text)).await.is_err() {
                warn!("Failed to send message to session: {}", session_id_clone);
                break;
            }
        }
    });

    // Task for receiving messages from client
    let session_id_clone = session_id.clone();
    let session_manager_clone = session_manager.clone();
    let recv_task = tokio::spawn(async move {
        // Heartbeat timer
        let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            tokio::select! {
                // Receive messages from client
                Some(result) = receiver.next() => {
                    match result {
                        Ok(msg) => {
                            if let Err(e) = handle_message(
                                msg,
                                &session_id_clone,
                                &session_manager_clone,
                            )
                            .await
                            {
                                error!("Error handling message: {}", e);
                            }
                        }
                        Err(e) => {
                            warn!("WebSocket error: {}", e);
                            break;
                        }
                    }
                }
                // Send periodic heartbeat
                _ = heartbeat_interval.tick() => {
                    session_manager_clone
                        .send_to_session(&session_id_clone, WsMessage::Ping)
                        .await;
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = send_task => {
            debug!("Send task completed for session: {}", session_id);
        }
        _ = recv_task => {
            debug!("Receive task completed for session: {}", session_id);
        }
    }

    // Clean up session
    session_manager.unregister(&session_id).await;
    info!("WebSocket connection closed: {}", session_id);
}

/// Handle incoming WebSocket message
async fn handle_message(
    msg: Message,
    session_id: &str,
    session_manager: &SessionManager,
) -> Result<(), String> {
    match msg {
        Message::Text(text) => {
            debug!("Received text message from {}: {}", session_id, text);

            let ws_message: WsMessage = serde_json::from_str(&text)
                .map_err(|e| format!("Failed to parse message: {}", e))?;

            match ws_message {
                WsMessage::Subscribe { device_id } => {
                    info!("Session {} subscribed to device: {}", session_id, device_id);
                    session_manager
                        .send_to_session(
                            session_id,
                            WsMessage::Ack {
                                request_id: None,
                                message: format!("Subscribed to device: {}", device_id),
                            },
                        )
                        .await;
                }
                WsMessage::Unsubscribe { device_id } => {
                    info!(
                        "Session {} unsubscribed from device: {}",
                        session_id, device_id
                    );
                    session_manager
                        .send_to_session(
                            session_id,
                            WsMessage::Ack {
                                request_id: None,
                                message: format!("Unsubscribed from device: {}", device_id),
                            },
                        )
                        .await;
                }
                WsMessage::Command {
                    device_id,
                    action,
                    parameters: _,
                } => {
                    info!(
                        "Received command from {}: {} on device {}",
                        session_id, action, device_id
                    );
                    // TODO: Forward command to device via router
                    session_manager
                        .send_to_session(
                            session_id,
                            WsMessage::Ack {
                                request_id: None,
                                message: format!("Command sent to device: {}", device_id),
                            },
                        )
                        .await;
                }
                WsMessage::Pong => {
                    debug!("Received pong from session: {}", session_id);
                }
                _ => {
                    warn!("Unhandled message type from session: {}", session_id);
                }
            }

            Ok(())
        }
        Message::Binary(_) => {
            debug!("Received binary message from session: {}", session_id);
            Ok(())
        }
        Message::Ping(_) => {
            debug!("Received ping from session: {}", session_id);
            Ok(())
        }
        Message::Pong(_) => {
            debug!("Received pong from session: {}", session_id);
            Ok(())
        }
        Message::Close(_) => {
            info!("Received close message from session: {}", session_id);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_manager_creation() {
        let manager = SessionManager::new();
        assert_eq!(manager.session_count().await, 0);
    }

    #[tokio::test]
    async fn test_session_registration() {
        let manager = SessionManager::new();
        let session_id = "test-session-1".to_string();

        let _rx = manager.register(session_id.clone()).await;
        assert_eq!(manager.session_count().await, 1);

        manager.unregister(&session_id).await;
        assert_eq!(manager.session_count().await, 0);
    }

    #[tokio::test]
    async fn test_ws_message_serialization() {
        let msg = WsMessage::Subscribe {
            device_id: "device-001".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("subscribe"));
        assert!(json.contains("device-001"));
    }

    #[tokio::test]
    async fn test_ws_message_deserialization() {
        let json = r#"{"type":"subscribe","device_id":"device-001"}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();

        match msg {
            WsMessage::Subscribe { device_id } => {
                assert_eq!(device_id, "device-001");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[tokio::test]
    async fn test_telemetry_message() {
        let msg = WsMessage::Telemetry {
            device_id: "device-001".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            data: serde_json::json!({"temperature": 25.5}),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("telemetry"));
        assert!(json.contains("temperature"));
    }

    #[tokio::test]
    async fn test_heartbeat_messages() {
        let ping = WsMessage::Ping;
        let pong = WsMessage::Pong;

        let ping_json = serde_json::to_string(&ping).unwrap();
        let pong_json = serde_json::to_string(&pong).unwrap();

        assert!(ping_json.contains("ping"));
        assert!(pong_json.contains("pong"));
    }

    #[tokio::test]
    async fn test_error_message() {
        let msg = WsMessage::Error {
            code: "INVALID_DEVICE".to_string(),
            message: "Device not found".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("INVALID_DEVICE"));
    }
}
