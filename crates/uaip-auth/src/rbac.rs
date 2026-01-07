//! Role-Based Access Control (RBAC)
//!
//! This module provides role-based access control for UAIP resources.

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::collections::{HashMap, HashSet};
use uaip_core::error::{Result, UaipError};

/// Permission represents a specific action on a resource
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// Resource type (e.g., "device", "telemetry", "command")
    pub resource: String,
    /// Action (e.g., "read", "write", "execute", "delete")
    pub action: String,
}

impl Permission {
    /// Create a new permission
    pub fn new(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
        }
    }

    /// Parse from colon-separated string (e.g., "device:read")
    pub fn parse(perm_str: &str) -> Result<Self> {
        let parts: Vec<&str> = perm_str.split(':').collect();
        if parts.len() != 2 {
            return Err(UaipError::InvalidParameter(format!(
                "Invalid permission format: {}",
                perm_str
            )));
        }

        Ok(Self {
            resource: parts[0].to_string(),
            action: parts[1].to_string(),
        })
    }

    /// Convert to string representation
    pub fn to_string_repr(&self) -> String {
        format!("{}:{}", self.resource, self.action)
    }

    /// Check if this permission matches a wildcard permission
    pub fn matches(&self, pattern: &Permission) -> bool {
        let resource_match = pattern.resource == "*" || pattern.resource == self.resource;
        let action_match = pattern.action == "*" || pattern.action == self.action;
        resource_match && action_match
    }
}

/// Role represents a collection of permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Role ID (UUID)
    pub id: Option<String>,
    /// Role name
    pub name: String,
    /// Role description
    pub description: String,
    /// Set of permissions
    pub permissions: HashSet<Permission>,
}

impl Role {
    /// Create a new role
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            description: description.into(),
            permissions: HashSet::new(),
        }
    }

    /// Add a permission to this role
    pub fn add_permission(mut self, permission: Permission) -> Self {
        self.permissions.insert(permission);
        self
    }

    /// Add multiple permissions
    pub fn add_permissions(mut self, permissions: Vec<Permission>) -> Self {
        self.permissions.extend(permissions);
        self
    }

    /// Check if role has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.iter().any(|p| permission.matches(p))
    }
}

/// RBAC manager for checking permissions
pub struct RbacManager {
    /// Database connection pool (optional, if using DB-backed RBAC)
    pool: Option<Pool<Postgres>>,
    /// In-memory role definitions (fallback/cache)
    roles: HashMap<String, Role>,
    /// In-memory assignments (fallback/cache)
    assignments: HashMap<String, HashSet<String>>,
}

impl RbacManager {
    /// Create a new in-memory RBAC manager
    pub fn new() -> Self {
        Self {
            pool: None,
            roles: HashMap::new(),
            assignments: HashMap::new(),
        }
    }

    /// Create a new DB-backed RBAC manager
    pub fn new_with_db(pool: Pool<Postgres>) -> Self {
        Self {
            pool: Some(pool),
            roles: HashMap::new(),
            assignments: HashMap::new(),
        }
    }

    /// Register a new role (In-memory only, use DB migrations for persistent roles)
    pub fn register_role(&mut self, role: Role) {
        self.roles.insert(role.name.clone(), role);
    }

    /// Assign a role to a user/agent
    pub async fn assign_role(
        &mut self,
        entity_id: &str,
        role_name: &str,
        entity_type: &str, // 'device' or 'ai_agent'
    ) -> Result<()> {
        if let Some(pool) = &self.pool {
            // DB implementation
            let role_record = sqlx::query!("SELECT id FROM roles WHERE name = $1", role_name)
                .fetch_optional(pool)
                .await
                .map_err(|e| UaipError::DatabaseError(e.to_string()))?;

            let role_id = match role_record {
                Some(record) => record.id,
                None => {
                    return Err(UaipError::InvalidParameter(format!(
                        "Role '{}' does not exist",
                        role_name
                    )))
                }
            };
            
            // Assuming entity_id is a UUID string
            let entity_uuid = uuid::Uuid::parse_str(entity_id)
                .map_err(|e| UaipError::InvalidParameter(format!("Invalid entity UUID: {}", e)))?;

            sqlx::query!(
                "INSERT INTO entity_roles (entity_id, entity_type, role_id) VALUES ($1, $2, $3) ON CONFLICT (entity_id, entity_type, role_id) DO NOTHING",
                entity_uuid,
                entity_type,
                role_id
            )
            .execute(pool)
            .await
            .map_err(|e| UaipError::DatabaseError(e.to_string()))?;

            Ok(())
        } else {
            // In-memory fallback
            if !self.roles.contains_key(role_name) {
                return Err(UaipError::InvalidParameter(format!(
                    "Role '{}' does not exist",
                    role_name
                )));
            }
            self.assignments
                .entry(entity_id.to_string())
                .or_default()
                .insert(role_name.to_string());
            Ok(())
        }
    }
    
    /// Revoke a role from a user/agent
    pub async fn revoke_role(&mut self, entity_id: &str, role_name: &str, entity_type: &str) -> Result<()> {
        if let Some(pool) = &self.pool {
             let role_record = sqlx::query!("SELECT id FROM roles WHERE name = $1", role_name)
                .fetch_optional(pool)
                .await
                .map_err(|e| UaipError::DatabaseError(e.to_string()))?;

             if let Some(record) = role_record {
                  let entity_uuid = uuid::Uuid::parse_str(entity_id)
                    .map_err(|e| UaipError::InvalidParameter(format!("Invalid entity UUID: {}", e)))?;
                    
                  sqlx::query!(
                    "DELETE FROM entity_roles WHERE entity_id = $1 AND role_id = $2 AND entity_type = $3",
                    entity_uuid,
                    record.id,
                    entity_type
                  )
                  .execute(pool)
                  .await
                  .map_err(|e| UaipError::DatabaseError(e.to_string()))?;
             }
             Ok(())
        } else {
             if let Some(roles) = self.assignments.get_mut(entity_id) {
                roles.remove(role_name);
            }
            Ok(())
        }
    }

    /// Check if entity has a specific permission
    pub async fn has_permission(&self, entity_id: &str, permission: &Permission) -> Result<bool> {
        if let Some(pool) = &self.pool {
            // Using the DB function has_permission
            let entity_uuid = uuid::Uuid::parse_str(entity_id)
                .map_err(|e| UaipError::InvalidParameter(format!("Invalid entity UUID: {}", e)))?;

             // Allow checking either ai_agent or device implicitly or we assume ai_agent primarily
             // For now, let's try both or rely on the caller passing correct context.
             // But the signature doesn't take type. Let's start by checking as 'ai_agent' first.
             let has_perm = sqlx::query!(
                "SELECT has_permission($1, 'ai_agent', $2, $3) as allowed",
                entity_uuid,
                permission.resource,
                permission.action
             )
             .fetch_one(pool)
             .await
             .map_err(|e| UaipError::DatabaseError(e.to_string()))?;
             
             Ok(has_perm.allowed.unwrap_or(false))

        } else {
            // In-memory fallback
            if let Some(role_names) = self.assignments.get(entity_id) {
               for name in role_names {
                   if let Some(role) = self.roles.get(name) {
                       if role.has_permission(permission) {
                           return Ok(true);
                       }
                   }
               }
            }
            Ok(false)
        }
    }
    
    /// Check permission from string
    pub async fn check_permission(&self, entity_id: &str, permission_str: &str) -> Result<bool> {
        let permission = Permission::parse(permission_str)?;
        self.has_permission(entity_id, &permission).await
    }
}

impl Default for RbacManager {
    fn default() -> Self {
        Self::new()
    }
}

