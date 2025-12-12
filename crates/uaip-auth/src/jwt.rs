//! JWT Token Generation and Validation
//!
//! This module handles JWT tokens for AI agent authentication using OAuth 2.0.

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use uaip_core::error::{Result, UaipError};

/// JWT Claims structure for AI agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (AI agent ID)
    pub sub: String,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// Scopes/permissions
    pub scopes: Vec<String>,
    /// Client ID
    pub client_id: String,
    /// Session ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// JWT token generator and validator
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    audience: String,
    expiry_seconds: i64,
}

impl JwtManager {
    /// Create a new JWT manager
    pub fn new(secret: &str, issuer: String, audience: String, expiry_seconds: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            issuer,
            audience,
            expiry_seconds,
        }
    }

    /// Generate a new JWT token for an AI agent
    pub fn generate_token(
        &self,
        agent_id: &str,
        client_id: &str,
        scopes: Vec<String>,
        session_id: Option<String>,
    ) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.expiry_seconds);

        let claims = Claims {
            sub: agent_id.to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            scopes,
            client_id: client_id.to_string(),
            session_id,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| UaipError::AuthenticationFailed(format!("Failed to generate token: {}", e)))
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        validation.validate_exp = true;

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| UaipError::AuthenticationFailed(format!("Invalid token: {}", e)))
    }

    /// Refresh a token (generate new token with same claims but new expiry)
    pub fn refresh_token(&self, old_token: &str) -> Result<String> {
        let claims = self.validate_token(old_token)?;

        self.generate_token(
            &claims.sub,
            &claims.client_id,
            claims.scopes,
            claims.session_id,
        )
    }

    /// Extract claims without full validation (useful for expired tokens)
    pub fn decode_without_validation(&self, token: &str) -> Result<Claims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false;
        validation.insecure_disable_signature_validation();

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| UaipError::InvalidMessage(format!("Failed to decode token: {}", e)))
    }

    /// Check if token is expired
    pub fn is_token_expired(&self, token: &str) -> bool {
        match self.decode_without_validation(token) {
            Ok(claims) => {
                let now = Utc::now().timestamp();
                claims.exp < now
            }
            Err(_) => true,
        }
    }

    /// Get remaining time until token expires
    pub fn get_token_ttl(&self, token: &str) -> Result<i64> {
        let claims = self.decode_without_validation(token)?;
        let now = Utc::now().timestamp();
        Ok((claims.exp - now).max(0))
    }

    /// Verify that token has required scope
    pub fn has_scope(&self, token: &str, required_scope: &str) -> Result<bool> {
        let claims = self.validate_token(token)?;
        Ok(claims.scopes.iter().any(|s| s == required_scope))
    }

    /// Verify that token has all required scopes
    pub fn has_all_scopes(&self, token: &str, required_scopes: &[String]) -> Result<bool> {
        let claims = self.validate_token(token)?;
        Ok(required_scopes.iter().all(|req| claims.scopes.contains(req)))
    }
}

/// OAuth 2.0 token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub scope: String,
}

impl TokenResponse {
    /// Create a new token response
    pub fn new(access_token: String, expires_in: i64, scopes: Vec<String>) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in,
            refresh_token: None,
            scope: scopes.join(" "),
        }
    }

    /// Add a refresh token
    pub fn with_refresh_token(mut self, refresh_token: String) -> Self {
        self.refresh_token = Some(refresh_token);
        self
    }
}

/// OAuth 2.0 token request (client credentials flow)
#[derive(Debug, Clone, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub client_id: String,
    pub client_secret: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

impl TokenRequest {
    /// Validate the token request
    pub fn validate(&self) -> Result<()> {
        if self.grant_type != "client_credentials" {
            return Err(UaipError::InvalidParameter(
                "Only client_credentials grant type is supported".to_string(),
            ));
        }

        if self.client_id.is_empty() {
            return Err(UaipError::InvalidParameter("client_id is required".to_string()));
        }

        if self.client_secret.is_empty() {
            return Err(UaipError::InvalidParameter("client_secret is required".to_string()));
        }

        Ok(())
    }

