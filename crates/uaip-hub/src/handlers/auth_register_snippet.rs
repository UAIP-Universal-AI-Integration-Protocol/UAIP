/// Register handler (Public)
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RegisterRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let db_pool = state.db_pool.as_ref().ok_or_else(|| {
        UaipError::InternalError("Database not configured".to_string())
    })?;

    // 1. Validate Password Strength (Basic)
    if request.password.len() < 12 {
        return Err(UaipError::InvalidParameter("Password must be at least 12 characters".to_string()).into());
    }
    
    // 2. Check if user exists
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
        .bind(&request.email)
        .fetch_one(db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking user existence: {}", e);
            UaipError::InternalError("Registration failed".to_string())
        })?;

    if exists {
        // Return generic error or specific? For security, generic is better, but for UX specific is better. 
        // Let's use specific for now as it's an internal tool mostly.
        return Err(UaipError::InvalidParameter("Email already registered".to_string()).into());
    }

    // 3. Hash Password
    let password_hash = bcrypt::hash(&request.password, bcrypt::DEFAULT_COST).map_err(|_| {
        UaipError::InternalError("Failed to hash password".to_string())
    })?;

    // 4. Create User
    // Default Role: viewer
    // Active: true
    let user_id = uuid::Uuid::new_v4();
    let default_role = "viewer";

    // Transaction for user creation + role assignment
    let mut tx = db_pool.begin().await.map_err(|e| {
        tracing::error!("Failed to start transaction: {}", e);
        UaipError::InternalError("Registration failed".to_string())
    })?;

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, role, active, require_password_change) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(user_id)
    .bind(&request.email)
    .bind(password_hash)
    .bind(&request.name)
    .bind(default_role)
    .bind(true) // active
    .bind(false) // require_password_change (they just set it)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create user: {}", e);
        UaipError::InternalError("Registration failed".to_string())
    })?;

    // Assign Role in entity_roles
    // Get role ID
    let role_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM roles WHERE name = $1")
        .bind(default_role)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch default role id: {}", e);
            UaipError::InternalError("Registration failed".to_string())
        })?;

    sqlx::query(
        "INSERT INTO entity_roles (entity_id, entity_type, role_id) VALUES ($1, 'user', $2)"
    )
    .bind(user_id)
    .bind(role_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to assign role: {}", e);
        UaipError::InternalError("Registration failed".to_string())
    })?;

    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        UaipError::InternalError("Registration failed".to_string())
    })?;

    // 5. Auto-Login (Generate Token)
    let scopes = vec!["device:read".to_string(), "ai:read".to_string()]; // Viewer scopes

    generate_token_response(
        &user_id.to_string(),
        &request.email,
        &request.name,
        scopes,
        false,
        db_pool,
        true
    ).await
}
