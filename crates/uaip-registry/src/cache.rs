//! Redis caching layer for device states

use chrono::{DateTime, Utc};
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{Deserialize, Serialize};

use crate::models::{Device, DeviceStatus};
use uaip_core::error::{UaipError, UaipResult};

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Time-to-live for cached devices (seconds)
    pub device_ttl: u64,
    /// Time-to-live for device status (seconds)
    pub status_ttl: u64,
    /// Key prefix for cache entries
    pub key_prefix: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            device_ttl: 300, // 5 minutes
            status_ttl: 60,  // 1 minute
            key_prefix: "uaip:".to_string(),
        }
    }
}

/// Cached device state (lighter than full Device model)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDeviceState {
    pub device_id: String,
    pub status: DeviceStatus,
    pub last_seen: Option<DateTime<Utc>>,
    pub cached_at: DateTime<Utc>,
}

/// Redis cache service
pub struct CacheService {
    connection: ConnectionManager,
    config: CacheConfig,
}

impl CacheService {
    /// Create a new cache service
    ///
    /// # Arguments
    /// * `connection` - Redis connection manager
    /// * `config` - Cache configuration
    pub fn new(connection: ConnectionManager, config: CacheConfig) -> Self {
        Self { connection, config }
    }

    /// Cache a device
    ///
    /// # Arguments
    /// * `device` - Device to cache
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn cache_device(&mut self, device: &Device) -> UaipResult<()> {
        let key = format!("{}device:{}", self.config.key_prefix, device.device_id);
        let value = serde_json::to_string(device).map_err(UaipError::SerializationError)?;

        self.connection
            .set_ex::<_, _, ()>(&key, value, self.config.device_ttl)
            .await
            .map_err(|e| UaipError::DatabaseError(format!("Redis error: {}", e)))?;

        Ok(())
    }

    /// Get cached device
    ///
    /// # Arguments
    /// * `device_id` - Device identifier
    ///
    /// # Returns
    /// * `Result<Option<Device>>` - Cached device or None if not found
    pub async fn get_device(&mut self, device_id: &str) -> UaipResult<Option<Device>> {
        let key = format!("{}device:{}", self.config.key_prefix, device_id);

        let value: Option<String> = self
            .connection
            .get(&key)
            .await
            .map_err(|e| UaipError::DatabaseError(format!("Redis error: {}", e)))?;

        match value {
            Some(json) => {
                let device: Device =
                    serde_json::from_str(&json).map_err(UaipError::SerializationError)?;
                Ok(Some(device))
            }
            None => Ok(None),
        }
    }

    /// Cache device status
    ///
    /// # Arguments
    /// * `device_id` - Device identifier
    /// * `status` - Device status
    /// * `last_seen` - Last seen timestamp
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn cache_device_status(
        &mut self,
        device_id: &str,
        status: DeviceStatus,
        last_seen: Option<DateTime<Utc>>,
    ) -> UaipResult<()> {
        let key = format!("{}status:{}", self.config.key_prefix, device_id);

        let state = CachedDeviceState {
            device_id: device_id.to_string(),
            status,
            last_seen,
            cached_at: Utc::now(),
        };

        let value = serde_json::to_string(&state).map_err(UaipError::SerializationError)?;

        self.connection
            .set_ex::<_, _, ()>(&key, value, self.config.status_ttl)
            .await
            .map_err(|e| UaipError::DatabaseError(format!("Redis error: {}", e)))?;

        Ok(())
    }

