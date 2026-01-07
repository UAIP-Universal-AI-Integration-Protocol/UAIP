use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use sqlx::Row;
use uaip_core::error::UaipError;
use crate::api::rest::{ApiResult, AppState};

/// User registration request
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: String,
}

/// User info response
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserInfo {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
    pub active: bool,
    #[sqlx(default)]
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// List users response
#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserInfo>,
    pub total: usize,
}

/// Create a new user (Human)
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateUserRequest>,
) -> ApiResult<Json<UserInfo>> {
    let db_pool = state.db_pool.as_ref().ok_or_else(|| {
        UaipError::InternalError("Database not configured".to_string())
    })?;

    // Check if user already exists
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
        .bind(&request.email)
        .fetch_one(db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking user existence: {}", e);
            UaipError::InternalError("Failed to check user existence".to_string())
        })?;

    if exists {
        return Err(UaipError::InvalidParameter("User with this email already exists".to_string()).into());
    }

    // Hash password
    let password_hash = bcrypt::hash(&request.password, bcrypt::DEFAULT_COST).map_err(|e| {
        tracing::error!("Password hashing error: {}", e);
        UaipError::InternalError("Failed to secure password".to_string())
    })?;

    // Create user
    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    
    // Normalize user role
    let role = match request.role.as_str() {
        "admin" | "operator" | "viewer" => request.role.clone(),
        _ => "viewer".to_string() 
    };

    let mut transaction = db_pool.begin().await.map_err(|e| {
        tracing::error!("Failed to start transaction: {}", e);
        UaipError::InternalError("Database error".to_string())
    })?;

    // 1. Insert into users
    sqlx::query(
        r#"
        INSERT INTO users (id, email, name, password_hash, role, active, created_at, require_password_change)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#
    )
    .bind(user_id)
    .bind(&request.email)
    .bind(&request.name)
    .bind(&password_hash)
    .bind(&role)
    .bind(true) // active
    .bind(now)
    .bind(true) // Force password change on first login
    .execute(&mut *transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create user: {}", e);
        UaipError::InternalError("Failed to create user".to_string())
    })?;

    // 2. Assign Role in RBAC system (Legacy/Parallel RBAC sync)
    // Map simplified roles to actual RBAC role names if they differ
    let db_role_name = match role.as_str() {
        "admin" => "admin",
        "operator" => "device_manager", 
        "viewer" => "monitor",
        _ => "monitor"
    };

    let role_record = sqlx::query("SELECT id FROM roles WHERE name = $1")
        .bind(db_role_name)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|e| {
             tracing::error!("Failed to fetch role: {}", e);
             UaipError::InternalError("Failed to assign role".to_string())
        })?;

    if let Some(record) = role_record {
        let role_id: Uuid = record.try_get("id").unwrap();
        sqlx::query(
            "INSERT INTO entity_roles (entity_id, entity_type, role_id) VALUES ($1, 'user', $2)",
        )
        .bind(user_id)
        .bind(role_id)
        .execute(&mut *transaction)
        .await
        .map_err(|e| {
             tracing::error!("Failed to assign entity role: {}", e);
             UaipError::InternalError("Failed to assign role".to_string())
        })?;
    }
    
    transaction.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        UaipError::InternalError("Database error".to_string())
    })?;

    let user_info = UserInfo {
        id: user_id,
        name: request.name,
        email: request.email,
        role: role,
        active: true,
        last_login: None,
        created_at: now,
    };

    Ok(Json(user_info))
}

/// List all users
pub async fn list_users(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<UserListResponse>> {
    let db_pool = state.db_pool.as_ref().ok_or_else(|| {
        UaipError::InternalError("Database not configured".to_string())
    })?;

    // Fetch users directly from users table
    let users = sqlx::query_as::<_, UserInfo>(
        r#"
        SELECT id, name, email, role, active, last_login, created_at 
        FROM users 
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch users: {}", e);
        UaipError::InternalError("Failed to fetch users".to_string())
    })?;

    let total = users.len();
    Ok(Json(UserListResponse { users, total }))
}

/// Delete a user
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<bool>> {
    let db_pool = state.db_pool.as_ref().ok_or_else(|| {
        UaipError::InternalError("Database not configured".to_string())
    })?;

    // Cascade delete handles entity_roles usually, but let's be explicit if FK not set up for that table (it likely is)
    // Actually entity_roles might not have FK constraint to users yet because users is new.
    // So we should delete from entity_roles first.
    let mut transaction = db_pool.begin().await.map_err(|_e| {
        UaipError::InternalError("Database error".to_string())
    })?;

    sqlx::query("DELETE FROM entity_roles WHERE entity_id = $1")
        .bind(id)
        .execute(&mut *transaction)
        .await
        .ok(); // Ignore if fails or doesn't exist

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&mut *transaction)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete user: {}", e);
            UaipError::InternalError("Failed to delete user".to_string())
        })?;

    if result.rows_affected() == 0 {
        return Err(UaipError::DeviceNotFound("User not found".to_string()).into());
    }
    
    transaction.commit().await.map_err(|_| UaipError::InternalError("Database error".to_string()))?;

    Ok(Json(true))
}

