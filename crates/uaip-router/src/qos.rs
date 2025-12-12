//! QoS (Quality of Service) levels implementation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use uaip_core::error::{UaipError, UaipResult};
use uaip_core::message::UaipMessage;

/// QoS levels for message delivery
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QosLevel {
    /// QoS 0: At most once (fire-and-forget)
    AtMostOnce,
    /// QoS 1: At least once (requires acknowledgment)
    AtLeastOnce,
    /// QoS 2: Exactly once (two-phase commit)
    ExactlyOnce,
}

/// Message delivery state for QoS 1 and 2
#[derive(Debug, Clone, PartialEq)]
enum DeliveryState {
    /// Message sent, awaiting ACK (QoS 1)
    AwaitingAck,
    /// Message received, awaiting PUBREC (QoS 2 phase 1)
    AwaitingPubRec,
    /// PUBREC received, awaiting PUBCOMP (QoS 2 phase 2)
    AwaitingPubComp,
    /// Message fully delivered
    Completed,
}

/// Tracked message for QoS 1 and 2
#[derive(Debug, Clone)]
struct TrackedMessage {
    message: UaipMessage,
    state: DeliveryState,
    attempts: u32,
    max_attempts: u32,
}

/// QoS handler service
pub struct QosHandler {
    /// Tracked messages (message_id -> TrackedMessage)
    tracked: Arc<RwLock<HashMap<String, TrackedMessage>>>,
    /// Statistics
    stats: Arc<RwLock<QosStats>>,
}

/// QoS statistics
#[derive(Debug, Clone, Default)]
pub struct QosStats {
    pub qos0_sent: u64,
    pub qos1_sent: u64,
    pub qos1_acked: u64,
    pub qos2_sent: u64,
    pub qos2_completed: u64,
    pub retries: u64,
    pub failures: u64,
}

