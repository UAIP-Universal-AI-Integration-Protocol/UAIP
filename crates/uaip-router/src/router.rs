//! Message routing service

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use uaip_core::error::UaipResult;
use uaip_core::message::UaipMessage;

use crate::priority_queue::MessagePriorityQueue;
use crate::qos::{QosHandler, QosLevel};

/// Route entry for a recipient
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct RouteEntry {
    /// Recipient identifier
    recipient_id: String,
    /// Active connection (simulated for now)
    connected: bool,
}

/// Message router service
pub struct MessageRouter {
    /// Priority queue for pending messages
    queue: Arc<MessagePriorityQueue>,
    /// QoS handler
    qos_handler: Arc<QosHandler>,
    /// Routing table (recipient_id -> RouteEntry)
    routes: Arc<RwLock<HashMap<String, RouteEntry>>>,
    /// Message delivery statistics
    stats: Arc<RwLock<RouterStats>>,
}

/// Router statistics
#[derive(Debug, Clone, Default)]
pub struct RouterStats {
    pub messages_routed: u64,
    pub messages_queued: u64,
    pub messages_failed: u64,
    pub messages_delivered: u64,
}

impl MessageRouter {
    /// Create a new message router
    pub fn new(queue: Arc<MessagePriorityQueue>, qos_handler: Arc<QosHandler>) -> Self {
        Self {
            queue,
            qos_handler,
            routes: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(RouterStats::default())),
        }
    }

    /// Register a recipient route
    ///
    /// # Arguments
    /// * `recipient_id` - Recipient identifier
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn register_route(&self, recipient_id: String) -> UaipResult<()> {
        let route = RouteEntry {
            recipient_id: recipient_id.clone(),
            connected: true,
        };

        let mut routes = self.routes.write().await;
        routes.insert(recipient_id, route);

        Ok(())
    }

    /// Unregister a recipient route
    ///
    /// # Arguments
    /// * `recipient_id` - Recipient identifier
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn unregister_route(&self, recipient_id: &str) -> UaipResult<()> {
        let mut routes = self.routes.write().await;
        routes.remove(recipient_id);

        Ok(())
    }

    /// Check if a recipient route exists
    ///
    /// # Arguments
    /// * `recipient_id` - Recipient identifier
    ///
    /// # Returns
    /// * `bool` - True if route exists
    pub async fn has_route(&self, recipient_id: &str) -> bool {
        let routes = self.routes.read().await;
        routes.contains_key(recipient_id)
    }

    /// Route a message
    ///
    /// # Arguments
    /// * `message` - Message to route
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn route_message(&self, message: UaipMessage) -> UaipResult<()> {
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.messages_routed += 1;
        }

        // Check if recipient route exists
        let recipient_id = &message.header.recipient.id;
        let route_exists = self.has_route(recipient_id).await;

        if !route_exists {
            // Queue message for later delivery
            self.queue.push(message.clone()).await;

            let mut stats = self.stats.write().await;
            stats.messages_queued += 1;

            return Ok(());
        }

        // Deliver message based on QoS level
        let qos_level = match message.metadata.qos {
            uaip_core::message::QosLevel::AtMostOnce => QosLevel::AtMostOnce,
            uaip_core::message::QosLevel::AtLeastOnce => QosLevel::AtLeastOnce,
            uaip_core::message::QosLevel::ExactlyOnce => QosLevel::ExactlyOnce,
        };

        match self
            .qos_handler
            .handle_message(message.clone(), qos_level)
            .await
        {
            Ok(_) => {
                let mut stats = self.stats.write().await;
                stats.messages_delivered += 1;
                Ok(())
            }
            Err(e) => {
                // Queue message for retry
                self.queue.push(message).await;

                let mut stats = self.stats.write().await;
                stats.messages_failed += 1;
                stats.messages_queued += 1;

                Err(e)
            }
        }
    }

    /// Process queued messages
    ///
    /// Attempts to deliver messages from the priority queue
    ///
    /// # Returns
    /// * `Result<usize>` - Number of messages processed
    pub async fn process_queue(&self) -> UaipResult<usize> {
        let mut processed = 0;

        while let Some(message) = self.queue.pop().await {
            // Check if recipient is now available
            let recipient_id = &message.header.recipient.id;
            if !self.has_route(recipient_id).await {
                // Re-queue if still not available
                self.queue.push(message).await;
                break;
            }

            // Try to deliver
            match self.route_message(message).await {
                Ok(_) => processed += 1,
                Err(_) => break, // Will be re-queued by route_message
            }
        }

        Ok(processed)
    }

    /// Get router statistics
    ///
    /// # Returns
    /// * `RouterStats` - Current statistics
    pub async fn get_stats(&self) -> RouterStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Get number of queued messages
    ///
    /// # Returns
    /// * `usize` - Number of queued messages
    pub async fn queue_size(&self) -> usize {
        self.queue.len().await
    }

    /// Get number of registered routes
    ///
    /// # Returns
    /// * `usize` - Number of routes
    pub async fn route_count(&self) -> usize {
        let routes = self.routes.read().await;
        routes.len()
    }

    /// Clear all queued messages
    pub async fn clear_queue(&self) {
        self.queue.clear().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uaip_core::message::{
        Action, AuthMethod, Authentication, Entity, EntityType, Header, Metadata, Payload,
        Priority, Security,
    };

    fn create_test_message(sender_id: &str, recipient_id: &str, priority: Priority) -> UaipMessage {
        UaipMessage {
            header: Header {
                version: "1.0".to_string(),
                message_id: uuid::Uuid::new_v4().to_string(),
                correlation_id: None,
                timestamp: chrono::Utc::now(),
                ttl: 300000,
                priority,
                sender: Entity {
                    id: sender_id.to_string(),
                    entity_type: EntityType::Device,
                },
                recipient: Entity {
                    id: recipient_id.to_string(),
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
    async fn test_router_creation() {
        let queue = Arc::new(MessagePriorityQueue::new());
        let qos_handler = Arc::new(QosHandler::new());
        let router = MessageRouter::new(queue, qos_handler);

        assert_eq!(router.route_count().await, 0);
        assert_eq!(router.queue_size().await, 0);
    }

    #[tokio::test]
    async fn test_route_registration() {
        let queue = Arc::new(MessagePriorityQueue::new());
        let qos_handler = Arc::new(QosHandler::new());
        let router = MessageRouter::new(queue, qos_handler);

        router
            .register_route("recipient-1".to_string())
            .await
            .unwrap();
        assert_eq!(router.route_count().await, 1);
        assert!(router.has_route("recipient-1").await);

        router.unregister_route("recipient-1").await.unwrap();
        assert_eq!(router.route_count().await, 0);
        assert!(!router.has_route("recipient-1").await);
    }

    #[tokio::test]
    async fn test_message_queuing_when_no_route() {
        let queue = Arc::new(MessagePriorityQueue::new());
        let qos_handler = Arc::new(QosHandler::new());
        let router = MessageRouter::new(queue, qos_handler);

        let message = create_test_message("sender-1", "recipient-1", Priority::Normal);

        // Route message without registered route
        router.route_message(message).await.unwrap();

        // Message should be queued
        assert_eq!(router.queue_size().await, 1);

        let stats = router.get_stats().await;
        assert_eq!(stats.messages_routed, 1);
        assert_eq!(stats.messages_queued, 1);
    }

    #[tokio::test]
    async fn test_router_stats() {
        let queue = Arc::new(MessagePriorityQueue::new());
        let qos_handler = Arc::new(QosHandler::new());
        let router = MessageRouter::new(queue, qos_handler);

        let message = create_test_message("sender-1", "recipient-1", Priority::High);

        router.route_message(message).await.unwrap();

        let stats = router.get_stats().await;
        assert_eq!(stats.messages_routed, 1);
    }
}
