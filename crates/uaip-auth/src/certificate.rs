//! X.509 Certificate Management
//!
//! This module handles X.509 certificates for device authentication.

use uaip_core::error::{Result, UaipError};
use x509_parser::prelude::*;
use chrono::{DateTime, Utc};
use std::collections::HashSet;

/// Certificate information extracted from X.509
#[derive(Debug, Clone)]
pub struct CertificateInfo {
    /// Subject common name (CN)
    pub common_name: String,
    /// Subject organization (O)
    pub organization: Option<String>,
    /// Serial number
    pub serial_number: String,
    /// Issuer common name
    pub issuer_cn: String,
    /// Not valid before
    pub not_before: DateTime<Utc>,
    /// Not valid after
    pub not_after: DateTime<Utc>,
    /// Public key (PEM format)
    pub public_key: Vec<u8>,
    /// Fingerprint (SHA-256)
    pub fingerprint: String,
}

/// Certificate validator
pub struct CertificateValidator {
    /// Trusted CA certificates (fingerprints)
    trusted_cas: HashSet<String>,
    /// Revoked certificate serial numbers
    revoked_serials: HashSet<String>,
}

impl CertificateValidator {
    /// Create a new certificate validator
    pub fn new() -> Self {
        Self {
            trusted_cas: HashSet::new(),
            revoked_serials: HashSet::new(),
        }
    }

    /// Add a trusted CA certificate fingerprint
    pub fn add_trusted_ca(&mut self, fingerprint: String) {
        self.trusted_cas.insert(fingerprint);
    }

    /// Revoke a certificate by serial number
    pub fn revoke_certificate(&mut self, serial: String) {
        self.revoked_serials.insert(serial);
    }

    /// Parse a PEM-encoded certificate
    pub fn parse_certificate(&self, pem_data: &str) -> Result<CertificateInfo> {
        // Remove PEM headers and decode base64
        let pem_data = pem_data.trim();
        let cert_data = self.extract_cert_data(pem_data)?;

        // Parse the X.509 certificate
        let (_, cert) = X509Certificate::from_der(&cert_data)
            .map_err(|e| UaipError::CertificateError(format!("Failed to parse certificate: {}", e)))?;

        // Extract subject CN
        let common_name = cert
            .subject()
            .iter_common_name()
            .next()
            .and_then(|cn| cn.as_str().ok())
            .ok_or_else(|| UaipError::CertificateError("No common name found".to_string()))?
            .to_string();

        // Extract organization
        let organization = cert
            .subject()
            .iter_organization()
            .next()
            .and_then(|o| o.as_str().ok())
            .map(String::from);

        // Extract issuer CN
        let issuer_cn = cert
            .issuer()
            .iter_common_name()
            .next()
            .and_then(|cn| cn.as_str().ok())
            .unwrap_or("Unknown")
            .to_string();

        // Get validity period
        let not_before = cert.validity().not_before.timestamp();
        let not_after = cert.validity().not_after.timestamp();

        // Extract serial number
        let serial_number = cert
            .serial
            .to_str_radix(16)
            .to_uppercase();

        // Calculate fingerprint (simplified - in production use proper SHA-256)
        let fingerprint = format!("SHA256:{:X}", cert_data.len()); // Placeholder

        Ok(CertificateInfo {
            common_name,
            organization,
            serial_number,
            issuer_cn,
            not_before: DateTime::from_timestamp(not_before, 0)
                .unwrap_or_else(|| Utc::now()),
            not_after: DateTime::from_timestamp(not_after, 0)
                .unwrap_or_else(|| Utc::now()),
            public_key: cert.public_key().raw.to_vec(),
            fingerprint,
        })
    }