impl QosHandler {
    /// Create a new QoS handler
    pub fn new() -> Self {
        Self {
            tracked: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(QosStats::default())),
        }
    }

    /// Handle message delivery with specified QoS level
    ///
    /// # Arguments
    /// * `message` - Message to deliver
    /// * `qos_level` - QoS level to use
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn handle_message(
        &self,
        message: UaipMessage,
        qos_level: QosLevel,
    ) -> UaipResult<()> {
        match qos_level {
            QosLevel::AtMostOnce => self.handle_qos0(message).await,
            QosLevel::AtLeastOnce => self.handle_qos1(message).await,
            QosLevel::ExactlyOnce => self.handle_qos2(message).await,
        }
    }

    /// Handle QoS 0: Fire-and-forget
    ///
    /// Message is sent once with no acknowledgment
    async fn handle_qos0(&self, message: UaipMessage) -> UaipResult<()> {
        // Simulate message delivery
        self.deliver_message(&message).await?;

        let mut stats = self.stats.write().await;
        stats.qos0_sent += 1;

        Ok(())
    }

    /// Handle QoS 1: At-least-once delivery
    ///
    /// Message is sent and tracked until acknowledgment is received
    async fn handle_qos1(&self, message: UaipMessage) -> UaipResult<()> {
        let message_id = message.header.message_id.clone();

        // Track message
        {
            let mut tracked = self.tracked.write().await;
            tracked.insert(
                message_id.clone(),
                TrackedMessage {
                    message: message.clone(),
                    state: DeliveryState::AwaitingAck,
                    attempts: 1,
                    max_attempts: 3,
                },
            );
        }

        // Deliver message
        self.deliver_message(&message).await?;

        let mut stats = self.stats.write().await;
        stats.qos1_sent += 1;

        Ok(())
    }

    /// Handle QoS 2: Exactly-once delivery (two-phase commit)
    ///
    /// Message is delivered using a four-step handshake:
    /// 1. PUBLISH -> 2. PUBREC -> 3. PUBREL -> 4. PUBCOMP
    async fn handle_qos2(&self, message: UaipMessage) -> UaipResult<()> {
        let message_id = message.header.message_id.clone();

        // Track message (Phase 1: PUBLISH -> PUBREC)
        {
            let mut tracked = self.tracked.write().await;
            tracked.insert(
                message_id.clone(),
                TrackedMessage {
                    message: message.clone(),
                    state: DeliveryState::AwaitingPubRec,
                    attempts: 1,
                    max_attempts: 3,
                },
            );
        }

        // Deliver message
        self.deliver_message(&message).await?;

        let mut stats = self.stats.write().await;
        stats.qos2_sent += 1;

        Ok(())
    }

    /// Process acknowledgment for QoS 1
    ///
    /// # Arguments
    /// * `message_id` - Message identifier
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn acknowledge_qos1(&self, message_id: &str) -> UaipResult<()> {
        let mut tracked = self.tracked.write().await;

        if let Some(mut msg) = tracked.remove(message_id) {
            msg.state = DeliveryState::Completed;

            let mut stats = self.stats.write().await;
            stats.qos1_acked += 1;

            Ok(())
        } else {
            Err(UaipError::NotFound(format!(
                "Message {} not found in tracking",
                message_id
            )))
        }
    }

    /// Process PUBREC for QoS 2 (Phase 1 complete)
    ///
    /// # Arguments
    /// * `message_id` - Message identifier
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn acknowledge_qos2_pubrec(&self, message_id: &str) -> UaipResult<()> {
        let mut tracked = self.tracked.write().await;

        if let Some(msg) = tracked.get_mut(message_id) {
            if msg.state == DeliveryState::AwaitingPubRec {
                msg.state = DeliveryState::AwaitingPubComp;
                Ok(())
            } else {
                Err(UaipError::InvalidState(format!(
                    "Message {} not in PUBREC state",
                    message_id
                )))
            }
        } else {
            Err(UaipError::NotFound(format!(
                "Message {} not found in tracking",
                message_id
            )))
        }
    }

    /// Process PUBCOMP for QoS 2 (Phase 2 complete)
    ///
    /// # Arguments
    /// * `message_id` - Message identifier
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn acknowledge_qos2_pubcomp(&self, message_id: &str) -> UaipResult<()> {
        let mut tracked = self.tracked.write().await;

        if let Some(mut msg) = tracked.remove(message_id) {
            if msg.state == DeliveryState::AwaitingPubComp {
                msg.state = DeliveryState::Completed;

                let mut stats = self.stats.write().await;
                stats.qos2_completed += 1;

                Ok(())
            } else {
                // Re-insert if not in correct state
                tracked.insert(message_id.to_string(), msg);
                Err(UaipError::InvalidState(format!(
                    "Message {} not in PUBCOMP state",
                    message_id
                )))
            }
        } else {
            Err(UaipError::NotFound(format!(
                "Message {} not found in tracking",
                message_id
            )))
        }
    }

    /// Retry failed message delivery
    ///
    /// # Arguments
    /// * `message_id` - Message identifier
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn retry_message(&self, message_id: &str) -> UaipResult<()> {
        let mut tracked = self.tracked.write().await;

        if let Some(msg) = tracked.get_mut(message_id) {
            if msg.attempts >= msg.max_attempts {
                let mut stats = self.stats.write().await;
                stats.failures += 1;

                return Err(UaipError::MaxRetriesExceeded(format!(
                    "Message {} exceeded max retries",
                    message_id
                )));
            }

            msg.attempts += 1;

            // Simulate retry
            self.deliver_message(&msg.message).await?;

            let mut stats = self.stats.write().await;
            stats.retries += 1;

            Ok(())
        } else {
            Err(UaipError::NotFound(format!(
                "Message {} not found in tracking",
                message_id
            )))
        }
    }

    /// Get number of tracked messages
    pub async fn tracked_count(&self) -> usize {
        let tracked = self.tracked.read().await;
        tracked.len()
    }

    /// Get QoS statistics
    pub async fn get_stats(&self) -> QosStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Clear all tracked messages
    pub async fn clear_tracked(&self) {
        let mut tracked = self.tracked.write().await;
        tracked.clear();
    }

    /// Simulate message delivery (placeholder for actual delivery mechanism)
    async fn deliver_message(&self, _message: &UaipMessage) -> UaipResult<()> {
        // In a real implementation, this would:
        // - Send message over NATS/WebSocket/etc.
        // - Handle network errors
        // - Update connection state

        // For now, simulate successful delivery
        Ok(())
    }
}

