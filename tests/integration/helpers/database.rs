// Database Test Helpers

use sqlx::PgPool;
use uuid::Uuid;

/// Database test helper
pub struct DatabaseHelper<'a> {
    pool: &'a PgPool,
}

impl<'a> DatabaseHelper<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Insert a test user
    pub async fn insert_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> anyhow::Result<String> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, $4)"
        )
        .bind(&id)
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .execute(self.pool)
        .await?;
        Ok(id)
    }

    /// Insert a test device
    pub async fn insert_device(
        &self,
        name: &str,
        device_type: &str,
        status: &str,
    ) -> anyhow::Result<String> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO devices (id, name, device_type, status, registered_at) VALUES ($1, $2, $3, $4, NOW())"
        )
        .bind(&id)
        .bind(name)
        .bind(device_type)
        .bind(status)
        .execute(self.pool)
        .await?;
        Ok(id)
    }

    /// Insert a test message
    pub async fn insert_message(
        &self,
        source_id: &str,
        destination_id: &str,
        payload: serde_json::Value,
    ) -> anyhow::Result<String> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO messages (id, source_id, destination_id, payload, status, qos, priority, created_at)
             VALUES ($1, $2, $3, $4, 'queued', 1, 5, NOW())"
        )
        .bind(&id)
        .bind(source_id)
        .bind(destination_id)
        .bind(payload)
        .execute(self.pool)
        .await?;
        Ok(id)
    }

    /// Count devices
    pub async fn count_devices(&self) -> anyhow::Result<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM devices")
            .fetch_one(self.pool)
            .await?;
        Ok(count)
    }

    /// Count devices by status
    pub async fn count_devices_by_status(&self, status: &str) -> anyhow::Result<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM devices WHERE status = $1")
            .bind(status)
            .fetch_one(self.pool)
            .await?;
        Ok(count)
    }

    /// Count messages
    pub async fn count_messages(&self) -> anyhow::Result<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages")
            .fetch_one(self.pool)
            .await?;
        Ok(count)
    }

    /// Count messages by status
    pub async fn count_messages_by_status(&self, status: &str) -> anyhow::Result<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages WHERE status = $1")
            .bind(status)
            .fetch_one(self.pool)
            .await?;
        Ok(count)
    }

    /// Get device by ID
    pub async fn get_device(&self, id: &str) -> anyhow::Result<Option<Device>> {
        let device = sqlx::query_as::<_, Device>(
            "SELECT id, name, device_type, status FROM devices WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;
        Ok(device)
    }

    /// Get all devices
    pub async fn get_all_devices(&self) -> anyhow::Result<Vec<Device>> {
        let devices = sqlx::query_as::<_, Device>(
            "SELECT id, name, device_type, status FROM devices ORDER BY name"
        )
        .fetch_all(self.pool)
        .await?;
        Ok(devices)
    }

    /// Delete device by ID
    pub async fn delete_device(&self, id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM devices WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    /// Delete all test data
    pub async fn cleanup(&self) -> anyhow::Result<()> {
        sqlx::query("TRUNCATE TABLE messages, devices, user_roles, users, roles CASCADE")
            .execute(self.pool)
            .await?;
        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub status: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Message {
    pub id: String,
    pub source_id: String,
    pub destination_id: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub qos: i16,
    pub priority: i16,
}

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
}