    /// Validate a certificate
    pub fn validate(&self, cert_info: &CertificateInfo) -> Result<()> {
        let now = Utc::now();

        // Check if certificate is revoked
        if self.revoked_serials.contains(&cert_info.serial_number) {
            return Err(UaipError::CertificateError("Certificate has been revoked".to_string()));
        }

        // Check validity period
        if now < cert_info.not_before {
            return Err(UaipError::CertificateError("Certificate not yet valid".to_string()));
        }

        if now > cert_info.not_after {
            return Err(UaipError::CertificateError("Certificate has expired".to_string()));
        }

        // In production, verify certificate chain against trusted CAs
        // For now, we'll accept all non-revoked, non-expired certificates

        Ok(())
    }

    /// Extract certificate data from PEM format
    fn extract_cert_data(&self, pem_data: &str) -> Result<Vec<u8>> {
        let lines: Vec<&str> = pem_data.lines().collect();

        let start_idx = lines.iter().position(|l| l.contains("BEGIN CERTIFICATE"))
            .ok_or_else(|| UaipError::CertificateError("No BEGIN CERTIFICATE marker found".to_string()))?;

        let end_idx = lines.iter().position(|l| l.contains("END CERTIFICATE"))
            .ok_or_else(|| UaipError::CertificateError("No END CERTIFICATE marker found".to_string()))?;

        if start_idx >= end_idx {
            return Err(UaipError::CertificateError("Invalid PEM format".to_string()));
        }

        // Join base64 lines (skip the markers)
        let base64_data = lines[start_idx + 1..end_idx].join("");

        // Decode base64
        // In production, use proper base64 decoder
        let cert_data = base64_data.as_bytes().to_vec();

        Ok(cert_data)
    }

    /// Verify certificate challenge (for device authentication)
    pub fn verify_challenge(
        &self,
        cert_info: &CertificateInfo,
        _challenge: &[u8],
        _signature: &[u8],
    ) -> Result<bool> {
        // In production, verify signature using cert's public key
        // For now, placeholder implementation

        self.validate(cert_info)?;

        // TODO: Implement actual signature verification using ring or another crypto library
        // For now, return true if certificate is valid
        Ok(true)
    }
}

impl Default for CertificateValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Challenge-response data for device authentication
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChallengeRequest {
    /// Device ID
    pub device_id: String,
    /// Certificate (PEM format)
    pub certificate: String,
}

/// Challenge response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChallengeResponse {
    /// Challenge nonce
    pub challenge: String,
    /// Expiry timestamp
    pub expires_at: DateTime<Utc>,
}

impl ChallengeResponse {
    /// Create a new challenge
    pub fn new(challenge: String, ttl_seconds: i64) -> Self {
        Self {
            challenge,
            expires_at: Utc::now() + chrono::Duration::seconds(ttl_seconds),
        }
    }

    /// Check if challenge has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Challenge verification request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChallengeVerification {
    /// Device ID
    pub device_id: String,
    /// Challenge nonce
    pub challenge: String,
    /// Signature (base64 encoded)
    pub signature: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_validator_creation() {
        let validator = CertificateValidator::new();
        assert_eq!(validator.trusted_cas.len(), 0);
        assert_eq!(validator.revoked_serials.len(), 0);
    }

    #[test]
    fn test_add_trusted_ca() {
        let mut validator = CertificateValidator::new();
        validator.add_trusted_ca("SHA256:ABCD1234".to_string());
        assert!(validator.trusted_cas.contains("SHA256:ABCD1234"));
    }

    #[test]
    fn test_revoke_certificate() {
        let mut validator = CertificateValidator::new();
        validator.revoke_certificate("123456".to_string());
        assert!(validator.revoked_serials.contains("123456"));
    }

    #[test]
    fn test_challenge_response_creation() {
        let challenge = ChallengeResponse::new("random_challenge_123".to_string(), 300);
        assert_eq!(challenge.challenge, "random_challenge_123");
        assert!(!challenge.is_expired());
    }

    #[test]
    fn test_challenge_expiry() {
        let challenge = ChallengeResponse::new("test".to_string(), -10);
        assert!(challenge.is_expired());
    }

    // Note: Full X.509 parsing tests would require actual certificate files
    // In production, add integration tests with real certificates
}
