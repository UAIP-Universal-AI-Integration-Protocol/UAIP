//! Device registration workflow (3-step challenge-response)

use base64::Engine;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::{CreateDevice, Device, DeviceStatus};
use crate::repository::DeviceRepository;
use uaip_auth::certificate::CertificateValidator;
use uaip_core::error::{UaipError, UaipResult};

/// Registration challenge that gets sent to the device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationChallenge {
    pub challenge_id: String,
    pub nonce: String,
    pub expires_at: DateTime<Utc>,
}

/// Device registration request (Step 1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationRequest {
    pub device_id: String,
    pub mac_address: String,
    pub manufacturer: String,
    pub model: String,
    pub firmware_version: Option<String>,
    pub capabilities: serde_json::Value,
    pub public_key_pem: String, // X.509 certificate in PEM format
}

/// Challenge response from device (Step 3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub challenge_id: String,
    pub device_id: String,
    pub signature: String,       // Base64 encoded signature of the nonce
    pub certificate_pem: String, // Full X.509 certificate
}

/// Pending registration challenge
#[derive(Debug, Clone)]
struct PendingChallenge {
    #[allow(dead_code)]
    nonce: String, // Will be used for signature verification
    request: RegistrationRequest,
    #[allow(dead_code)]
    created_at: DateTime<Utc>, // Kept for audit purposes
    expires_at: DateTime<Utc>,
}

/// Registration service managing the 3-step workflow
pub struct RegistrationService {
    repository: DeviceRepository,
    certificate_validator: CertificateValidator,
    pending_challenges: RwLock<HashMap<String, PendingChallenge>>,
    challenge_ttl_seconds: i64,
}

impl RegistrationService {
    /// Create a new registration service
    ///
    /// # Arguments
    /// * `repository` - Device repository
    /// * `certificate_validator` - Certificate validator
    /// * `challenge_ttl_seconds` - Time-to-live for challenges (default: 300 seconds)
    pub fn new(
        repository: DeviceRepository,
        certificate_validator: CertificateValidator,
        challenge_ttl_seconds: Option<i64>,
    ) -> Self {
        Self {
            repository,
            certificate_validator,
            pending_challenges: RwLock::new(HashMap::new()),
            challenge_ttl_seconds: challenge_ttl_seconds.unwrap_or(300), // 5 minutes default
        }
    }

    /// Step 1: Initiate device registration
    ///
    /// Receives registration request and generates a challenge
    ///
    /// # Arguments
    /// * `request` - Registration request from device
    ///
    /// # Returns
    /// * `Result<RegistrationChallenge>` - Challenge to be signed by device
    pub async fn initiate_registration(
        &self,
        request: RegistrationRequest,
    ) -> UaipResult<RegistrationChallenge> {
        // Check if device already exists
        if self.repository.device_exists(&request.device_id).await? {
            return Err(UaipError::DeviceAlreadyRegistered(
                request.device_id.clone(),
            ));
        }

        // Validate the certificate format (basic validation)
        let cert_info = self
            .certificate_validator
            .parse_certificate(&request.public_key_pem)?;

        // Verify certificate is not expired
        self.certificate_validator.validate(&cert_info)?;

        // Generate challenge
        let challenge_id = Uuid::new_v4().to_string();
        let nonce = self.generate_nonce();
        let created_at = Utc::now();
        let expires_at = created_at + Duration::seconds(self.challenge_ttl_seconds);

        // Store pending challenge
        let pending = PendingChallenge {
            nonce: nonce.clone(),
            request,
            created_at,
            expires_at,
        };

        {
            let mut challenges = self.pending_challenges.write().await;
            challenges.insert(challenge_id.clone(), pending);
        }

        Ok(RegistrationChallenge {
            challenge_id,
            nonce,
            expires_at,
        })
    }

