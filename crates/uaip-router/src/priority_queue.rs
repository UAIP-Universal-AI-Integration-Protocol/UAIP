//! Priority queue for message processing

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use tokio::sync::Mutex;

use uaip_core::message::{Priority, UaipMessage};

/// Priority wrapper for messages
#[derive(Debug, Clone)]
struct PriorityMessage {
    message: UaipMessage,
    priority: Priority,
    sequence: u64, // For FIFO within same priority
}

impl PartialEq for PriorityMessage {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.sequence == other.sequence
    }
}

impl Eq for PriorityMessage {}

impl PartialOrd for PriorityMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority comes first
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {
                // Within same priority, older messages (lower sequence) come first
                other.sequence.cmp(&self.sequence)
            }
            other => other,
        }
    }
}

/// Priority queue for messages
pub struct MessagePriorityQueue {
    heap: Mutex<BinaryHeap<PriorityMessage>>,
    sequence_counter: Mutex<u64>,
}

impl MessagePriorityQueue {
    /// Create a new priority queue
    pub fn new() -> Self {
        Self {
            heap: Mutex::new(BinaryHeap::new()),
            sequence_counter: Mutex::new(0),
        }
    }

    /// Push a message into the queue
    ///
    /// # Arguments
    /// * `message` - Message to enqueue
    pub async fn push(&self, message: UaipMessage) {
        let priority = message.header.priority.clone();

        // Get next sequence number
        let mut counter = self.sequence_counter.lock().await;
        let sequence = *counter;
        *counter += 1;
        drop(counter);

        let priority_msg = PriorityMessage {
            message,
            priority,
            sequence,
        };

        let mut heap = self.heap.lock().await;
        heap.push(priority_msg);
    }

    /// Pop the highest priority message
    ///
    /// # Returns
    /// * `Option<UaipMessage>` - Highest priority message or None if empty
    pub async fn pop(&self) -> Option<UaipMessage> {
        let mut heap = self.heap.lock().await;
        heap.pop().map(|pm| pm.message)
    }

    /// Peek at the highest priority message without removing it
    ///
    /// # Returns
    /// * `Option<UaipMessage>` - Highest priority message or None if empty
    pub async fn peek(&self) -> Option<UaipMessage> {
        let heap = self.heap.lock().await;
        heap.peek().map(|pm| pm.message.clone())
    }

    /// Get the number of messages in the queue
    pub async fn len(&self) -> usize {
        let heap = self.heap.lock().await;
        heap.len()
    }

    /// Check if the queue is empty
    pub async fn is_empty(&self) -> bool {
        let heap = self.heap.lock().await;
        heap.is_empty()
    }

    /// Clear all messages from the queue
    pub async fn clear(&self) {
        let mut heap = self.heap.lock().await;
        heap.clear();
    }

    /// Get queue statistics by priority
    pub async fn stats_by_priority(&self) -> PriorityStats {
        let heap = self.heap.lock().await;

        let mut critical = 0;
        let mut high = 0;
        let mut normal = 0;
        let mut low = 0;

        for msg in heap.iter() {
            match msg.priority {
                Priority::Critical => critical += 1,
                Priority::High => high += 1,
                Priority::Normal => normal += 1,
                Priority::Low => low += 1,
            }
        }

        PriorityStats {
            total: heap.len(),
            critical,
            high,
            normal,
            low,
        }
    }
}

impl Default for MessagePriorityQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Priority queue statistics
#[derive(Debug, Clone)]
pub struct PriorityStats {
    pub total: usize,
    pub critical: usize,
    pub high: usize,
    pub normal: usize,
    pub low: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uaip_core::message::{
        Action, AuthMethod, Authentication, Entity, EntityType, Header, Metadata, Payload,
        QosLevel, Security,
    };

    fn create_test_message(priority: Priority) -> UaipMessage {
        UaipMessage {
            header: Header {
                version: "1.0".to_string(),
                message_id: uuid::Uuid::new_v4().to_string(),
                correlation_id: None,
                timestamp: chrono::Utc::now(),
                ttl: 300000,
                priority,
                sender: Entity {
                    id: "test-sender".to_string(),
                    entity_type: EntityType::Device,
                },
                recipient: Entity {
                    id: "test-recipient".to_string(),
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
                qos: QosLevel::AtMostOnce,
                content_type: None,
                user_data: None,
            },
        }
    }

    #[tokio::test]
    async fn test_priority_queue_order() {
        let queue = MessagePriorityQueue::new();

        // Push messages in random order
        queue.push(create_test_message(Priority::Normal)).await;
        queue.push(create_test_message(Priority::Critical)).await;
        queue.push(create_test_message(Priority::Low)).await;
        queue.push(create_test_message(Priority::High)).await;

        // Pop should return in priority order
        assert_eq!(
            queue.pop().await.unwrap().header.priority,
            Priority::Critical
        );
        assert_eq!(queue.pop().await.unwrap().header.priority, Priority::High);
        assert_eq!(queue.pop().await.unwrap().header.priority, Priority::Normal);
        assert_eq!(queue.pop().await.unwrap().header.priority, Priority::Low);
        assert!(queue.pop().await.is_none());
    }

    #[tokio::test]
    async fn test_priority_queue_len() {
        let queue = MessagePriorityQueue::new();

        assert_eq!(queue.len().await, 0);
        assert!(queue.is_empty().await);

        queue.push(create_test_message(Priority::Normal)).await;
        assert_eq!(queue.len().await, 1);
        assert!(!queue.is_empty().await);

        queue.push(create_test_message(Priority::High)).await;
        assert_eq!(queue.len().await, 2);

        queue.pop().await;
        assert_eq!(queue.len().await, 1);

        queue.clear().await;
        assert_eq!(queue.len().await, 0);
        assert!(queue.is_empty().await);
    }

    #[tokio::test]
    async fn test_priority_stats() {
        let queue = MessagePriorityQueue::new();

        queue.push(create_test_message(Priority::Critical)).await;
        queue.push(create_test_message(Priority::Critical)).await;
        queue.push(create_test_message(Priority::High)).await;
        queue.push(create_test_message(Priority::Normal)).await;
        queue.push(create_test_message(Priority::Low)).await;

        let stats = queue.stats_by_priority().await;
        assert_eq!(stats.total, 5);
        assert_eq!(stats.critical, 2);
        assert_eq!(stats.high, 1);
        assert_eq!(stats.normal, 1);
        assert_eq!(stats.low, 1);
    }

    #[tokio::test]
    async fn test_peek() {
        let queue = MessagePriorityQueue::new();

        queue.push(create_test_message(Priority::Normal)).await;
        queue.push(create_test_message(Priority::Critical)).await;

        // Peek should return critical without removing
        let peeked = queue.peek().await.unwrap();
        assert_eq!(peeked.header.priority, Priority::Critical);
        assert_eq!(queue.len().await, 2);

        // Pop should still return critical
        let popped = queue.pop().await.unwrap();
        assert_eq!(popped.header.priority, Priority::Critical);
        assert_eq!(queue.len().await, 1);
    }
}
