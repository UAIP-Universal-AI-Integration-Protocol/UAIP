//! AI Session Management
//!
//! Manages AI agent sessions, tracks active connections, and handles session lifecycle.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use uaip_core::{
    ai_agent::{AiAgent, AiSession, SessionState},
    device::DeviceId,
    error::{Result, UaipError},
};

/// AI Session Manager
pub struct AiSessionManager {
    /// Active sessions indexed by session ID
    sessions: Arc<RwLock<HashMap<Uuid, AiSession>>>,

    /// Agent registry indexed by agent ID
    agents: Arc<RwLock<HashMap<Uuid, AiAgent>>>,

    /// Session timeout in seconds
    session_timeout_secs: u64,

    /// Maximum sessions per agent
    max_sessions_per_agent: usize,
}

impl AiSessionManager {
    /// Create a new session manager
    pub fn new(session_timeout_secs: u64, max_sessions_per_agent: usize) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            agents: Arc::new(RwLock::new(HashMap::new())),
            session_timeout_secs,
            max_sessions_per_agent,
        }
    }

    /// Register a new AI agent
    pub async fn register_agent(&self, agent: AiAgent) -> Result<Uuid> {
        let agent_id = agent.id;
        let mut agents = self.agents.write().await;

        if agents.contains_key(&agent_id) {
            return Err(UaipError::InvalidParameter(format!(
                "Agent {} already registered",
                agent_id
            )));
        }

        info!(
            "Registering AI agent: {} (type: {:?})",
            agent.name, agent.agent_type
        );

        agents.insert(agent_id, agent);
        Ok(agent_id)
    }

    /// Unregister an AI agent
    pub async fn unregister_agent(&self, agent_id: &Uuid) -> Result<()> {
        let mut agents = self.agents.write().await;

        if agents.remove(agent_id).is_none() {
            return Err(UaipError::InvalidParameter(format!(
                "Agent {} not found",
                agent_id
            )));
        }

        // Terminate all sessions for this agent
        self.terminate_agent_sessions(agent_id).await?;

        info!("Unregistered AI agent: {}", agent_id);
        Ok(())
    }

    /// Get agent information
    pub async fn get_agent(&self, agent_id: &Uuid) -> Option<AiAgent> {
        self.agents.read().await.get(agent_id).cloned()
    }

    /// List all registered agents
    pub async fn list_agents(&self) -> Vec<AiAgent> {
        self.agents.read().await.values().cloned().collect()
    }

    /// Create a new session for an agent
    pub async fn create_session(&self, agent_id: Uuid) -> Result<Uuid> {
        // Verify agent exists
        let agents = self.agents.read().await;
        if !agents.contains_key(&agent_id) {
            return Err(UaipError::InvalidParameter(format!(
                "Agent {} not found",
                agent_id
            )));
        }
        drop(agents);

        // Check session limit for this agent
        let sessions = self.sessions.read().await;
        let agent_session_count = sessions
            .values()
            .filter(|s| s.agent_id == agent_id && s.state == SessionState::Active)
            .count();

        if agent_session_count >= self.max_sessions_per_agent {
            return Err(UaipError::ResourceUnavailable(format!(
                "Maximum sessions ({}) reached for agent {}",
                self.max_sessions_per_agent, agent_id
            )));
        }
        drop(sessions);

        // Create new session
        let session = AiSession::new(agent_id);
        let session_id = session.id;

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id, session);

        info!("Created session {} for agent {}", session_id, agent_id);
        Ok(session_id)
    }

    /// Get session information
    pub async fn get_session(&self, session_id: &Uuid) -> Option<AiSession> {
        self.sessions.read().await.get(session_id).cloned()
    }

    /// Add device to session
    pub async fn add_device_to_session(
        &self,
        session_id: &Uuid,
        device_id: DeviceId,
    ) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        let session = sessions.get_mut(session_id).ok_or_else(|| {
            UaipError::InvalidParameter(format!("Session {} not found", session_id))
        })?;

        if session.state != SessionState::Active {
            return Err(UaipError::InvalidState(format!(
                "Session {} is not active (state: {:?})",
                session_id, session.state
            )));
        }

        session.add_device(device_id.clone());
        debug!(
            "Added device {} to session {}",
            device_id, session_id
        );

        Ok(())
    }

    /// Remove device from session
    pub async fn remove_device_from_session(
        &self,
        session_id: &Uuid,
        device_id: &DeviceId,
    ) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        let session = sessions.get_mut(session_id).ok_or_else(|| {
            UaipError::InvalidParameter(format!("Session {} not found", session_id))
        })?;

        session.remove_device(device_id);
        debug!(
            "Removed device {} from session {}",
            device_id, session_id
        );

        Ok(())
    }

    /// Update session activity timestamp
    pub async fn update_session_activity(&self, session_id: &Uuid) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        let session = sessions.get_mut(session_id).ok_or_else(|| {
            UaipError::InvalidParameter(format!("Session {} not found", session_id))
        })?;

        session.update_activity();
        Ok(())
    }

    /// Terminate a session
    pub async fn terminate_session(&self, session_id: &Uuid) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        let session = sessions.get_mut(session_id).ok_or_else(|| {
            UaipError::InvalidParameter(format!("Session {} not found", session_id))
        })?;

        session.state = SessionState::Terminated;
        info!("Terminated session {}", session_id);

        Ok(())
    }

    /// Terminate all sessions for an agent
    async fn terminate_agent_sessions(&self, agent_id: &Uuid) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        let mut terminated_count = 0;
        for session in sessions.values_mut() {
            if session.agent_id == *agent_id && session.state == SessionState::Active {
                session.state = SessionState::Terminated;
                terminated_count += 1;
            }
        }

        info!(
            "Terminated {} sessions for agent {}",
            terminated_count, agent_id
        );

        Ok(())
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> usize {
        let mut sessions = self.sessions.write().await;

        let expired_ids: Vec<Uuid> = sessions
            .iter()
            .filter(|(_, session)| session.is_expired(self.session_timeout_secs))
            .map(|(id, _)| *id)
            .collect();

        for id in &expired_ids {
            sessions.remove(id);
            debug!("Removed expired session {}", id);
        }

        let count = expired_ids.len();
        if count > 0 {
            info!("Cleaned up {} expired sessions", count);
        }

        count
    }

    /// Get active session count for an agent
    pub async fn get_agent_active_sessions(&self, agent_id: &Uuid) -> usize {
        self.sessions
            .read()
            .await
            .values()
            .filter(|s| s.agent_id == *agent_id && s.state == SessionState::Active)
            .count()
    }

    /// Get total active sessions
    pub async fn get_active_sessions_count(&self) -> usize {
        self.sessions
            .read()
            .await
            .values()
            .filter(|s| s.state == SessionState::Active)
            .count()
    }

    /// Get session statistics
    pub async fn get_session_stats(&self, session_id: &Uuid) -> Option<uaip_core::ai_agent::SessionStats> {
        self.sessions
            .read()
            .await
            .get(session_id)
            .map(|s| s.stats.clone())
    }

    /// List sessions for an agent
    pub async fn list_agent_sessions(&self, agent_id: &Uuid) -> Vec<AiSession> {
        self.sessions
            .read()
            .await
            .values()
            .filter(|s| s.agent_id == *agent_id)
            .cloned()
            .collect()
    }
}

