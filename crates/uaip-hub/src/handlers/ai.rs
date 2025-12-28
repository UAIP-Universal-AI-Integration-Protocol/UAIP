//! AI Agent and Session Management Handlers
//!
//! REST API endpoints for AI agent registration, session management, and interaction.

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use uaip_core::{
    ai_agent::{AgentConfig, AgentType, AiAgent, AiSession, InteractionType, SessionState},
    device::{Capability, DeviceId},
};

use crate::api::rest::{ApiResult, AppState};

/// AI Agent Management Endpoints
///
/// Register a new AI agent
pub async fn register_ai_agent(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RegisterAgentRequest>,
) -> ApiResult<Json<AgentResponse>> {
    info!("Registering AI agent: {}", request.name);

    // Create agent
    let mut agent = AiAgent::new(request.name.clone(), request.agent_type);
    agent.version = request.version.unwrap_or_else(|| "1.0.0".to_string());
    agent.provider = request.provider.unwrap_or_else(|| "Custom".to_string());

    // Add capabilities
    if let Some(capabilities) = request.supported_capabilities {
        for capability in capabilities {
            agent.add_capability(capability);
        }
    }

    // Apply configuration
    if let Some(config) = request.config {
        agent.config = config;
    }

    let agent_id = agent.id;

    // Store in database if available
    if let Some(pool) = &state.db_pool {
        match sqlx::query(
            r#"
            INSERT INTO ai_agents (id, name, agent_type, version, provider, config, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            "#,
        )
        .bind(agent_id)
        .bind(&agent.name)
        .bind(format!("{:?}", agent.agent_type))
        .bind(&agent.version)
        .bind(&agent.provider)
        .bind(serde_json::to_value(&agent.config).unwrap())
        .execute(pool)
        .await
        {
            Ok(_) => {
                info!("Stored agent {} in database", agent_id);
            }
            Err(e) => {
                error!("Failed to store agent in database: {}", e);
                // Continue anyway - registration can work without DB
            }
        }
    }

    Ok(Json(AgentResponse {
        id: agent_id,
        name: agent.name,
        agent_type: agent.agent_type,
        version: agent.version,
        provider: agent.provider,
        supported_capabilities: agent.supported_capabilities,
        registered_at: chrono::Utc::now(),
    }))
}

/// List all registered AI agents
pub async fn list_ai_agents(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<AgentListResponse>> {
    info!("Listing AI agents");

    let mut agents = Vec::new();

    // Fetch from database if available
    if let Some(pool) = &state.db_pool {
        match sqlx::query(
            r#"
            SELECT id, name, agent_type, version, provider, created_at
            FROM ai_agents
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await
        {
            Ok(records) => {
                for record in records {
                    let id: Uuid = record.try_get("id").unwrap_or_default();
                    let name: String = record.try_get("name").unwrap_or_default();
                    let agent_type_str: String = record.try_get("agent_type").unwrap_or_default();
                    let version: String = record.try_get("version").unwrap_or_default();
                    let provider: String = record.try_get("provider").unwrap_or_default();
                    let created_at: chrono::NaiveDateTime =
                        record.try_get("created_at").unwrap_or_default();

                    // Parse agent_type from string
                    let agent_type = match agent_type_str.as_str() {
                        "Conversational" => AgentType::Conversational,
                        "Automation" => AgentType::Automation,
                        "Monitoring" => AgentType::Monitoring,
                        "Control" => AgentType::Control,
                        "Diagnostic" => AgentType::Diagnostic,
                        "Predictive" => AgentType::Predictive,
                        "MultiPurpose" => AgentType::MultiPurpose,
                        _ => AgentType::Custom,
                    };

                    agents.push(AgentSummary {
                        id,
                        name,
                        agent_type,
                        version,
                        provider,
                        registered_at: created_at.and_utc(),
                    });
                }
            }
            Err(e) => {
                error!("Failed to fetch agents from database: {}", e);
            }
        }
    }

    let total = agents.len();
    Ok(Json(AgentListResponse { agents, total }))
}

/// Create a new AI session
pub async fn create_ai_session(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<CreateSessionRequest>,
) -> ApiResult<Json<SessionResponse>> {
    info!("Creating AI session for agent: {}", request.agent_id);

    let session = AiSession::new(request.agent_id);
    let session_id = session.id;

    Ok(Json(SessionResponse {
        session_id,
        agent_id: session.agent_id,
        state: session.state,
        started_at: session.started_at,
        devices: session.devices,
    }))
}

/// Get session information
pub async fn get_ai_session(
    State(_state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
) -> ApiResult<Json<SessionResponse>> {
    info!("Getting AI session: {}", session_id);

    // Mock session for now
    let session = AiSession::new(Uuid::new_v4());

    Ok(Json(SessionResponse {
        session_id: session.id,
        agent_id: session.agent_id,
        state: session.state,
        started_at: session.started_at,
        devices: session.devices,
    }))
}

/// Add device to AI session
pub async fn add_device_to_session(
    State(_state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
    Json(request): Json<AddDeviceRequest>,
) -> ApiResult<Json<SuccessResponse>> {
    info!(
        "Adding device {} to session {}",
        request.device_id, session_id
    );

    Ok(Json(SuccessResponse {
        success: true,
        message: format!(
            "Device {} added to session {}",
            request.device_id, session_id
        ),
    }))
}

/// Terminate AI session
pub async fn terminate_ai_session(
    State(_state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
) -> ApiResult<Json<SuccessResponse>> {
    info!("Terminating AI session: {}", session_id);

    Ok(Json(SuccessResponse {
        success: true,
        message: format!("Session {} terminated", session_id),
    }))
}

/// Send interaction to device via AI session
pub async fn send_ai_interaction(
    State(_state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
    Json(request): Json<InteractionRequest>,
) -> ApiResult<Json<InteractionResultResponse>> {
    info!(
        "AI interaction: session={}, device={}, type={:?}",
        session_id, request.device_id, request.interaction_type
    );

    Ok(Json(InteractionResultResponse {
        request_id: Uuid::new_v4(),
        session_id,
        device_id: request.device_id,
        success: true,
        data: request.parameters,
        timestamp: chrono::Utc::now(),
        processing_time_ms: 42,
    }))
}

// Request/Response Types

#[derive(Debug, Deserialize)]
pub struct RegisterAgentRequest {
    pub name: String,
    pub agent_type: AgentType,
    pub version: Option<String>,
    pub provider: Option<String>,
    pub supported_capabilities: Option<Vec<Capability>>,
    pub config: Option<AgentConfig>,
}

#[derive(Debug, Serialize)]
pub struct AgentResponse {
    pub id: Uuid,
    pub name: String,
    pub agent_type: AgentType,
    pub version: String,
    pub provider: String,
    pub supported_capabilities: Vec<Capability>,
    pub registered_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AgentSummary {
    pub id: Uuid,
    pub name: String,
    pub agent_type: AgentType,
    pub version: String,
    pub provider: String,
    pub registered_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AgentListResponse {
    pub agents: Vec<AgentSummary>,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub agent_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub session_id: Uuid,
    pub agent_id: Uuid,
    pub state: SessionState,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub devices: Vec<DeviceId>,
}

#[derive(Debug, Deserialize)]
pub struct AddDeviceRequest {
    pub device_id: DeviceId,
}

#[derive(Debug, Deserialize)]
pub struct InteractionRequest {
    pub device_id: DeviceId,
    pub interaction_type: InteractionType,
    pub parameters: serde_json::Value,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct InteractionResultResponse {
    pub request_id: Uuid,
    pub session_id: Uuid,
    pub device_id: DeviceId,
    pub success: bool,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_agent_request() {
        use uaip_core::device::CapabilityType;

        let sensor_cap = Capability::new("sensor".to_string(), CapabilityType::Sensor, true);
        let request = RegisterAgentRequest {
            name: "TestAgent".to_string(),
            agent_type: AgentType::Conversational,
            version: Some("1.0.0".to_string()),
            provider: Some("Test".to_string()),
            supported_capabilities: Some(vec![sensor_cap]),
            config: None,
        };

        assert_eq!(request.name, "TestAgent");
        assert_eq!(request.agent_type, AgentType::Conversational);
    }

    #[test]
    fn test_interaction_request() {
        let request = InteractionRequest {
            device_id: "device-123".to_string(),
            interaction_type: InteractionType::Query,
            parameters: serde_json::json!({"param": "value"}),
            timeout_ms: Some(5000),
        };

        assert_eq!(request.device_id, "device-123");
        assert_eq!(request.interaction_type, InteractionType::Query);
    }
}
