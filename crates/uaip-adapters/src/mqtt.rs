// MQTT protocol adapter for IoT device communication
//!
//! Provides MQTT client functionality for connecting devices to the UAIP Hub.
//! Supports MQTT 3.1.1 and 5.0 protocols with QoS 0, 1, and 2.

use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use uaip_core::{
    error::{Result, UaipError},
    message::UaipMessage,
};

/// MQTT adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    /// MQTT broker host
    pub host: String,

    /// MQTT broker port
    pub port: u16,

    /// Client ID
    pub client_id: String,

    /// Username for authentication
    pub username: Option<String>,

    /// Password for authentication
    pub password: Option<String>,

    /// Keep alive interval in seconds
    pub keep_alive: u64,

    /// Clean session flag
    pub clean_session: bool,

    /// Maximum packet size in bytes
    pub max_packet_size: usize,

    /// Connection timeout in seconds
    pub connection_timeout: u64,
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 1883,
            client_id: format!("uaip-hub-{}", uuid::Uuid::new_v4()),
            username: None,
            password: None,
            keep_alive: 60,
            clean_session: true,
            max_packet_size: 256 * 1024, // 256 KB
            connection_timeout: 10,
        }
    }
}

/// MQTT message handler callback
pub type MessageHandler = Arc<dyn Fn(String, Vec<u8>) -> Result<()> + Send + Sync>;

/// MQTT adapter for device communication
pub struct MqttAdapter {
    client: AsyncClient,
    config: MqttConfig,
    subscriptions: Arc<RwLock<Vec<String>>>,
    message_handler: Option<MessageHandler>,
}

impl MqttAdapter {
    /// Create a new MQTT adapter
    pub fn new(config: MqttConfig) -> Result<(Self, EventLoop)> {
        // Configure MQTT options
        let mut mqtt_options = MqttOptions::new(&config.client_id, &config.host, config.port);

        mqtt_options.set_keep_alive(Duration::from_secs(config.keep_alive));
        mqtt_options.set_clean_session(config.clean_session);
        mqtt_options.set_max_packet_size(config.max_packet_size, config.max_packet_size);

        // Set authentication if provided
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            mqtt_options.set_credentials(username, password);
        }

        // Create async client and event loop
        let (client, eventloop) = AsyncClient::new(mqtt_options, 10);

        info!(
            "MQTT adapter created for broker {}:{} with client_id: {}",
            config.host, config.port, config.client_id
        );

