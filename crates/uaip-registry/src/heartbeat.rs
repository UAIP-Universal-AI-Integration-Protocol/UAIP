//! Device heartbeat and status tracking

use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::time::interval;

use crate::models::DeviceStatus;
use crate::repository::DeviceRepository;
use uaip_core::error::UaipResult;

/// Heartbeat configuration
#[derive(Debug, Clone)]
pub struct HeartbeatConfig {
    /// How often devices should send heartbeats (seconds)
    pub heartbeat_interval: i64,
    /// Grace period before marking device offline (seconds)
    pub timeout_grace_period: i64,
    /// How often to check for stale devices (seconds)
    pub check_interval: i64,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: 30,      // Devices send heartbeat every 30s
            timeout_grace_period: 60,     // 60s grace period
            check_interval: 15,           // Check every 15s
        }
    }
}

/// Device heartbeat information
#[derive(Debug, Clone)]
struct HeartbeatInfo {
    last_heartbeat: DateTime<Utc>,
    status: DeviceStatus,
    consecutive_failures: u32,
}

/// Heartbeat service for tracking device status
pub struct HeartbeatService {
    repository: DeviceRepository,
    config: HeartbeatConfig,
    heartbeats: RwLock<HashMap<String, HeartbeatInfo>>,
}

impl HeartbeatService {
    /// Create a new heartbeat service
    ///
    /// # Arguments
    /// * `repository` - Device repository
    /// * `config` - Heartbeat configuration
    pub fn new(repository: DeviceRepository, config: HeartbeatConfig) -> Self {
        Self {
            repository,
            config,
            heartbeats: RwLock::new(HashMap::new()),
        }
    }

    /// Record a heartbeat from a device
    ///
    /// # Arguments
    /// * `device_id` - Device identifier
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn record_heartbeat(&self, device_id: &str) -> UaipResult<()> {
        let now = Utc::now();

        // Update in-memory heartbeat tracking
        {
            let mut heartbeats = self.heartbeats.write().await;
            heartbeats.insert(
                device_id.to_string(),
                HeartbeatInfo {
                    last_heartbeat: now,
                    status: DeviceStatus::Online,
                    consecutive_failures: 0,
                },
            );
        }

        // Update database
        self.repository.update_last_seen(device_id, now).await?;

        // Ensure device is marked as online
        let device = self.repository.get_device_by_device_id(device_id).await?;
        if device.status != DeviceStatus::Online {
            self.repository
                .update_status(device_id, DeviceStatus::Online)
                .await?;
        }

        Ok(())
    }

    /// Get device status based on heartbeat
    ///
    /// # Arguments
    /// * `device_id` - Device identifier
    ///
    /// # Returns
    /// * `Option<DeviceStatus>` - Current status or None if not tracked
    pub async fn get_device_status(&self, device_id: &str) -> Option<DeviceStatus> {
        let heartbeats = self.heartbeats.read().await;
        heartbeats.get(device_id).map(|info| info.status.clone())
    }

    /// Get time since last heartbeat
    ///
    /// # Arguments
    /// * `device_id` - Device identifier
    ///
    /// # Returns
    /// * `Option<Duration>` - Time since last heartbeat or None
    pub async fn time_since_last_heartbeat(&self, device_id: &str) -> Option<Duration> {
        let heartbeats = self.heartbeats.read().await;
        heartbeats
            .get(device_id)
            .map(|info| Utc::now() - info.last_heartbeat)
    }

    /// Check for stale devices and update their status
    ///
    /// This should be called periodically to detect offline devices
    ///
    /// # Returns
    /// * `Result<usize>` - Number of devices marked as offline
    pub async fn check_stale_devices(&self) -> UaipResult<usize> {
        let now = Utc::now();
        let timeout_threshold = now - Duration::seconds(
            self.config.heartbeat_interval + self.config.timeout_grace_period,
        );

        let mut offline_count = 0;
        let mut devices_to_update = Vec::new();

        // Check in-memory heartbeats
        {
            let mut heartbeats = self.heartbeats.write().await;

            for (device_id, info) in heartbeats.iter_mut() {
                if info.last_heartbeat < timeout_threshold && info.status != DeviceStatus::Offline {
                    info.status = DeviceStatus::Offline;
                    info.consecutive_failures += 1;
                    devices_to_update.push(device_id.clone());
                    offline_count += 1;
                }
            }
        }

        // Update database for offline devices
        for device_id in devices_to_update {
            if let Err(e) = self
                .repository
                .update_status(&device_id, DeviceStatus::Offline)
                .await
            {
                tracing::warn!(
                    "Failed to update status for device {}: {}",
                    device_id,
                    e
                );
            }
        }

        Ok(offline_count)
    }

