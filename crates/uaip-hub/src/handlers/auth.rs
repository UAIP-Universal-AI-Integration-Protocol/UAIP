//! Authentication handlers with JWT token generation and database validation

use axum::{extract::State, Json};
use std::sync::Arc;

use uaip_auth::jwt::JwtManager;
use uaip_core::error::UaipError;

use crate::api::rest::{ApiResult, AppState, LoginRequest, LoginResponse};

/// AI Agent record from database
#[derive(Debug, sqlx::FromRow)]
struct AiAgent {
    id: sqlx::types::Uuid,
    client_id: String,
    name: String,
    client_secret_hash: String,
    scopes: Vec<String>,
    active: bool,
}

/// Login handler (OAuth 2.0 client_credentials flow)
pub async fn login(
    State(state): State<Arc<AppState>>,
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

    // Validate client_id is provided
    if request.client_id.is_empty() {
        return Err(UaipError::InvalidParameter("client_id is required".to_string()).into());
    }

    // Validate client_secret is provided
    let client_secret = request.client_secret.as_ref().ok_or_else(|| {
        UaipError::InvalidParameter("client_secret is required".to_string())
    })?;

    if client_secret.is_empty() {
        return Err(UaipError::InvalidParameter("client_secret cannot be empty".to_string()).into());
    }

    // Get database pool (or return error if not configured)
    let db_pool = state.db_pool.as_ref().ok_or_else(|| {
        UaipError::InternalError("Database not configured".to_string())
    })?;

    // Query the AI agent from the database
    let agent = sqlx::query_as::<_, AiAgent>(
        "SELECT id, client_id, name, client_secret_hash, scopes, active
         FROM ai_agents
         WHERE client_id = $1",
    )
    .bind(&request.client_id)
    .fetch_optional(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error during authentication: {}", e);
        UaipError::InternalError("Authentication failed".to_string())
    })?
    .ok_or_else(|| {
        tracing::warn!("Authentication failed for client_id: {}", request.client_id);
        UaipError::AuthenticationFailed("Invalid credentials".to_string())
    })?;

    // Check if agent is active
    if !agent.active {
        tracing::warn!(
            "Authentication attempted for inactive agent: {}",
            request.client_id
        );
        // Log audit event
        log_audit_event(
            db_pool,
            &agent.id.to_string(),
            "ai_agent",
            "login",
            false,
            Some("Agent is inactive"),
        )
        .await;

        return Err(UaipError::AuthenticationFailed("Account is inactive".to_string()).into());
    }

    // Verify the client secret using bcrypt
    let password_valid = bcrypt::verify(client_secret, &agent.client_secret_hash)
        .map_err(|e| {
            tracing::error!("Password verification error: {}", e);
            UaipError::InternalError("Authentication failed".to_string())
        })?;

    if !password_valid {
        tracing::warn!(
            "Invalid password for client_id: {}",
            request.client_id
        );
        // Log failed authentication
        log_audit_event(
            db_pool,
            &agent.id.to_string(),
            "ai_agent",
            "login",
            false,
            Some("Invalid password"),
        )
        .await;

        return Err(UaipError::AuthenticationFailed("Invalid credentials".to_string()).into());
    }

    // Determine scopes (use requested scopes if provided, otherwise use default agent scopes)
    let scopes = request
        .scope
        .as_ref()
        .map(|s| s.split_whitespace().map(String::from).collect())
        .unwrap_or_else(|| agent.scopes.clone());

    // Create JWT manager (get secret from environment or use default for development)
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "uaip-development-secret-change-in-production".to_string());

    let jwt_manager = JwtManager::new(
        &jwt_secret,
        "uaip-hub".to_string(),
        "uaip-api".to_string(),
        3600, // 1 hour expiry
    );

    // Generate access token
    let access_token = jwt_manager
        .generate_token(&agent.id.to_string(), &agent.client_id, scopes.clone(), None)
        .map_err(|e| {
            tracing::error!("Failed to generate JWT token: {}", e);
            UaipError::InternalError("Failed to generate token".to_string())
        })?;

    // Generate refresh token (7-day expiry)
    let refresh_jwt_manager = JwtManager::new(
        &jwt_secret,
        "uaip-hub".to_string(),
        "uaip-api".to_string(),
        604800, // 7 days
    );

    let refresh_token = refresh_jwt_manager
        .generate_token(&agent.id.to_string(), &agent.client_id, scopes.clone(), None)
        .map_err(|e| {
            tracing::error!("Failed to generate refresh token: {}", e);
            UaipError::InternalError("Failed to generate refresh token".to_string())
        })?;

    // Update last_authenticated timestamp
    sqlx::query(
        "UPDATE ai_agents SET last_authenticated = NOW() WHERE id = $1",
    )
    .bind(&agent.id)
    .execute(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update last_authenticated: {}", e);
        // Don't fail authentication for this
    })
    .ok();

    // Log successful authentication
    log_audit_event(
        db_pool,
        &agent.id.to_string(),
        "ai_agent",
        "login",
        true,
        None,
    )
    .await;

    tracing::info!(
        "Successful authentication for client_id: {} ({})",
        request.client_id,
        agent.name
    );

    Ok(Json(LoginResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        scope: Some(scopes.join(" ")),
        refresh_token: Some(refresh_token),
    }))
}

/// Log an audit event
async fn log_audit_event(
    pool: &sqlx::PgPool,
    entity_id: &str,
    entity_type: &str,
    action: &str,
    success: bool,
    error_message: Option<&str>,
) {
    sqlx::query(
        "INSERT INTO audit_log (entity_id, entity_type, action, success, error_message, metadata)
         VALUES ($1, $2, $3, $4, $5, '{}')",
    )
    .bind(entity_id)
    .bind(entity_type)
    .bind(action)
    .bind(success)
    .bind(error_message)
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to log audit event: {}", e);
    })
    .ok();
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[tokio::test]
    async fn test_login_missing_client_id() {
        let state = Arc::new(AppState::new());
        let request = LoginRequest {
            grant_type: "client_credentials".to_string(),
            client_id: "".to_string(),
            client_secret: Some("test_secret".to_string()),
            scope: None,
        };

        let result = login(State(state), Json(request)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_login_missing_client_secret() {
        let state = Arc::new(AppState::new());
        let request = LoginRequest {
            grant_type: "client_credentials".to_string(),
            client_id: "test_client".to_string(),
            client_secret: None,
            scope: None,
        };

        let result = login(State(state), Json(request)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_login_no_database() {
        let state = Arc::new(AppState::new());
        let request = LoginRequest {
            grant_type: "client_credentials".to_string(),
            client_id: "test_client".to_string(),
            client_secret: Some("test_secret".to_string()),
            scope: Some("device:read".to_string()),
        };

        let result = login(State(state), Json(request)).await;
        assert!(result.is_err());
        // Should fail with "Database not configured"
    }
}
