//! HTTP/REST Protocol Adapter
//!
//! Provides HTTP client functionality for connecting devices that communicate via REST APIs.
//! Supports common HTTP methods (GET, POST, PUT, DELETE) with request/response handling.

use reqwest::{Client, Method, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info};

use uaip_core::{
    error::{Result, UaipError},
    message::UaipMessage,
};

/// HTTP adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    /// Base URL for HTTP requests
    pub base_url: String,

    /// Request timeout in seconds
    pub timeout_seconds: u64,

    /// Maximum number of retries
    pub max_retries: u32,

    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,

    /// Default headers to include in all requests
    #[serde(default)]
    pub default_headers: HashMap<String, String>,

    /// Authentication configuration
    pub auth: Option<HttpAuth>,

    /// Enable TLS certificate verification
    #[serde(default = "default_true")]
    pub verify_tls: bool,

    /// Connection pool max idle per host
    #[serde(default = "default_pool_size")]
    pub pool_max_idle_per_host: usize,
}

fn default_true() -> bool {
    true
}

fn default_pool_size() -> usize {
    10
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
            default_headers: HashMap::new(),
            auth: None,
            verify_tls: true,
            pool_max_idle_per_host: 10,
        }
    }
}

/// HTTP authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum HttpAuth {
    /// Basic authentication
    Basic { username: String, password: String },
    /// Bearer token authentication
    Bearer { token: String },
    /// API key authentication
    ApiKey {
        header_name: String,
        api_key: String,
    },
}

/// HTTP adapter for device communication
pub struct HttpAdapter {
    client: Client,
    config: HttpConfig,
}

impl HttpAdapter {
    /// Create a new HTTP adapter
    pub fn new(config: HttpConfig) -> Result<Self> {
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .danger_accept_invalid_certs(!config.verify_tls);

        // Add default headers
        let mut headers = reqwest::header::HeaderMap::new();
        for (key, value) in &config.default_headers {
            let header_name =
                reqwest::header::HeaderName::from_bytes(key.as_bytes()).map_err(|e| {
                    UaipError::InvalidConfiguration(format!("Invalid header name: {}", e))
                })?;
            let header_value = reqwest::header::HeaderValue::from_str(value).map_err(|e| {
                UaipError::InvalidConfiguration(format!("Invalid header value: {}", e))
            })?;
            headers.insert(header_name, header_value);
        }

        client_builder = client_builder.default_headers(headers);

        let client = client_builder.build().map_err(|e| {
            UaipError::ConnectionError(format!("Failed to create HTTP client: {}", e))
        })?;

        info!("HTTP adapter created for base URL: {}", config.base_url);

        Ok(Self { client, config })
    }

    /// Build a request with authentication
    fn build_request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = if path.starts_with("http://") || path.starts_with("https://") {
            path.to_string()
        } else {
            format!("{}{}", self.config.base_url.trim_end_matches('/'), path)
        };

        let mut request = self.client.request(method, &url);

        // Add authentication
        if let Some(auth) = &self.config.auth {
            request = match auth {
                HttpAuth::Basic { username, password } => {
                    request.basic_auth(username, Some(password))
                }
                HttpAuth::Bearer { token } => request.bearer_auth(token),
                HttpAuth::ApiKey {
                    header_name,
                    api_key,
                } => request.header(header_name, api_key),
            };
        }