    /// Start the heartbeat monitoring task
    ///
    /// This spawns a background task that periodically checks for stale devices
    ///
    /// # Returns
    /// * `tokio::task::JoinHandle` - Handle to the background task
    pub fn start_monitoring(self: std::sync::Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut check_interval = interval(std::time::Duration::from_secs(
                self.config.check_interval as u64,
            ));

            loop {
                check_interval.tick().await;

                match self.check_stale_devices().await {
                    Ok(count) => {
                        if count > 0 {
                            tracing::info!("Marked {} devices as offline", count);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error checking stale devices: {}", e);
                    }
                }
            }
        })
    }

    /// Initialize heartbeat tracking from database
    ///
    /// Loads all devices from database and initializes heartbeat tracking
    ///
    /// # Returns
    /// * `Result<usize>` - Number of devices initialized
    pub async fn initialize_from_database(&self) -> UaipResult<usize> {
        use crate::models::DeviceFilter;

        // Load all devices from database
        let devices = self
            .repository
            .list_devices(DeviceFilter::default())
            .await?;

        let mut heartbeats = self.heartbeats.write().await;

        for device in &devices {
            let last_heartbeat = device.last_seen.unwrap_or(device.registered_at);

            heartbeats.insert(
                device.device_id.clone(),
                HeartbeatInfo {
                    last_heartbeat,
                    status: device.status.clone(),
                    consecutive_failures: 0,
                },
            );
        }

        let count = devices.len();
        tracing::info!("Initialized heartbeat tracking for {} devices", count);

        Ok(count)
    }

    /// Remove device from heartbeat tracking
    ///
    /// # Arguments
    /// * `device_id` - Device identifier
    pub async fn remove_device(&self, device_id: &str) {
        let mut heartbeats = self.heartbeats.write().await;
        heartbeats.remove(device_id);
    }

    /// Get count of tracked devices
    pub async fn tracked_devices_count(&self) -> usize {
        let heartbeats = self.heartbeats.read().await;
        heartbeats.len()
    }

    /// Get statistics about device heartbeats
    ///
    /// # Returns
    /// * `HeartbeatStats` - Statistics
    pub async fn get_stats(&self) -> HeartbeatStats {
        let heartbeats = self.heartbeats.read().await;

        let mut online = 0;
        let mut offline = 0;
        let mut other = 0;

        for info in heartbeats.values() {
            match info.status {
                DeviceStatus::Online => online += 1,
                DeviceStatus::Offline => offline += 1,
                _ => other += 1,
            }
        }

        HeartbeatStats {
            total_devices: heartbeats.len(),
            online_devices: online,
            offline_devices: offline,
            other_devices: other,
        }
    }
}

/// Heartbeat statistics
#[derive(Debug, Clone)]
pub struct HeartbeatStats {
    pub total_devices: usize,
    pub online_devices: usize,
    pub offline_devices: usize,
    pub other_devices: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_config_default() {
        let config = HeartbeatConfig::default();
        assert_eq!(config.heartbeat_interval, 30);
        assert_eq!(config.timeout_grace_period, 60);
        assert_eq!(config.check_interval, 15);
    }

    #[test]
    fn test_heartbeat_stats() {
        let stats = HeartbeatStats {
            total_devices: 100,
            online_devices: 80,
            offline_devices: 15,
            other_devices: 5,
        };

        assert_eq!(stats.total_devices, 100);
        assert_eq!(stats.online_devices, 80);
        assert_eq!(
            stats.online_devices + stats.offline_devices + stats.other_devices,
            100
        );
    }

    #[test]
    fn test_heartbeat_config_custom() {
        let config = HeartbeatConfig {
            heartbeat_interval: 60,
            timeout_grace_period: 120,
            check_interval: 30,
        };

        assert_eq!(config.heartbeat_interval, 60);
        assert_eq!(config.timeout_grace_period, 120);
    }
}