/// Update user status
#[derive(Debug, Deserialize)]
pub struct UpdateUserStatusRequest {
    pub active: bool,
}

pub async fn update_user_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateUserStatusRequest>,
) -> ApiResult<Json<bool>> {
    let db_pool = state.db_pool.as_ref().ok_or_else(|| {
        UaipError::InternalError("Database not configured".to_string())
    })?;

    let result = sqlx::query("UPDATE users SET active = $1 WHERE id = $2")
        .bind(request.active)
        .bind(id)
        .execute(db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update user status: {}", e);
            UaipError::InternalError("Failed to update user status".to_string())
        })?;

    if result.rows_affected() == 0 {
         return Err(UaipError::DeviceNotFound("User not found".to_string()).into());
    }

    Ok(Json(true))
}

/// Update user request
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub role: Option<String>,
    pub active: Option<bool>,
}

/// Update user handler
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> ApiResult<Json<bool>> {
    let db_pool = state.db_pool.as_ref().ok_or_else(|| {
        UaipError::InternalError("Database not configured".to_string())
    })?;

    let mut transaction = db_pool.begin().await.map_err(|e| {
        tracing::error!("Failed to start transaction: {}", e);
        UaipError::InternalError("Database error".to_string())
    })?;

    // 1. Update basic info in users
    if let Some(name) = &request.name {
        sqlx::query("UPDATE users SET name = $1 WHERE id = $2")
            .bind(name)
            .bind(id)
            .execute(&mut *transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update user name: {}", e);
                UaipError::InternalError("Failed to update user".to_string())
            })?;
    }
    
    if let Some(active) = request.active {
            sqlx::query("UPDATE users SET active = $1 WHERE id = $2")
            .bind(active)
            .bind(id)
            .execute(&mut *transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update user status: {}", e);
                UaipError::InternalError("Failed to update user status".to_string())
            })?;
    }

    // 2. Update Role if provided
    if let Some(role) = &request.role {
        // Update role in users table
        sqlx::query("UPDATE users SET role = $1 WHERE id = $2")
            .bind(role)
            .bind(id)
            .execute(&mut *transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update user role: {}", e);
                UaipError::InternalError("Failed to update user role".to_string())
            })?;

        // Update RBAC mapping
        let db_role_name = match role.as_str() {
            "admin" => "admin",
            "operator" => "device_manager",
            "viewer" => "monitor",
            _ => "monitor"
        };

        let role_record = sqlx::query("SELECT id FROM roles WHERE name = $1")
            .bind(db_role_name)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|e| {
                 tracing::error!("Failed to fetch role: {}", e);
                 UaipError::InternalError("Failed to update user role".to_string())
            })?;
            
        if let Some(record) = role_record {
            let role_id: Uuid = record.try_get("id").unwrap();
            
            // Remove old roles
            sqlx::query("DELETE FROM entity_roles WHERE entity_id = $1")
                .bind(id)
                .execute(&mut *transaction)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to clear old roles: {}", e);
                    UaipError::InternalError("Failed to update user role".to_string())
                })?;

            // Add new role
            sqlx::query(
                "INSERT INTO entity_roles (entity_id, entity_type, role_id) VALUES ($1, 'user', $2)",
            )
            .bind(id)
            .bind(role_id)
            .execute(&mut *transaction)
            .await
            .map_err(|e| {
                 tracing::error!("Failed to assign new role: {}", e);
                 UaipError::InternalError("Failed to update user role".to_string())
            })?;
        }
    }

    transaction.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        UaipError::InternalError("Database error".to_string())
    })?;

    Ok(Json(true))
}

/// Admin reset password request
#[derive(Debug, Deserialize)]
pub struct AdminResetPasswordRequest {
    pub new_password: String,
    pub require_change: bool,
}

/// Admin reset password handler
pub async fn admin_reset_password(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<AdminResetPasswordRequest>,
) -> ApiResult<Json<bool>> {
    let db_pool = state.db_pool.as_ref().ok_or_else(|| {
        UaipError::InternalError("Database not configured".to_string())
    })?;

    // Hash new password
    if request.new_password.len() < 8 {
         return Err(UaipError::InvalidParameter("Password must be at least 8 characters".to_string()).into());
    }

    let password_hash = bcrypt::hash(&request.new_password, bcrypt::DEFAULT_COST).map_err(|e| {
        tracing::error!("Password hashing error: {}", e);
        UaipError::InternalError("Failed to secure password".to_string())
    })?;

    let result = sqlx::query("UPDATE users SET password_hash = $1, require_password_change = $2 WHERE id = $3")
        .bind(password_hash)
        .bind(request.require_change)
        .bind(id)
        .execute(db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to reset password: {}", e);
            UaipError::InternalError("Failed to reset password".to_string())
        })?;

    if result.rows_affected() == 0 {
         return Err(UaipError::DeviceNotFound("User not found".to_string()).into());
    }

    Ok(Json(true))
}
