//! AES-256-GCM Encryption
//!
//! Provides authenticated encryption using AES-256-GCM for securing sensitive data.
//! Supports encryption/decryption of byte arrays and strings with automatic nonce generation.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// AES-256-GCM key size (32 bytes)
pub const KEY_SIZE: usize = 32;

/// Nonce size for AES-GCM (12 bytes)
pub const NONCE_SIZE: usize = 12;

/// Encrypted data with nonce
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Base64-encoded nonce
    pub nonce: String,

    /// Base64-encoded ciphertext (includes authentication tag)
    pub ciphertext: String,
}

/// AES-256-GCM encryption engine
pub struct EncryptionEngine {
    cipher: Aes256Gcm,
}

impl std::fmt::Debug for EncryptionEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptionEngine")
            .field("cipher", &"<AES-256-GCM>")
            .finish()
    }
}

impl EncryptionEngine {
    /// Create a new encryption engine with a random key
    pub fn new() -> Self {
        let mut key = [0u8; KEY_SIZE];
        OsRng.fill_bytes(&mut key);

        let cipher = Aes256Gcm::new_from_slice(&key).expect("Invalid key length");

        // Zeroize key from memory
        key.zeroize();

        Self { cipher }
    }

    /// Create encryption engine from an existing key
    pub fn from_key(key: &[u8; KEY_SIZE]) -> Result<Self, EncryptionError> {
        let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| EncryptionError::InvalidKey)?;

        Ok(Self { cipher })
    }

    /// Create encryption engine from a base64-encoded key
    pub fn from_base64_key(key_b64: &str) -> Result<Self, EncryptionError> {
        let key_bytes = BASE64
            .decode(key_b64)
            .map_err(|_| EncryptionError::InvalidKey)?;

        if key_bytes.len() != KEY_SIZE {
            return Err(EncryptionError::InvalidKey);
        }

        let mut key = [0u8; KEY_SIZE];
        key.copy_from_slice(&key_bytes);

        Self::from_key(&key)
    }

    /// Generate a new random encryption key
    pub fn generate_key() -> [u8; KEY_SIZE] {
        let mut key = [0u8; KEY_SIZE];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Generate a random nonce
    fn generate_nonce() -> [u8; NONCE_SIZE] {
        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);
        nonce
    }

    /// Encrypt plaintext bytes
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData, EncryptionError> {
        let nonce_bytes = Self::generate_nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| EncryptionError::EncryptionFailed)?;

        Ok(EncryptedData {
            nonce: BASE64.encode(nonce_bytes),
            ciphertext: BASE64.encode(ciphertext),
        })
    }

    /// Encrypt a string
    pub fn encrypt_string(&self, plaintext: &str) -> Result<EncryptedData, EncryptionError> {
        self.encrypt(plaintext.as_bytes())
    }

    /// Decrypt ciphertext bytes
    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>, EncryptionError> {
        let nonce_bytes = BASE64
            .decode(&encrypted.nonce)
            .map_err(|_| EncryptionError::InvalidNonce)?;

        if nonce_bytes.len() != NONCE_SIZE {
            return Err(EncryptionError::InvalidNonce);
        }

        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = BASE64
            .decode(&encrypted.ciphertext)
            .map_err(|_| EncryptionError::InvalidCiphertext)?;

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| EncryptionError::DecryptionFailed)?;

        Ok(plaintext)
    }

    /// Decrypt to a string
    pub fn decrypt_string(&self, encrypted: &EncryptedData) -> Result<String, EncryptionError> {
        let plaintext = self.decrypt(encrypted)?;
        String::from_utf8(plaintext).map_err(|_| EncryptionError::InvalidUtf8)
    }

    /// Encrypt and encode as JSON
    pub fn encrypt_to_json(&self, plaintext: &[u8]) -> Result<String, EncryptionError> {
        let encrypted = self.encrypt(plaintext)?;
        serde_json::to_string(&encrypted).map_err(|_| EncryptionError::SerializationFailed)
    }

    /// Decrypt from JSON
    pub fn decrypt_from_json(&self, json: &str) -> Result<Vec<u8>, EncryptionError> {
        let encrypted: EncryptedData =
            serde_json::from_str(json).map_err(|_| EncryptionError::DeserializationFailed)?;
        self.decrypt(&encrypted)
    }
}

impl Default for EncryptionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Encryption errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncryptionError {
    /// Invalid encryption key
    InvalidKey,

    /// Invalid nonce
    InvalidNonce,

    /// Invalid ciphertext
    InvalidCiphertext,

    /// Encryption operation failed
    EncryptionFailed,

    /// Decryption operation failed (may indicate tampering)
    DecryptionFailed,

    /// Invalid UTF-8 in decrypted data
    InvalidUtf8,

    /// JSON serialization failed
    SerializationFailed,

    /// JSON deserialization failed
    DeserializationFailed,
}

impl std::fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidKey => write!(f, "Invalid encryption key"),
            Self::InvalidNonce => write!(f, "Invalid nonce"),
            Self::InvalidCiphertext => write!(f, "Invalid ciphertext"),
            Self::EncryptionFailed => write!(f, "Encryption failed"),
            Self::DecryptionFailed => {
                write!(f, "Decryption failed - data may be corrupted or tampered")
            }
            Self::InvalidUtf8 => write!(f, "Decrypted data is not valid UTF-8"),
            Self::SerializationFailed => write!(f, "JSON serialization failed"),
            Self::DeserializationFailed => write!(f, "JSON deserialization failed"),
        }
    }
}