    /// Parse scopes from the request
    pub fn parse_scopes(&self) -> Vec<String> {
        self.scope
            .as_ref()
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> JwtManager {
        JwtManager::new(
            "test_secret_key_for_testing",
            "uaip-hub".to_string(),
            "uaip-api".to_string(),
            3600,
        )
    }

    #[test]
    fn test_generate_and_validate_token() {
        let manager = create_test_manager();
        let scopes = vec!["device:read".to_string(), "device:write".to_string()];

        let token = manager
            .generate_token("agent_001", "client_001", scopes.clone(), None)
            .expect("Should generate token");

        let claims = manager.validate_token(&token).expect("Should validate token");

        assert_eq!(claims.sub, "agent_001");
        assert_eq!(claims.client_id, "client_001");
        assert_eq!(claims.scopes, scopes);
        assert_eq!(claims.iss, "uaip-hub");
        assert_eq!(claims.aud, "uaip-api");
    }

    #[test]
    fn test_token_with_session() {
        let manager = create_test_manager();
        let scopes = vec!["device:read".to_string()];

        let token = manager
            .generate_token("agent_001", "client_001", scopes, Some("session_123".to_string()))
            .expect("Should generate token");

        let claims = manager.validate_token(&token).expect("Should validate token");
        assert_eq!(claims.session_id, Some("session_123".to_string()));
    }

    #[test]
    fn test_invalid_token() {
        let manager = create_test_manager();
        let result = manager.validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_expired_token() {
        // Create manager with very short expiry
        let manager = JwtManager::new(
            "test_secret",
            "uaip-hub".to_string(),
            "uaip-api".to_string(),
            -10, // Negative means already expired
        );

        let token = manager
            .generate_token("agent_001", "client_001", vec![], None)
            .expect("Should generate token");

        assert!(manager.is_token_expired(&token));
    }

    #[test]
    fn test_refresh_token() {
        let manager = create_test_manager();
        let scopes = vec!["device:read".to_string()];

        let token = manager
            .generate_token("agent_001", "client_001", scopes.clone(), None)
            .expect("Should generate token");

        std::thread::sleep(std::time::Duration::from_secs(1));

        let new_token = manager.refresh_token(&token).expect("Should refresh token");

        let old_claims = manager.validate_token(&token).expect("Old token should be valid");
        let new_claims = manager.validate_token(&new_token).expect("New token should be valid");

        assert_eq!(old_claims.sub, new_claims.sub);
        assert_eq!(old_claims.client_id, new_claims.client_id);
        assert!(new_claims.exp > old_claims.exp);
    }

    #[test]
    fn test_has_scope() {
        let manager = create_test_manager();
        let scopes = vec!["device:read".to_string(), "device:write".to_string()];

        let token = manager
            .generate_token("agent_001", "client_001", scopes, None)
            .expect("Should generate token");

        assert!(manager.has_scope(&token, "device:read").unwrap());
        assert!(manager.has_scope(&token, "device:write").unwrap());
        assert!(!manager.has_scope(&token, "device:delete").unwrap());
    }

    #[test]
    fn test_token_request_validation() {
        let valid_request = TokenRequest {
            grant_type: "client_credentials".to_string(),
            client_id: "client_001".to_string(),
            client_secret: "secret_123".to_string(),
            scope: Some("device:read device:write".to_string()),
        };

        assert!(valid_request.validate().is_ok());

        let scopes = valid_request.parse_scopes();
        assert_eq!(scopes, vec!["device:read", "device:write"]);
    }

    #[test]
    fn test_token_response_creation() {
        let response = TokenResponse::new(
            "access_token_here".to_string(),
            3600,
            vec!["device:read".to_string(), "device:write".to_string()],
        );

        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.expires_in, 3600);
        assert_eq!(response.scope, "device:read device:write");
    }
}