impl Default for AiSessionManager {
    fn default() -> Self {
        Self::new(3600, 10) // 1 hour timeout, max 10 sessions per agent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uaip_core::ai_agent::AgentType;

    #[tokio::test]
    async fn test_register_agent() {
        let manager = AiSessionManager::default();
        let agent = AiAgent::new("TestAgent".to_string(), AgentType::Conversational);
        let agent_id = agent.id;

        let result = manager.register_agent(agent).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), agent_id);

        let retrieved = manager.get_agent(&agent_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "TestAgent");
    }

    #[tokio::test]
    async fn test_create_session() {
        let manager = AiSessionManager::default();
        let agent = AiAgent::new("TestAgent".to_string(), AgentType::Conversational);
        let agent_id = agent.id;

        manager.register_agent(agent).await.unwrap();

        let result = manager.create_session(agent_id).await;
        assert!(result.is_ok());

        let session_id = result.unwrap();
        let session = manager.get_session(&session_id).await;
        assert!(session.is_some());
        assert_eq!(session.unwrap().agent_id, agent_id);
    }

    #[tokio::test]
    async fn test_session_device_management() {
        let manager = AiSessionManager::default();
        let agent = AiAgent::new("TestAgent".to_string(), AgentType::Control);
        let agent_id = agent.id;

        manager.register_agent(agent).await.unwrap();
        let session_id = manager.create_session(agent_id).await.unwrap();

        let device_id = "device-123".to_string();
        manager
            .add_device_to_session(&session_id, device_id.clone())
            .await
            .unwrap();

        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.devices.len(), 1);
        assert!(session.devices.contains(&device_id));

        manager
            .remove_device_from_session(&session_id, &device_id)
            .await
            .unwrap();

        let session = manager.get_session(&session_id).await.unwrap();
        assert!(session.devices.is_empty());
    }

    #[tokio::test]
    async fn test_terminate_session() {
        let manager = AiSessionManager::default();
        let agent = AiAgent::new("TestAgent".to_string(), AgentType::Monitoring);
        let agent_id = agent.id;

        manager.register_agent(agent).await.unwrap();
        let session_id = manager.create_session(agent_id).await.unwrap();

        manager.terminate_session(&session_id).await.unwrap();

        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.state, SessionState::Terminated);
    }

    #[tokio::test]
    async fn test_list_agents() {
        let manager = AiSessionManager::default();

        let agent1 = AiAgent::new("Agent1".to_string(), AgentType::Conversational);
        let agent2 = AiAgent::new("Agent2".to_string(), AgentType::Automation);

        manager.register_agent(agent1).await.unwrap();
        manager.register_agent(agent2).await.unwrap();

        let agents = manager.list_agents().await;
        assert_eq!(agents.len(), 2);
    }

    #[tokio::test]
    async fn test_session_limit() {
        let manager = AiSessionManager::new(3600, 2); // Max 2 sessions
        let agent = AiAgent::new("TestAgent".to_string(), AgentType::Control);
        let agent_id = agent.id;

        manager.register_agent(agent).await.unwrap();

        // Create first session - should succeed
        assert!(manager.create_session(agent_id).await.is_ok());

        // Create second session - should succeed
        assert!(manager.create_session(agent_id).await.is_ok());

        // Create third session - should fail (limit reached)
        let result = manager.create_session(agent_id).await;
        assert!(result.is_err());
    }
}
