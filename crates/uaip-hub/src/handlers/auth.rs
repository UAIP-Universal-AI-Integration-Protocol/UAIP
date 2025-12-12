//! Authentication handlers

use axum::{extract::State, Json};
use std::sync::Arc;

use uaip_core::error::UaipError;

use crate::api::rest::{ApiResult, AppState, LoginRequest, LoginResponse};

/// Login handler (OAuth 2.0 client_credentials flow)
pub async fn login(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    // Validate grant type
    if request.grant_type != "client_credentials" {
        return Err(UaipError::InvalidParameter(format!(
            "Unsupported grant_type: {}. Only 'client_credentials' is supported",
            request.grant_type
        ))
        .into());
    }

    // TODO: Validate client_id and client_secret against database
    // TODO: Generate JWT token using uaip-auth
    // For now, return a placeholder token

    let access_token = format!("placeholder_token_{}", uuid::Uuid::new_v4());

    Ok(Json(LoginResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600, // 1 hour
        scope: request.scope,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_login_success() {
        let state = Arc::new(AppState::new());
        let request = LoginRequest {
            grant_type: "client_credentials".to_string(),
            client_id: "test_client".to_string(),
            client_secret: Some("test_secret".to_string()),
            scope: Some("device:read device:write".to_string()),
        };

        let result = login(State(state), Json(request)).await;
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.expires_in, 3600);
        assert!(response.access_token.starts_with("placeholder_token_"));
    }

    #[tokio::test]
    async fn test_login_invalid_grant_type() {
        let state = Arc::new(AppState::new());
        let request = LoginRequest {
            grant_type: "password".to_string(),
            client_id: "test_client".to_string(),
            client_secret: Some("test_secret".to_string()),
            scope: None,
        };

        let result = login(State(state), Json(request)).await;
        assert!(result.is_err());
    }
}
