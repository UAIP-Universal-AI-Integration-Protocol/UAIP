// Test Fixtures for Integration Tests

use uuid::Uuid;
use serde_json::json;
use super::client::{DeviceRegistration, DeviceCommand, Message};

/// Create a test device registration
pub fn device_fixture(name: &str, device_type: &str) -> DeviceRegistration {
    DeviceRegistration {
        name: name.to_string(),
        device_type: device_type.to_string(),
        capabilities: Some(vec!["temperature".to_string(), "humidity".to_string()]),
        location: Some("Test Location".to_string()),
    }
}

/// Create a temperature sensor fixture
pub fn temperature_sensor_fixture(name: &str) -> DeviceRegistration {
    DeviceRegistration {
        name: name.to_string(),
        device_type: "temperature_sensor".to_string(),
        capabilities: Some(vec!["temperature_reading".to_string()]),
        location: Some("Living Room".to_string()),
    }
}

/// Create a smart light fixture
pub fn smart_light_fixture(name: &str) -> DeviceRegistration {
    DeviceRegistration {
        name: name.to_string(),
        device_type: "light".to_string(),
        capabilities: Some(vec!["on_off".to_string(), "brightness".to_string()]),
        location: Some("Bedroom".to_string()),
    }
}

/// Create a thermostat fixture
pub fn thermostat_fixture(name: &str) -> DeviceRegistration {
    DeviceRegistration {
        name: name.to_string(),
        device_type: "thermostat".to_string(),
        capabilities: Some(vec![
            "temperature_control".to_string(),
            "temperature_reading".to_string(),
            "humidity_reading".to_string(),
        ]),
        location: Some("Living Room".to_string()),
    }
}

/// Create a device command fixture
pub fn command_fixture(command: &str) -> DeviceCommand {
    DeviceCommand {
        command: command.to_string(),
        params: json!({}),
        qos: Some(1),
        priority: Some(5),
    }
}

/// Create a light command fixture
pub fn light_command_fixture(on: bool, brightness: Option<u8>) -> DeviceCommand {
    DeviceCommand {
        command: if on { "turn_on" } else { "turn_off" }.to_string(),
        params: brightness.map(|b| json!({ "brightness": b })).unwrap_or(json!({})),
        qos: Some(1),
        priority: Some(5),
    }
}

/// Create a thermostat command fixture
pub fn thermostat_command_fixture(target_temp: f64) -> DeviceCommand {
    DeviceCommand {
        command: "set_temperature".to_string(),
        params: json!({ "temperature": target_temp }),
        qos: Some(2),
        priority: Some(8),
    }
}

/// Create a message fixture
pub fn message_fixture(source: &str, destination: &str) -> Message {
    Message {
        source: source.to_string(),
        destination: destination.to_string(),
        payload: json!({
            "temperature": 22.5,
            "humidity": 55.0,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }),
        qos: Some(1),
        priority: Some(5),
    }
}

/// Create a high-priority message fixture
pub fn high_priority_message_fixture(source: &str, destination: &str) -> Message {
    Message {
        source: source.to_string(),
        destination: destination.to_string(),
        payload: json!({
            "alert": "critical_temperature",
            "value": 45.0,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }),
        qos: Some(2),
        priority: Some(10),
    }
}

/// Generate a random device ID
pub fn random_device_id() -> String {
    Uuid::new_v4().to_string()
}

/// Create a batch of device fixtures
pub fn device_batch_fixture(count: usize, prefix: &str) -> Vec<DeviceRegistration> {
    (0..count)
        .map(|i| device_fixture(&format!("{}{}", prefix, i), "sensor"))
        .collect()
}

/// Create test user credentials
pub struct TestUser {
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn admin() -> Self {
        Self {
            username: "admin".to_string(),
            password: "admin_password".to_string(),
        }
    }

    pub fn user() -> Self {
        Self {
            username: "testuser".to_string(),
            password: "test_password".to_string(),
        }
    }

    pub fn custom(username: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
        }
    }
}

/// Create SQL insert for test user
pub fn create_test_user_sql(username: &str, password_hash: &str) -> String {
    format!(
        "INSERT INTO users (id, username, email, password_hash) VALUES ('{}', '{}', '{}@test.com', '{}')",
        Uuid::new_v4(),
        username,
        username,
        password_hash
    )
}

/// Create SQL insert for test device
pub fn create_test_device_sql(id: &str, name: &str, device_type: &str) -> String {
    format!(
        "INSERT INTO devices (id, name, device_type, status) VALUES ('{}', '{}', '{}', 'online')",
        id, name, device_type
    )
}
