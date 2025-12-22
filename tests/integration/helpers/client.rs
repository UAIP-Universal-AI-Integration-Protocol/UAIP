// HTTP Client Wrapper for Integration Tests

use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// API Client for integration tests
#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl ApiClient {
    pub fn new(client: Client, base_url: String) -> Self {
        Self {
            client,
            base_url,
            token: None,
        }
    }

    /// Set authentication token
    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    /// Login and set token
    pub async fn login(&mut self, username: &str, password: &str) -> anyhow::Result<LoginResponse> {
        let response = self.post("/api/v1/auth/login", &LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        }).await?;

        let login_resp: LoginResponse = response.json().await?;
        self.token = Some(login_resp.token.clone());
        Ok(login_resp)
    }

    /// GET request
    pub async fn get(&self, path: &str) -> anyhow::Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.get(&url);

        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }

        Ok(req.send().await?)
    }

    /// POST request
    pub async fn post<T: Serialize>(&self, path: &str, body: &T) -> anyhow::Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.post(&url).json(body);

        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }

        Ok(req.send().await?)
    }

    /// PUT request
    pub async fn put<T: Serialize>(&self, path: &str, body: &T) -> anyhow::Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.put(&url).json(body);

        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }

        Ok(req.send().await?)
    }

    /// DELETE request
    pub async fn delete(&self, path: &str) -> anyhow::Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.delete(&url);

        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }

        Ok(req.send().await?)
    }

    /// POST request expecting JSON response
    pub async fn post_json<T, R>(&self, path: &str, body: &T) -> anyhow::Result<R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let response = self.post(path, body).await?;
        Ok(response.json().await?)
    }

    /// GET request expecting JSON response
    pub async fn get_json<R>(&self, path: &str) -> anyhow::Result<R>
    where
        R: for<'de> Deserialize<'de>,
    {
        let response = self.get(path).await?;
        Ok(response.json().await?)
    }
}

#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: u64,
}

#[derive(Debug, Serialize)]
pub struct DeviceRegistration {
    pub name: String,
    pub device_type: String,
    pub capabilities: Option<Vec<String>>,
    pub location: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct DeviceListResponse {
    pub devices: Vec<Device>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct DeviceCommand {
    pub command: String,
    pub params: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qos: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct CommandResponse {
    pub message_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize)]
pub struct Message {
    pub source: String,
    pub destination: String,
    pub payload: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qos: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct MessageResponse {
    pub message_id: String,
    pub status: String,
}
