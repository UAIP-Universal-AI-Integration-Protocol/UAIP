//! NATS message broker integration

use async_nats::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

use uaip_core::error::{UaipError, UaipResult};
use uaip_core::message::UaipMessage;

/// NATS broker configuration
#[derive(Debug, Clone)]
pub struct NatsConfig {
    /// NATS server URL
    pub server_url: String,
    /// Subject prefix for UAIP messages
    pub subject_prefix: String,
    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            server_url: "nats://localhost:4222".to_string(),
            subject_prefix: "uaip".to_string(),
            connect_timeout_secs: 5,
        }
    }
}

/// NATS broker service
pub struct NatsBroker {
    client: Arc<RwLock<Option<Client>>>,
    config: NatsConfig,
    stats: Arc<RwLock<NatsStats>>,
}

/// NATS statistics
#[derive(Debug, Clone, Default)]
pub struct NatsStats {
    pub messages_published: u64,
    pub messages_received: u64,
    pub publish_errors: u64,
    pub connection_count: u64,
}

impl NatsBroker {
    /// Create a new NATS broker
    ///
    /// # Arguments
    /// * `config` - NATS configuration
    pub fn new(config: NatsConfig) -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            config,
            stats: Arc::new(RwLock::new(NatsStats::default())),
        }
    }

    /// Connect to NATS server
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn connect(&self) -> UaipResult<()> {
        let client = async_nats::connect(&self.config.server_url)
            .await
            .map_err(|e| UaipError::ConnectionError(format!("Failed to connect to NATS: {}", e)))?;

        let mut client_lock = self.client.write().await;
        *client_lock = Some(client);

        let mut stats = self.stats.write().await;
        stats.connection_count += 1;

        Ok(())
    }

    /// Disconnect from NATS server
    pub async fn disconnect(&self) {
        let mut client_lock = self.client.write().await;
        *client_lock = None;
    }

    /// Check if connected to NATS
    ///
    /// # Returns
    /// * `bool` - True if connected
    pub async fn is_connected(&self) -> bool {
        let client_lock = self.client.read().await;
        client_lock.is_some()
    }

    /// Publish a message to NATS
    ///
    /// # Arguments
    /// * `message` - UAIP message to publish
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn publish(&self, message: &UaipMessage) -> UaipResult<()> {
        let client_lock = self.client.read().await;
        let client = client_lock
            .as_ref()
            .ok_or_else(|| UaipError::ConnectionError("Not connected to NATS".to_string()))?;

        // Build subject: uaip.{recipient_type}.{recipient_id}
        let subject = format!(
            "{}.{:?}.{}",
            self.config.subject_prefix,
            message.header.recipient.entity_type,
            message.header.recipient.id
        );

        // Serialize message to JSON
        let payload = serde_json::to_vec(message).map_err(UaipError::SerializationError)?;

        // Publish to NATS
        match client.publish(subject, payload.into()).await {
            Ok(_) => {
                let mut stats = self.stats.write().await;
                stats.messages_published += 1;
                Ok(())
            }
            Err(e) => {
                let mut stats = self.stats.write().await;
                stats.publish_errors += 1;
                Err(UaipError::ConnectionError(format!(
                    "Failed to publish to NATS: {}",
                    e
                )))
            }
        }
    }

    /// Subscribe to messages for a specific recipient
    ///
    /// # Arguments
    /// * `recipient_type` - Type of recipient (Device, AiAgent, etc.)
    /// * `recipient_id` - Recipient identifier
    ///
    /// # Returns
    /// * `Result<async_nats::Subscriber>` - NATS subscriber
    pub async fn subscribe(
        &self,
        recipient_type: &str,
        recipient_id: &str,
    ) -> UaipResult<async_nats::Subscriber> {
        let client_lock = self.client.read().await;
        let client = client_lock
            .as_ref()
            .ok_or_else(|| UaipError::ConnectionError("Not connected to NATS".to_string()))?;

        let subject = format!(
            "{}.{}.{}",
            self.config.subject_prefix, recipient_type, recipient_id
        );

        let subscriber = client.subscribe(subject).await.map_err(|e| {
            UaipError::ConnectionError(format!("Failed to subscribe to NATS: {}", e))
        })?;

        Ok(subscriber)
    }

    /// Subscribe to all messages with a wildcard
    ///
    /// # Returns
    /// * `Result<async_nats::Subscriber>` - NATS subscriber
    pub async fn subscribe_all(&self) -> UaipResult<async_nats::Subscriber> {
        let client_lock = self.client.read().await;
        let client = client_lock
            .as_ref()
            .ok_or_else(|| UaipError::ConnectionError("Not connected to NATS".to_string()))?;

        let subject = format!("{}.>", self.config.subject_prefix);

        let subscriber = client.subscribe(subject).await.map_err(|e| {
            UaipError::ConnectionError(format!("Failed to subscribe to NATS: {}", e))
        })?;

        Ok(subscriber)
    }

    /// Get NATS statistics
    ///
    /// # Returns
    /// * `NatsStats` - Current statistics
    pub async fn get_stats(&self) -> NatsStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Reset statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = NatsStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nats_config_default() {
        let config = NatsConfig::default();
        assert_eq!(config.server_url, "nats://localhost:4222");
        assert_eq!(config.subject_prefix, "uaip");
        assert_eq!(config.connect_timeout_secs, 5);
    }

    #[test]
    fn test_nats_config_custom() {
        let config = NatsConfig {
            server_url: "nats://custom:4222".to_string(),
            subject_prefix: "custom".to_string(),
            connect_timeout_secs: 10,
        };

        assert_eq!(config.server_url, "nats://custom:4222");
        assert_eq!(config.subject_prefix, "custom");
        assert_eq!(config.connect_timeout_secs, 10);
    }

    #[tokio::test]
    async fn test_nats_broker_creation() {
        let config = NatsConfig::default();
        let broker = NatsBroker::new(config);

        assert!(!broker.is_connected().await);
    }

    #[tokio::test]
    async fn test_nats_stats() {
        let config = NatsConfig::default();
        let broker = NatsBroker::new(config);

        let stats = broker.get_stats().await;
        assert_eq!(stats.messages_published, 0);
        assert_eq!(stats.messages_received, 0);
        assert_eq!(stats.publish_errors, 0);
    }

    // Note: Connection tests require a running NATS server
    // These are integration tests and should be run separately
}