impl Default for QosHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uaip_core::message::{
        Action, AuthMethod, Authentication, Entity, EntityType, Header, Metadata, Payload,
        Priority, Security,
    };

    fn create_test_message(message_id: &str) -> UaipMessage {
        UaipMessage {
            header: Header {
                version: "1.0".to_string(),
                message_id: message_id.to_string(),
                correlation_id: None,
                timestamp: chrono::Utc::now(),
                ttl: 300000,
                priority: Priority::Normal,
                sender: Entity {
                    id: "sender-1".to_string(),
                    entity_type: EntityType::Device,
                },
                recipient: Entity {
                    id: "recipient-1".to_string(),
                    entity_type: EntityType::AiAgent,
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
                action: Action::Execute,
                device_type: None,
                capability: None,
                data: None,
                parameters: None,
            },
            metadata: Metadata {
                requires_ack: false,
                ack_timeout: None,
                retry_policy: None,
                qos: uaip_core::message::QosLevel::AtMostOnce,
                content_type: None,
                user_data: None,
            },
        }
    }

    #[tokio::test]
    async fn test_qos0_fire_and_forget() {
        let handler = QosHandler::new();
        let message = create_test_message("msg-001");

        handler
            .handle_message(message, QosLevel::AtMostOnce)
            .await
            .unwrap();

        let stats = handler.get_stats().await;
        assert_eq!(stats.qos0_sent, 1);
        assert_eq!(handler.tracked_count().await, 0); // QoS 0 not tracked
    }

    #[tokio::test]
    async fn test_qos1_at_least_once() {
        let handler = QosHandler::new();
        let message = create_test_message("msg-002");

        handler
            .handle_message(message, QosLevel::AtLeastOnce)
            .await
            .unwrap();

        assert_eq!(handler.tracked_count().await, 1);

        // Acknowledge message
        handler.acknowledge_qos1("msg-002").await.unwrap();

        let stats = handler.get_stats().await;
        assert_eq!(stats.qos1_sent, 1);
        assert_eq!(stats.qos1_acked, 1);
        assert_eq!(handler.tracked_count().await, 0);
    }

    #[tokio::test]
    async fn test_qos2_exactly_once() {
        let handler = QosHandler::new();
        let message = create_test_message("msg-003");

        handler
            .handle_message(message, QosLevel::ExactlyOnce)
            .await
            .unwrap();

        assert_eq!(handler.tracked_count().await, 1);

        // Phase 1: PUBREC
        handler.acknowledge_qos2_pubrec("msg-003").await.unwrap();
        assert_eq!(handler.tracked_count().await, 1);

        // Phase 2: PUBCOMP
        handler.acknowledge_qos2_pubcomp("msg-003").await.unwrap();

        let stats = handler.get_stats().await;
        assert_eq!(stats.qos2_sent, 1);
        assert_eq!(stats.qos2_completed, 1);
        assert_eq!(handler.tracked_count().await, 0);
    }

    #[tokio::test]
    async fn test_qos_retry() {
        let handler = QosHandler::new();
        let message = create_test_message("msg-004");

        handler
            .handle_message(message, QosLevel::AtLeastOnce)
            .await
            .unwrap();

        // Retry message
        handler.retry_message("msg-004").await.unwrap();

        let stats = handler.get_stats().await;
        assert_eq!(stats.retries, 1);
    }

    #[tokio::test]
    async fn test_max_retries_exceeded() {
        let handler = QosHandler::new();
        let message = create_test_message("msg-005");

        handler
            .handle_message(message, QosLevel::AtLeastOnce)
            .await
            .unwrap();

        // Retry until max attempts
        handler.retry_message("msg-005").await.unwrap();
        handler.retry_message("msg-005").await.unwrap();

        // Should fail on 4th attempt (max is 3)
        let result = handler.retry_message("msg-005").await;
        assert!(result.is_err());

        let stats = handler.get_stats().await;
        assert_eq!(stats.failures, 1);
    }
}