impl std::error::Error for EncryptionError {}

/// Utility function to encrypt a string with a new random key
pub fn encrypt_with_random_key(plaintext: &str) -> (EncryptedData, [u8; KEY_SIZE]) {
    let key = EncryptionEngine::generate_key();
    let engine = EncryptionEngine::from_key(&key).expect("Valid key");
    let encrypted = engine.encrypt_string(plaintext).expect("Encryption failed");
    (encrypted, key)
}

/// Utility function to decrypt with a key
pub fn decrypt_with_key(
    encrypted: &EncryptedData,
    key: &[u8; KEY_SIZE],
) -> Result<String, EncryptionError> {
    let engine = EncryptionEngine::from_key(key)?;
    engine.decrypt_string(encrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        let key = EncryptionEngine::generate_key();
        assert_eq!(key.len(), KEY_SIZE);
    }

    #[test]
    fn test_encrypt_decrypt_bytes() {
        let engine = EncryptionEngine::new();
        let plaintext = b"Hello, World!";

        let encrypted = engine.encrypt(plaintext).unwrap();
        let decrypted = engine.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_string() {
        let engine = EncryptionEngine::new();
        let plaintext = "Secret message";

        let encrypted = engine.encrypt_string(plaintext).unwrap();
        let decrypted = engine.decrypt_string(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encryption_produces_different_nonces() {
        let engine = EncryptionEngine::new();
        let plaintext = "Same message";

        let encrypted1 = engine.encrypt_string(plaintext).unwrap();
        let encrypted2 = engine.encrypt_string(plaintext).unwrap();

        // Nonces should be different
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        // Ciphertexts should also be different
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);

        // But both should decrypt to the same plaintext
        assert_eq!(engine.decrypt_string(&encrypted1).unwrap(), plaintext);
        assert_eq!(engine.decrypt_string(&encrypted2).unwrap(), plaintext);
    }

    #[test]
    fn test_decryption_with_wrong_key_fails() {
        let engine1 = EncryptionEngine::new();
        let engine2 = EncryptionEngine::new();

        let plaintext = "Secret";
        let encrypted = engine1.encrypt_string(plaintext).unwrap();

        // Different key should fail to decrypt
        let result = engine2.decrypt_string(&encrypted);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), EncryptionError::DecryptionFailed);
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let engine = EncryptionEngine::new();
        let plaintext = "Original message";

        let mut encrypted = engine.encrypt_string(plaintext).unwrap();

        // Tamper with the ciphertext
        encrypted.ciphertext = BASE64.encode(b"tampered data");

        let result = engine.decrypt_string(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_key() {
        let key = EncryptionEngine::generate_key();
        let engine = EncryptionEngine::from_key(&key).unwrap();

        let plaintext = "Test message";
        let encrypted = engine.encrypt_string(plaintext).unwrap();
        let decrypted = engine.decrypt_string(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_from_base64_key() {
        let key = EncryptionEngine::generate_key();
        let key_b64 = BASE64.encode(key);

        let engine = EncryptionEngine::from_base64_key(&key_b64).unwrap();

        let plaintext = "Test with base64 key";
        let encrypted = engine.encrypt_string(plaintext).unwrap();
        let decrypted = engine.decrypt_string(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_invalid_base64_key() {
        let result = EncryptionEngine::from_base64_key("invalid-base64!");
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_key_length() {
        let short_key = BASE64.encode(b"short");
        let result = EncryptionEngine::from_base64_key(&short_key);
        assert_eq!(result.unwrap_err(), EncryptionError::InvalidKey);
    }

    #[test]
    fn test_encrypt_decrypt_json() {
        let engine = EncryptionEngine::new();
        let plaintext = b"JSON test data";

        let json = engine.encrypt_to_json(plaintext).unwrap();
        let decrypted = engine.decrypt_from_json(&json).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_encrypted_data_serialization() {
        let engine = EncryptionEngine::new();
        let plaintext = "Serialize me";

        let encrypted = engine.encrypt_string(plaintext).unwrap();
        let json = serde_json::to_string(&encrypted).unwrap();
        let deserialized: EncryptedData = serde_json::from_str(&json).unwrap();

        let decrypted = engine.decrypt_string(&deserialized).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_utility_functions() {
        let plaintext = "Utility test";
        let (encrypted, key) = encrypt_with_random_key(plaintext);
        let decrypted = decrypt_with_key(&encrypted, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_empty_string_encryption() {
        let engine = EncryptionEngine::new();
        let plaintext = "";

        let encrypted = engine.encrypt_string(plaintext).unwrap();
        let decrypted = engine.decrypt_string(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_long_string_encryption() {
        let engine = EncryptionEngine::new();
        let plaintext = "A".repeat(10000);

        let encrypted = engine.encrypt_string(&plaintext).unwrap();
        let decrypted = engine.decrypt_string(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_binary_data_encryption() {
        let engine = EncryptionEngine::new();
        let plaintext: Vec<u8> = (0..=255).collect();

        let encrypted = engine.encrypt(&plaintext).unwrap();
        let decrypted = engine.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }
}