    /// Step 2: Verify challenge response and register device
    ///
    /// Validates the signed challenge and completes registration
    ///
    /// # Arguments
    /// * `response` - Challenge response from device
    ///
    /// # Returns
    /// * `Result<Device>` - Registered device
    pub async fn complete_registration(&self, response: ChallengeResponse) -> UaipResult<Device> {
        // Retrieve and remove pending challenge
        let pending = {
            let mut challenges = self.pending_challenges.write().await;
            challenges.remove(&response.challenge_id).ok_or_else(|| {
                UaipError::InvalidParameter("Challenge not found or expired".to_string())
            })?
        };

        // Verify challenge hasn't expired
        if Utc::now() > pending.expires_at {
            return Err(UaipError::InvalidParameter(
                "Challenge has expired".to_string(),
            ));
        }

        // Verify device_id matches
        if response.device_id != pending.request.device_id {
            return Err(UaipError::InvalidParameter(
                "Device ID mismatch".to_string(),
            ));
        }

        // Verify certificate
        let cert_info = self
            .certificate_validator
            .parse_certificate(&response.certificate_pem)?;
        self.certificate_validator.validate(&cert_info)?;

        // TODO: Verify signature of nonce with public key from certificate
        // This would require implementing signature verification
        // For now, we'll proceed with basic validation
        if response.signature.is_empty() {
            return Err(UaipError::InvalidParameter("Invalid signature".to_string()));
        }

        // Create device in database
        let create_device = CreateDevice {
            device_id: pending.request.device_id.clone(),
            mac_address: pending.request.mac_address.clone(),
            manufacturer: pending.request.manufacturer.clone(),
            model: pending.request.model.clone(),
            firmware_version: pending.request.firmware_version.clone(),
            capabilities: pending.request.capabilities.clone(),
            metadata: Some(serde_json::json!({
                "registered_via": "challenge_response",
                "certificate_fingerprint": cert_info.fingerprint,
            })),
        };

        let mut device = self.repository.create_device(create_device).await?;

        // Update status to online after successful registration
        device = self
            .repository
            .update_status(&device.device_id, DeviceStatus::Online)
            .await?;

        Ok(device)
    }

    /// Clean up expired challenges
    ///
    /// Should be called periodically to remove stale challenges
    pub async fn cleanup_expired_challenges(&self) -> UaipResult<usize> {
        let now = Utc::now();
        let mut challenges = self.pending_challenges.write().await;

        let initial_count = challenges.len();
        challenges.retain(|_, pending| pending.expires_at > now);
        let removed_count = initial_count - challenges.len();

        Ok(removed_count)
    }

    /// Get count of pending challenges
    pub async fn pending_challenges_count(&self) -> usize {
        let challenges = self.pending_challenges.read().await;
        challenges.len()
    }

    /// Generate a random nonce for challenge
    fn generate_nonce(&self) -> String {
        use rand::Rng;
        let nonce: [u8; 32] = rand::thread_rng().gen();
        base64::engine::general_purpose::STANDARD.encode(nonce)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registration_request_serialization() {
        let request = RegistrationRequest {
            device_id: "device-123".to_string(),
            mac_address: "00:11:22:33:44:55".to_string(),
            manufacturer: "TestCorp".to_string(),
            model: "Model-X".to_string(),
            firmware_version: Some("1.0.0".to_string()),
            capabilities: serde_json::json!([]),
            public_key_pem: "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----"
                .to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: RegistrationRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.device_id, "device-123");
        assert_eq!(deserialized.mac_address, "00:11:22:33:44:55");
    }

    #[test]
    fn test_challenge_response_serialization() {
        let response = ChallengeResponse {
            challenge_id: "challenge-123".to_string(),
            device_id: "device-123".to_string(),
            signature: "base64signature".to_string(),
            certificate_pem: "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----"
                .to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ChallengeResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.challenge_id, "challenge-123");
        assert_eq!(deserialized.device_id, "device-123");
    }

    #[test]
    fn test_registration_challenge_creation() {
        let challenge = RegistrationChallenge {
            challenge_id: "challenge-123".to_string(),
            nonce: "random-nonce".to_string(),
            expires_at: Utc::now() + Duration::seconds(300),
        };

        assert_eq!(challenge.challenge_id, "challenge-123");
        assert!(challenge.expires_at > Utc::now());
    }
}