    /// Get cached device status
    ///
    /// # Arguments
    /// * `device_id` - Device identifier
    ///
    /// # Returns
    /// * `Result<Option<CachedDeviceState>>` - Cached status or None
    pub async fn get_device_status(
        &mut self,
        device_id: &str,
    ) -> UaipResult<Option<CachedDeviceState>> {
        let key = format!("{}status:{}", self.config.key_prefix, device_id);

        let value: Option<String> = self
            .connection
            .get(&key)
            .await
            .map_err(|e| UaipError::DatabaseError(format!("Redis error: {}", e)))?;

        match value {
            Some(json) => {
                let state: CachedDeviceState =
                    serde_json::from_str(&json).map_err(UaipError::SerializationError)?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    /// Invalidate device cache
    ///
    /// # Arguments
    /// * `device_id` - Device identifier
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn invalidate_device(&mut self, device_id: &str) -> UaipResult<()> {
        let device_key = format!("{}device:{}", self.config.key_prefix, device_id);
        let status_key = format!("{}status:{}", self.config.key_prefix, device_id);

        self.connection
            .del::<_, ()>(&[device_key, status_key])
            .await
            .map_err(|e| UaipError::DatabaseError(format!("Redis error: {}", e)))?;

        Ok(())
    }

    /// Invalidate all device caches
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn invalidate_all(&mut self) -> UaipResult<()> {
        // Get all keys matching the prefix pattern
        let device_pattern = format!("{}device:*", self.config.key_prefix);
        let status_pattern = format!("{}status:*", self.config.key_prefix);

        // Delete all matching keys
        let device_keys: Vec<String> = self
            .connection
            .keys(&device_pattern)
            .await
            .map_err(|e| UaipError::DatabaseError(format!("Redis error: {}", e)))?;

        let status_keys: Vec<String> = self
            .connection
            .keys(&status_pattern)
            .await
            .map_err(|e| UaipError::DatabaseError(format!("Redis error: {}", e)))?;

        if !device_keys.is_empty() {
            self.connection
                .del::<_, ()>(&device_keys)
                .await
                .map_err(|e| UaipError::DatabaseError(format!("Redis error: {}", e)))?;
        }

        if !status_keys.is_empty() {
            self.connection
                .del::<_, ()>(&status_keys)
                .await
                .map_err(|e| UaipError::DatabaseError(format!("Redis error: {}", e)))?;
        }

        Ok(())
    }

    /// Check if device is cached
    ///
    /// # Arguments
    /// * `device_id` - Device identifier
    ///
    /// # Returns
    /// * `Result<bool>` - True if cached
    pub async fn is_device_cached(&mut self, device_id: &str) -> UaipResult<bool> {
        let key = format!("{}device:{}", self.config.key_prefix, device_id);

        let exists: bool = self
            .connection
            .exists(&key)
            .await
            .map_err(|e| UaipError::DatabaseError(format!("Redis error: {}", e)))?;

        Ok(exists)
    }

    /// Get cache statistics
    ///
    /// # Returns
    /// * `Result<CacheStats>` - Cache statistics
    pub async fn get_stats(&mut self) -> UaipResult<CacheStats> {
        let device_pattern = format!("{}device:*", self.config.key_prefix);
        let status_pattern = format!("{}status:*", self.config.key_prefix);

        let device_count: usize = self
            .connection
            .keys::<_, Vec<String>>(&device_pattern)
            .await
            .map_err(|e| UaipError::DatabaseError(format!("Redis error: {}", e)))?
            .len();

        let status_count: usize = self
            .connection
            .keys::<_, Vec<String>>(&status_pattern)
            .await
            .map_err(|e| UaipError::DatabaseError(format!("Redis error: {}", e)))?
            .len();

        Ok(CacheStats {
            cached_devices: device_count,
            cached_statuses: status_count,
        })
    }

    // ========================================================================
    // PHASE 4: Redis Pipelining for Performance Optimization
    // ========================================================================

    /// Cache multiple devices using Redis pipelining for better performance
    ///
    /// # Arguments
    /// * `devices` - Vector of devices to cache
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Performance
    /// Using pipelining reduces network round-trips from N to 1 for N devices
    pub async fn cache_devices_batch(&mut self, devices: &[Device]) -> UaipResult<()> {
        use redis::pipe;

        let mut pipeline = pipe();
        pipeline.atomic();

        for device in devices {
            let key = format!("{}device:{}", self.config.key_prefix, device.device_id);
            let value = serde_json::to_string(device).map_err(UaipError::SerializationError)?;
            pipeline.set_ex(&key, value, self.config.device_ttl);
        }

        pipeline
            .query_async::<_, ()>(&mut self.connection)
            .await
            .map_err(|e| UaipError::DatabaseError(format!("Redis pipeline error: {}", e)))?;

        Ok(())
    }

    /// Get multiple devices using Redis pipelining
    ///
    /// # Arguments
    /// * `device_ids` - Vector of device identifiers
    ///
    /// # Returns
    /// * `Result<Vec<Option<Device>>>` - Vector of cached devices (None if not found)
    ///
    /// # Performance
    /// Using pipelining reduces network round-trips from N to 1 for N device IDs
    pub async fn get_devices_batch(&mut self, device_ids: &[String]) -> UaipResult<Vec<Option<Device>>> {
        use redis::pipe;

        let mut pipeline = pipe();

        for device_id in device_ids {
            let key = format!("{}device:{}", self.config.key_prefix, device_id);
            pipeline.get(&key);
        }

        let results: Vec<Option<String>> = pipeline
            .query_async(&mut self.connection)
            .await
            .map_err(|e| UaipError::DatabaseError(format!("Redis pipeline error: {}", e)))?;

        let mut devices = Vec::with_capacity(results.len());

        for result in results {
            match result {
                Some(json) => {
                    let device: Device =
                        serde_json::from_str(&json).map_err(UaipError::SerializationError)?;
                    devices.push(Some(device));
                }
                None => devices.push(None),
            }
        }

        Ok(devices)
    }

    /// Cache multiple device statuses using Redis pipelining
    ///
    /// # Arguments
    /// * `statuses` - Vector of (device_id, status, last_seen) tuples
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn cache_device_statuses_batch(
        &mut self,
        statuses: &[(String, DeviceStatus, Option<DateTime<Utc>>)],
    ) -> UaipResult<()> {
        use redis::pipe;

        let mut pipeline = pipe();
        pipeline.atomic();

        for (device_id, status, last_seen) in statuses {
            let key = format!("{}status:{}", self.config.key_prefix, device_id);

            let state = CachedDeviceState {
                device_id: device_id.clone(),
                status: *status,
                last_seen: *last_seen,
                cached_at: Utc::now(),
            };

            let value = serde_json::to_string(&state).map_err(UaipError::SerializationError)?;
            pipeline.set_ex(&key, value, self.config.status_ttl);
        }

        pipeline
            .query_async::<_, ()>(&mut self.connection)
            .await
            .map_err(|e| UaipError::DatabaseError(format!("Redis pipeline error: {}", e)))?;

        Ok(())
    }

    /// Get multiple device statuses using Redis pipelining
    ///
    /// # Arguments
    /// * `device_ids` - Vector of device identifiers
    ///
    /// # Returns
    /// * `Result<Vec<Option<CachedDeviceState>>>` - Vector of cached statuses
    pub async fn get_device_statuses_batch(
        &mut self,
        device_ids: &[String],
    ) -> UaipResult<Vec<Option<CachedDeviceState>>> {
        use redis::pipe;

        let mut pipeline = pipe();

        for device_id in device_ids {
            let key = format!("{}status:{}", self.config.key_prefix, device_id);
            pipeline.get(&key);
        }

        let results: Vec<Option<String>> = pipeline
            .query_async(&mut self.connection)
            .await
            .map_err(|e| UaipError::DatabaseError(format!("Redis pipeline error: {}", e)))?;

        let mut statuses = Vec::with_capacity(results.len());

        for result in results {
            match result {
                Some(json) => {
                    let state: CachedDeviceState =
                        serde_json::from_str(&json).map_err(UaipError::SerializationError)?;
                    statuses.push(Some(state));
                }
                None => statuses.push(None),
            }
        }

        Ok(statuses)
    }

    /// Invalidate multiple devices using Redis pipelining
    ///
    /// # Arguments
    /// * `device_ids` - Vector of device identifiers
    ///
    /// # Returns
    /// * `Result<usize>` - Number of keys deleted
    pub async fn invalidate_devices_batch(&mut self, device_ids: &[String]) -> UaipResult<usize> {
        use redis::pipe;

        let mut keys_to_delete = Vec::with_capacity(device_ids.len() * 2);

        for device_id in device_ids {
            keys_to_delete.push(format!("{}device:{}", self.config.key_prefix, device_id));
            keys_to_delete.push(format!("{}status:{}", self.config.key_prefix, device_id));
        }

        let deleted: usize = self
            .connection
            .del(&keys_to_delete)
            .await
            .map_err(|e| UaipError::DatabaseError(format!("Redis error: {}", e)))?;

        Ok(deleted)
    }

    /// Check if multiple devices are cached using Redis pipelining
    ///
    /// # Arguments
    /// * `device_ids` - Vector of device identifiers
    ///
    /// # Returns
    /// * `Result<Vec<bool>>` - Vector indicating if each device is cached
    pub async fn are_devices_cached(&mut self, device_ids: &[String]) -> UaipResult<Vec<bool>> {
        use redis::pipe;

        let mut pipeline = pipe();

        for device_id in device_ids {
            let key = format!("{}device:{}", self.config.key_prefix, device_id);
            pipeline.exists(&key);
        }

        let results: Vec<bool> = pipeline
            .query_async(&mut self.connection)
            .await
            .map_err(|e| UaipError::DatabaseError(format!("Redis pipeline error: {}", e)))?;

        Ok(results)
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub cached_devices: usize,
    pub cached_statuses: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.device_ttl, 300);
        assert_eq!(config.status_ttl, 60);
        assert_eq!(config.key_prefix, "uaip:");
    }

    #[test]
    fn test_cache_config_custom() {
        let config = CacheConfig {
            device_ttl: 600,
            status_ttl: 120,
            key_prefix: "test:".to_string(),
        };

        assert_eq!(config.device_ttl, 600);
        assert_eq!(config.status_ttl, 120);
        assert_eq!(config.key_prefix, "test:");
    }

    #[test]
    fn test_cached_device_state() {
        let state = CachedDeviceState {
            device_id: "device-123".to_string(),
            status: DeviceStatus::Online,
            last_seen: Some(Utc::now()),
            cached_at: Utc::now(),
        };

        assert_eq!(state.device_id, "device-123");
        assert_eq!(state.status, DeviceStatus::Online);
    }

    #[test]
    fn test_cache_stats() {
        let stats = CacheStats {
            cached_devices: 100,
            cached_statuses: 150,
        };

        assert_eq!(stats.cached_devices, 100);
        assert_eq!(stats.cached_statuses, 150);
    }
}