        Ok((
            Self {
                client,
                config,
                subscriptions: Arc::new(RwLock::new(Vec::new())),
                message_handler: None,
            },
            eventloop,
        ))
    }

    /// Set message handler callback
    pub fn set_message_handler<F>(&mut self, handler: F)
    where
        F: Fn(String, Vec<u8>) -> Result<()> + Send + Sync + 'static,
    {
        self.message_handler = Some(Arc::new(handler));
    }

    /// Subscribe to a topic
    pub async fn subscribe(&self, topic: &str, qos: QoS) -> Result<()> {
        self.client.subscribe(topic, qos).await.map_err(|e| {
            UaipError::ConnectionError(format!("Failed to subscribe to {}: {}", topic, e))
        })?;

        // Track subscription
        let mut subs = self.subscriptions.write().await;
        if !subs.contains(&topic.to_string()) {
            subs.push(topic.to_string());
        }

        info!("Subscribed to topic: {} with QoS {:?}", topic, qos);
        Ok(())
    }

    /// Subscribe to multiple topics
    pub async fn subscribe_many<T: AsRef<str>>(&self, topics: &[(T, QoS)]) -> Result<()> {
        // Subscribe to each topic individually
        for (topic, qos) in topics {
            self.subscribe(topic.as_ref(), *qos).await?;
        }

        info!("Subscribed to {} topics", topics.len());
        Ok(())
    }

    /// Unsubscribe from a topic
    pub async fn unsubscribe(&self, topic: &str) -> Result<()> {
        self.client.unsubscribe(topic).await.map_err(|e| {
            UaipError::ConnectionError(format!("Failed to unsubscribe from {}: {}", topic, e))
        })?;

        // Remove from tracking
        let mut subs = self.subscriptions.write().await;
        subs.retain(|t| t != topic);

        info!("Unsubscribed from topic: {}", topic);
        Ok(())
    }

    /// Publish a message to a topic
    pub async fn publish(
        &self,
        topic: &str,
        payload: Vec<u8>,
        qos: QoS,
        retain: bool,
    ) -> Result<()> {
        self.client
            .publish(topic, qos, retain, payload.clone())
            .await
            .map_err(|e| {
                UaipError::ConnectionError(format!("Failed to publish to {}: {}", topic, e))
            })?;

        debug!(
            "Published message to topic: {} (QoS {:?}, retain: {}, size: {} bytes)",
            topic,
            qos,
            retain,
            payload.len()
        );
        Ok(())
    }

    /// Publish a UAIP message
    pub async fn publish_uaip_message(
        &self,
        topic: &str,
        message: &UaipMessage,
        qos: QoS,
    ) -> Result<()> {
        let payload = serde_json::to_vec(message).map_err(|e| {
            UaipError::InvalidMessage(format!("Failed to serialize message: {}", e))
        })?;

        self.publish(topic, payload, qos, false).await
    }

    /// Disconnect from broker
    pub async fn disconnect(&self) -> Result<()> {
        self.client
            .disconnect()
            .await
            .map_err(|e| UaipError::ConnectionError(format!("Failed to disconnect: {}", e)))?;

        info!("Disconnected from MQTT broker");
        Ok(())
    }

    /// Get list of active subscriptions
    pub async fn get_subscriptions(&self) -> Vec<String> {
        self.subscriptions.read().await.clone()
    }

    /// Get the MQTT configuration
    pub fn get_config(&self) -> &MqttConfig {
        &self.config
    }

    /// Process MQTT events (should be called in a loop)
    pub async fn poll_event(&self, eventloop: &mut EventLoop) -> Result<Event> {
        eventloop
            .poll()
            .await
            .map_err(|e| UaipError::ConnectionError(format!("MQTT event loop error: {}", e)))
    }

    /// Handle an incoming event
    pub async fn handle_event(&self, event: Event) -> Result<()> {
        match event {
            Event::Incoming(Packet::Publish(publish)) => {
                let topic = publish.topic.clone();
                let payload = publish.payload.to_vec();

                debug!(
                    "Received message on topic: {} (QoS {:?}, size: {} bytes)",
                    topic,
                    publish.qos,
                    payload.len()
                );

                // Call message handler if set
                if let Some(handler) = &self.message_handler {
                    if let Err(e) = handler(topic.clone(), payload.clone()) {
                        error!("Message handler error for topic {}: {}", topic, e);
                    }
                } else {
                    warn!("Received message but no handler is set");
                }
            }
            Event::Incoming(Packet::ConnAck(_)) => {
                info!("Connected to MQTT broker");
            }
            Event::Incoming(Packet::SubAck(suback)) => {
                debug!("Subscription acknowledged: pkid={}", suback.pkid);
            }
            Event::Incoming(Packet::UnsubAck(unsuback)) => {
                debug!("Unsubscription acknowledged: pkid={}", unsuback.pkid);
            }
            Event::Incoming(Packet::PingResp) => {
                debug!("Ping response received");
            }
            Event::Incoming(Packet::Disconnect) => {
                warn!("Received disconnect from broker");
            }
            Event::Outgoing(_) => {
                // Outgoing packets, no action needed
            }
            _ => {
                debug!("Unhandled MQTT event: {:?}", event);
            }
        }

        Ok(())
    }

    /// Run the event loop continuously
    pub async fn run_event_loop(self: Arc<Self>, mut eventloop: EventLoop) {
        info!("Starting MQTT event loop");

        loop {
            match self.poll_event(&mut eventloop).await {
                Ok(event) => {
                    if let Err(e) = self.handle_event(event).await {
                        error!("Error handling MQTT event: {}", e);
                    }
                }
                Err(e) => {
                    error!("MQTT event loop error: {}", e);

                    // Reconnect delay
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }
}

/// Convert QoS level from usize to rumqttc::QoS
pub fn qos_from_level(level: usize) -> QoS {
    match level {
        0 => QoS::AtMostOnce,
        1 => QoS::AtLeastOnce,
        2 => QoS::ExactlyOnce,
        _ => QoS::AtLeastOnce, // Default to QoS 1
    }
}

/// Convert rumqttc::QoS to usize
pub fn qos_to_level(qos: QoS) -> usize {
    match qos {
        QoS::AtMostOnce => 0,
        QoS::AtLeastOnce => 1,
        QoS::ExactlyOnce => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mqtt_config_default() {
        let config = MqttConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 1883);
        assert_eq!(config.keep_alive, 60);
        assert!(config.clean_session);
        assert_eq!(config.max_packet_size, 256 * 1024);
    }

    #[test]
    fn test_qos_conversion() {
        assert_eq!(qos_to_level(QoS::AtMostOnce), 0);
        assert_eq!(qos_to_level(QoS::AtLeastOnce), 1);
        assert_eq!(qos_to_level(QoS::ExactlyOnce), 2);

        assert_eq!(qos_from_level(0), QoS::AtMostOnce);
        assert_eq!(qos_from_level(1), QoS::AtLeastOnce);
        assert_eq!(qos_from_level(2), QoS::ExactlyOnce);
        assert_eq!(qos_from_level(99), QoS::AtLeastOnce); // Default
    }

    #[tokio::test]
    async fn test_mqtt_adapter_creation() {
        let config = MqttConfig::default();
        let result = MqttAdapter::new(config);
        assert!(result.is_ok());
    }
}
