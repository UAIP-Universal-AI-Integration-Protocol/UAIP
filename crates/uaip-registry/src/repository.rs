//! Device repository (data access layer)

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{CreateDevice, Device, DeviceFilter, DeviceStatus, UpdateDevice};
use uaip_core::error::{UaipError, UaipResult};

/// Device repository for PostgreSQL operations
#[derive(Clone)]
pub struct DeviceRepository {
    pool: PgPool,
}

impl DeviceRepository {
    /// Create a new device repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new device in the database
    ///
    /// # Arguments
    /// * `create` - Device creation data
    ///
    /// # Returns
    /// * `Result<Device>` - The created device or an error
    pub async fn create_device(&self, create: CreateDevice) -> UaipResult<Device> {
        let metadata = create.metadata.unwrap_or_else(|| serde_json::json!({}));

        let device = sqlx::query_as::<_, Device>(
            r#"
            INSERT INTO devices (
                device_id, mac_address, manufacturer, model,
                firmware_version, status, capabilities, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(&create.device_id)
        .bind(&create.mac_address)
        .bind(&create.manufacturer)
        .bind(&create.model)
        .bind(&create.firmware_version)
        .bind(DeviceStatus::Offline) // New devices start as offline
        .bind(&create.capabilities)
        .bind(&metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return UaipError::DeviceAlreadyRegistered(create.device_id.clone());
                }
            }
            UaipError::DatabaseError(e.to_string())
        })?;