        request
    }

    /// Execute a request with retries
    async fn execute_with_retry(&self, request: RequestBuilder) -> Result<Response> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                debug!("Retry attempt {} after delay", attempt);
                tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
            }

            // Clone the request for retry
            let req = request.try_clone().ok_or_else(|| {
                UaipError::InternalError("Failed to clone request for retry".to_string())
            })?;

            match req.send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        debug!("HTTP request successful: {}", status);
                        return Ok(response);
                    } else if status.is_client_error() {
                        // Don't retry client errors
                        error!("HTTP client error: {}", status);
                        return Err(UaipError::InvalidParameter(format!(
                            "HTTP client error: {}",
                            status
                        )));
                    } else {
                        // Retry server errors
                        error!("HTTP server error: {}, will retry", status);
                        last_error = Some(UaipError::ConnectionError(format!(
                            "HTTP server error: {}",
                            status
                        )));
                    }
                }
                Err(e) => {
                    error!("HTTP request failed: {}", e);
                    last_error = Some(UaipError::ConnectionError(format!(
                        "HTTP request failed: {}",
                        e
                    )));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            UaipError::MaxRetriesExceeded("HTTP request retries exhausted".to_string())
        }))
    }

    /// Send a GET request
    pub async fn get(&self, path: &str) -> Result<Response> {
        let request = self.build_request(Method::GET, path);
        self.execute_with_retry(request).await
    }

    /// Send a GET request and parse JSON response
    pub async fn get_json<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let response = self.get(path).await?;
        let data = response
            .json::<T>()
            .await
            .map_err(|e| UaipError::InvalidMessage(format!("Failed to parse JSON: {}", e)))?;
        Ok(data)
    }

    /// Send a POST request with JSON body
    pub async fn post_json<T: Serialize>(&self, path: &str, body: &T) -> Result<Response> {
        let request = self.build_request(Method::POST, path).json(body);
        self.execute_with_retry(request).await
    }

    /// Send a POST request with JSON body and parse JSON response
    pub async fn post_json_response<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<R> {
        let response = self.post_json(path, body).await?;
        let data = response
            .json::<R>()
            .await
            .map_err(|e| UaipError::InvalidMessage(format!("Failed to parse JSON: {}", e)))?;
        Ok(data)
    }

    /// Send a PUT request with JSON body
    pub async fn put_json<T: Serialize>(&self, path: &str, body: &T) -> Result<Response> {
        let request = self.build_request(Method::PUT, path).json(body);
        self.execute_with_retry(request).await
    }

    /// Send a DELETE request
    pub async fn delete(&self, path: &str) -> Result<Response> {
        let request = self.build_request(Method::DELETE, path);
        self.execute_with_retry(request).await
    }

    /// Send a UAIP message via HTTP POST
    pub async fn send_uaip_message(&self, path: &str, message: &UaipMessage) -> Result<Response> {
        self.post_json(path, message).await
    }

    /// Send a UAIP message and expect a UAIP response
    pub async fn send_uaip_message_response(
        &self,
        path: &str,
        message: &UaipMessage,
    ) -> Result<UaipMessage> {
        self.post_json_response(path, message).await
    }

    /// Get the HTTP configuration
    pub fn get_config(&self) -> &HttpConfig {
        &self.config
    }

    /// Check if a URL is reachable (health check)
    pub async fn health_check(&self) -> Result<()> {
        let request = self.build_request(Method::GET, "/health");
        let response = request
            .send()
            .await
            .map_err(|e| UaipError::ConnectionError(format!("Health check failed: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(UaipError::ConnectionError(format!(
                "Health check failed with status: {}",
                response.status()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_config_default() {
        let config = HttpConfig::default();
        assert_eq!(config.base_url, "http://localhost:8080");
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_retries, 3);
        assert!(config.verify_tls);
        assert_eq!(config.pool_max_idle_per_host, 10);
    }

    #[test]
    fn test_http_config_custom() {
        let config = HttpConfig {
            base_url: "https://api.example.com".to_string(),
            timeout_seconds: 60,
            max_retries: 5,
            retry_delay_ms: 2000,
            default_headers: {
                let mut headers = HashMap::new();
                headers.insert("X-Custom-Header".to_string(), "value".to_string());
                headers
            },
            auth: Some(HttpAuth::Bearer {
                token: "test-token".to_string(),
            }),
            verify_tls: false,
            pool_max_idle_per_host: 20,
        };

        assert_eq!(config.base_url, "https://api.example.com");
        assert_eq!(config.timeout_seconds, 60);
        assert_eq!(config.max_retries, 5);
        assert!(!config.verify_tls);
    }

    #[test]
    fn test_http_auth_basic() {
        let auth = HttpAuth::Basic {
            username: "user".to_string(),
            password: "pass".to_string(),
        };

        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("basic"));
        assert!(json.contains("user"));
    }

    #[test]
    fn test_http_auth_bearer() {
        let auth = HttpAuth::Bearer {
            token: "token123".to_string(),
        };

        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("bearer"));
        assert!(json.contains("token123"));
    }

    #[test]
    fn test_http_auth_api_key() {
        let auth = HttpAuth::ApiKey {
            header_name: "X-API-Key".to_string(),
            api_key: "key123".to_string(),
        };

        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("apikey"));
        assert!(json.contains("X-API-Key"));
    }

    #[tokio::test]
    async fn test_http_adapter_creation() {
        let config = HttpConfig::default();
        let result = HttpAdapter::new(config);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_http_adapter_with_custom_config() {
        let config = HttpConfig {
            base_url: "https://api.test.com".to_string(),
            timeout_seconds: 10,
            max_retries: 2,
            retry_delay_ms: 500,
            default_headers: HashMap::new(),
            auth: Some(HttpAuth::Bearer {
                token: "test".to_string(),
            }),
            verify_tls: true,
            pool_max_idle_per_host: 5,
        };

        let adapter = HttpAdapter::new(config).unwrap();
        assert_eq!(adapter.get_config().base_url, "https://api.test.com");
        assert_eq!(adapter.get_config().timeout_seconds, 10);
        assert_eq!(adapter.get_config().max_retries, 2);
    }
}
