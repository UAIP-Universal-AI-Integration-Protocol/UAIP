//! Error Types for UAIP
//!
//! This module defines error types used throughout the UAIP system.

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Result type alias for UAIP operations
pub type Result<T> = std::result::Result<T, UaipError>;

/// Main error type for UAIP operations
#[derive(Debug, Error)]
pub enum UaipError {
    /// Authentication errors
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Authorization errors
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    /// Invalid message format
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    /// Device not found
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    /// Device already registered
    #[error("Device already registered: {0}")]
    DeviceAlreadyRegistered(String),

    /// Capability not supported
    #[error("Capability not supported: {0}")]
    CapabilityNotSupported(String),

    /// Connection errors
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Timeout errors
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Database errors
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Encryption/decryption errors
    #[error("Encryption error: {0}")]
    EncryptionError(String),

    /// Certificate errors
    #[error("Certificate error: {0}")]
    CertificateError(String),

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Operation not permitted
    #[error("Operation not permitted: {0}")]
    NotPermitted(String),

    /// Resource not available
    #[error("Resource not available: {0}")]
    ResourceUnavailable(String),

    /// Internal server error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Generic error with custom message
    #[error("{0}")]
    Custom(String),
}

/// Error response structure for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code
    pub code: ErrorCode,
    /// Error message
    pub message: String,
    /// Additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// Timestamp
    pub timestamp: String,
}

/// Standard error codes for UAIP protocol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Authentication & Authorization (1xxx)
    AuthenticationFailed,
    AuthorizationFailed,
    InvalidToken,
    TokenExpired,
    CertificateInvalid,
    CertificateExpired,
    CertificateRevoked,

    // Message & Protocol (2xxx)
    InvalidMessage,
    MessageTooLarge,
    UnsupportedVersion,
    MissingRequiredField,
    InvalidMessageFormat,
    CorrelationIdMismatch,

    // Device Management (3xxx)
    DeviceNotFound,
    DeviceOffline,
    DeviceAlreadyRegistered,
    DeviceNotRegistered,
    DeviceDeactivated,
    CapabilityNotSupported,
    InvalidDeviceState,

    // Connection & Network (4xxx)
    ConnectionFailed,
    ConnectionTimeout,
    ConnectionLost,
    NetworkError,
    ServiceUnavailable,

    // Rate Limiting & Quota (5xxx)
    RateLimitExceeded,
    QuotaExceeded,
    TooManyRequests,

    // Configuration & Parameters (6xxx)
    InvalidConfiguration,
    InvalidParameter,
    MissingParameter,
    ParameterOutOfRange,

    // Resource Management (7xxx)
    ResourceNotFound,
    ResourceAlreadyExists,
    ResourceUnavailable,
    InsufficientPermissions,

    // Data & Encryption (8xxx)
    EncryptionFailed,
    DecryptionFailed,
    InvalidSignature,
    DataCorrupted,

    // Internal Errors (9xxx)
    InternalError,
    DatabaseError,
    CacheError,
    QueueError,

    // Generic
    Unknown,
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(code: ErrorCode, message: String) -> Self {
        Self {
            code,
            message,
            details: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Add details to the error response
    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }
}

impl From<UaipError> for ErrorResponse {
    fn from(error: UaipError) -> Self {
        let (code, message) = match &error {
            UaipError::AuthenticationFailed(msg) => {
                (ErrorCode::AuthenticationFailed, msg.clone())
            }
            UaipError::AuthorizationFailed(msg) => {
                (ErrorCode::AuthorizationFailed, msg.clone())
            }
            UaipError::InvalidMessage(msg) => {
                (ErrorCode::InvalidMessage, msg.clone())
            }
            UaipError::DeviceNotFound(msg) => {
                (ErrorCode::DeviceNotFound, msg.clone())
            }
            UaipError::DeviceAlreadyRegistered(msg) => {
                (ErrorCode::DeviceAlreadyRegistered, msg.clone())
            }
            UaipError::CapabilityNotSupported(msg) => {
                (ErrorCode::CapabilityNotSupported, msg.clone())
            }
            UaipError::ConnectionError(msg) => {
                (ErrorCode::ConnectionFailed, msg.clone())
            }
            UaipError::Timeout(msg) => {
                (ErrorCode::ConnectionTimeout, msg.clone())
            }
            UaipError::RateLimitExceeded => {
                (ErrorCode::RateLimitExceeded, "Rate limit exceeded".to_string())
            }
            UaipError::InvalidConfiguration(msg) => {
                (ErrorCode::InvalidConfiguration, msg.clone())
            }
            UaipError::SerializationError(e) => {
                (ErrorCode::InvalidMessageFormat, e.to_string())
            }
            UaipError::DatabaseError(msg) => {
                (ErrorCode::DatabaseError, msg.clone())
            }
            UaipError::EncryptionError(msg) => {
                (ErrorCode::EncryptionFailed, msg.clone())
            }
            UaipError::CertificateError(msg) => {
                (ErrorCode::CertificateInvalid, msg.clone())
            }
            UaipError::InvalidParameter(msg) => {
                (ErrorCode::InvalidParameter, msg.clone())
            }
            UaipError::NotPermitted(msg) => {
                (ErrorCode::InsufficientPermissions, msg.clone())
            }
            UaipError::ResourceUnavailable(msg) => {
                (ErrorCode::ResourceUnavailable, msg.clone())
            }
            UaipError::InternalError(msg) => {
                (ErrorCode::InternalError, msg.clone())
            }
            UaipError::Custom(msg) => {
                (ErrorCode::Unknown, msg.clone())
            }
        };

        ErrorResponse::new(code, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = UaipError::DeviceNotFound("device_001".to_string());
        assert_eq!(err.to_string(), "Device not found: device_001");
    }

    #[test]
    fn test_error_response_conversion() {
        let err = UaipError::AuthenticationFailed("Invalid credentials".to_string());
        let response: ErrorResponse = err.into();

        assert_eq!(response.code, ErrorCode::AuthenticationFailed);
        assert_eq!(response.message, "Invalid credentials");
    }

    #[test]
    fn test_error_response_with_details() {
        let response = ErrorResponse::new(
            ErrorCode::DeviceNotFound,
            "Device not found".to_string(),
        )
        .with_details("The device may have been deleted".to_string());

        assert!(response.details.is_some());
    }
}