        Ok(device)
    }

    /// Get a device by its UUID
    ///
    /// # Arguments
    /// * `id` - Device UUID
    ///
    /// # Returns
    /// * `Result<Device>` - The device or an error
    pub async fn get_device_by_id(&self, id: Uuid) -> UaipResult<Device> {
        let device = sqlx::query_as::<_, Device>(
            r#"
            SELECT * FROM devices WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UaipError::DatabaseError(e.to_string()))?
        .ok_or_else(|| UaipError::DeviceNotFound(id.to_string()))?;

        Ok(device)
    }

    /// Get a device by its device_id
    ///
    /// # Arguments
    /// * `device_id` - Device identifier
    ///
    /// # Returns
    /// * `Result<Device>` - The device or an error
    pub async fn get_device_by_device_id(&self, device_id: &str) -> UaipResult<Device> {
        let device = sqlx::query_as::<_, Device>(
            r#"
            SELECT * FROM devices WHERE device_id = $1
            "#,
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UaipError::DatabaseError(e.to_string()))?
        .ok_or_else(|| UaipError::DeviceNotFound(device_id.to_string()))?;

        Ok(device)
    }

    /// Get a device by MAC address
    ///
    /// # Arguments
    /// * `mac_address` - Device MAC address
    ///
    /// # Returns
    /// * `Result<Device>` - The device or an error
    pub async fn get_device_by_mac(&self, mac_address: &str) -> UaipResult<Device> {
        let device = sqlx::query_as::<_, Device>(
            r#"
            SELECT * FROM devices WHERE mac_address = $1
            "#,
        )
        .bind(mac_address)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UaipError::DatabaseError(e.to_string()))?
        .ok_or_else(|| UaipError::DeviceNotFound(mac_address.to_string()))?;

        Ok(device)
    }

    /// Update a device
    ///
    /// # Arguments
    /// * `id` - Device UUID
    /// * `update` - Update data
    ///
    /// # Returns
    /// * `Result<Device>` - The updated device or an error
    pub async fn update_device(&self, id: Uuid, update: UpdateDevice) -> UaipResult<Device> {
        // Build dynamic update query
        let mut query = String::from("UPDATE devices SET ");
        let mut bindings = Vec::new();
        let mut param_count = 1;

        if let Some(firmware_version) = &update.firmware_version {
            query.push_str(&format!("firmware_version = ${}, ", param_count));
            bindings.push(firmware_version.clone());
            param_count += 1;
        }

        if let Some(status) = &update.status {
            query.push_str(&format!("status = ${}, ", param_count));
            bindings.push(status.to_string());
            param_count += 1;
        }

        if let Some(configuration) = &update.configuration {
            query.push_str(&format!("configuration = ${}, ", param_count));
            bindings.push(serde_json::to_string(configuration).unwrap());
            param_count += 1;
        }

        if let Some(capabilities) = &update.capabilities {
            query.push_str(&format!("capabilities = ${}, ", param_count));
            bindings.push(serde_json::to_string(capabilities).unwrap());
            param_count += 1;
        }

        if let Some(metadata) = &update.metadata {
            query.push_str(&format!("metadata = ${}, ", param_count));
            bindings.push(serde_json::to_string(metadata).unwrap());
            param_count += 1;
        }

        if let Some(certificate_expiry) = &update.certificate_expiry {
            query.push_str(&format!("certificate_expiry = ${}, ", param_count));
            bindings.push(certificate_expiry.to_rfc3339());
            param_count += 1;
        }

        // Remove trailing comma and space
        if query.ends_with(", ") {
            query.truncate(query.len() - 2);
        } else {
            // No fields to update
            return self.get_device_by_id(id).await;
        }

        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));

        // Execute update with dynamic parameters
        let mut query_builder = sqlx::query_as::<_, Device>(&query);
        for binding in bindings {
            query_builder = query_builder.bind(binding);
        }
        query_builder = query_builder.bind(id);

        let device = query_builder
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| UaipError::DatabaseError(e.to_string()))?
            .ok_or_else(|| UaipError::DeviceNotFound(id.to_string()))?;

        Ok(device)
    }

    /// Update device status
    ///
    /// # Arguments
    /// * `device_id` - Device identifier
    /// * `status` - New status
    ///
    /// # Returns
    /// * `Result<Device>` - The updated device or an error
    pub async fn update_status(&self, device_id: &str, status: DeviceStatus) -> UaipResult<Device> {
        let device = sqlx::query_as::<_, Device>(
            r#"
            UPDATE devices
            SET status = $1
            WHERE device_id = $2
            RETURNING *
            "#,
        )
        .bind(status)
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UaipError::DatabaseError(e.to_string()))?
        .ok_or_else(|| UaipError::DeviceNotFound(device_id.to_string()))?;

        Ok(device)
    }

    /// Update device last_seen timestamp
    ///
    /// # Arguments
    /// * `device_id` - Device identifier
    /// * `timestamp` - Last seen timestamp
    ///
    /// # Returns
    /// * `Result<Device>` - The updated device or an error
    pub async fn update_last_seen(
        &self,
        device_id: &str,
        timestamp: DateTime<Utc>,
    ) -> UaipResult<Device> {
        let device = sqlx::query_as::<_, Device>(
            r#"
            UPDATE devices
            SET last_seen = $1
            WHERE device_id = $2
            RETURNING *
            "#,
        )
        .bind(timestamp)
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UaipError::DatabaseError(e.to_string()))?
        .ok_or_else(|| UaipError::DeviceNotFound(device_id.to_string()))?;

        Ok(device)
    }

    /// Delete a device
    ///
    /// # Arguments
    /// * `id` - Device UUID
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error
    pub async fn delete_device(&self, id: Uuid) -> UaipResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM devices WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| UaipError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(UaipError::DeviceNotFound(id.to_string()));
        }

        Ok(())
    }

    /// List devices with optional filters
    ///
    /// # Arguments
    /// * `filter` - Query filters
    ///
    /// # Returns
    /// * `Result<Vec<Device>>` - List of devices or an error
    pub async fn list_devices(&self, filter: DeviceFilter) -> UaipResult<Vec<Device>> {
        let mut query = String::from("SELECT * FROM devices WHERE 1=1");
        let mut bindings: Vec<String> = Vec::new();
        let mut param_count = 1;

        if let Some(status) = &filter.status {
            query.push_str(&format!(" AND status = ${}", param_count));
            bindings.push(status.to_string());
            param_count += 1;
        }

        if let Some(manufacturer) = &filter.manufacturer {
            query.push_str(&format!(" AND manufacturer = ${}", param_count));
            bindings.push(manufacturer.clone());
            param_count += 1;
        }

        if let Some(model) = &filter.model {
            query.push_str(&format!(" AND model = ${}", param_count));
            bindings.push(model.clone());
            param_count += 1;
        }

        query.push_str(" ORDER BY registered_at DESC");

        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT ${}", param_count));
            bindings.push(limit.to_string());
            param_count += 1;
        }

        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET ${}", param_count));
            bindings.push(offset.to_string());
        }

        let mut query_builder = sqlx::query_as::<_, Device>(&query);
        for binding in bindings {
            query_builder = query_builder.bind(binding);
        }

        let devices = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| UaipError::DatabaseError(e.to_string()))?;

        Ok(devices)
    }

    /// Count total devices with optional filter
    ///
    /// # Arguments
    /// * `filter` - Query filters
    ///
    /// # Returns
    /// * `Result<i64>` - Device count or an error
    pub async fn count_devices(&self, filter: DeviceFilter) -> UaipResult<i64> {
        let mut query = String::from("SELECT COUNT(*) FROM devices WHERE 1=1");
        let mut bindings: Vec<String> = Vec::new();
        let mut param_count = 1;

        if let Some(status) = &filter.status {
            query.push_str(&format!(" AND status = ${}", param_count));
            bindings.push(status.to_string());
            param_count += 1;
        }

        if let Some(manufacturer) = &filter.manufacturer {
            query.push_str(&format!(" AND manufacturer = ${}", param_count));
            bindings.push(manufacturer.clone());
            param_count += 1;
        }

        if let Some(model) = &filter.model {
            query.push_str(&format!(" AND model = ${}", param_count));
            bindings.push(model.clone());
        }

        let mut query_builder = sqlx::query(&query);
        for binding in bindings {
            query_builder = query_builder.bind(binding);
        }

        let row = query_builder
            .fetch_one(&self.pool)
            .await
            .map_err(|e| UaipError::DatabaseError(e.to_string()))?;

        let count: i64 = row.get(0);
        Ok(count)
    }

    /// Check if a device exists by device_id
    ///
    /// # Arguments
    /// * `device_id` - Device identifier
    ///
    /// # Returns
    /// * `Result<bool>` - True if device exists, false otherwise
    pub async fn device_exists(&self, device_id: &str) -> UaipResult<bool> {
        let row = sqlx::query(
            r#"
            SELECT EXISTS(SELECT 1 FROM devices WHERE device_id = $1)
            "#,
        )
        .bind(device_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| UaipError::DatabaseError(e.to_string()))?;

        let exists: bool = row.get(0);
        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    // Note: These tests require a running PostgreSQL instance
    // They should be run as integration tests with a test database

    #[test]
    fn test_repository_creation() {
        // This is a simple unit test that doesn't require DB
        // Real tests will be in integration tests
    }
}
