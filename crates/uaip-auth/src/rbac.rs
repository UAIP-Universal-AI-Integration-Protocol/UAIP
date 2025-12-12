//! Role-Based Access Control (RBAC)
//!
//! This module provides role-based access control for UAIP resources.

use serde::{Deserialize, Serialize};
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

/// Pre-defined roles for UAIP
pub struct Roles;

impl Roles {
    /// Admin role - full access
    pub fn admin() -> Role {
        Role::new("admin", "Full system access").add_permission(Permission::new("*", "*"))
    }

    /// Device manager - can manage all devices
    pub fn device_manager() -> Role {
        Role::new("device_manager", "Can manage all devices").add_permissions(vec![
            Permission::new("device", "read"),
            Permission::new("device", "write"),
            Permission::new("device", "execute"),
            Permission::new("device", "delete"),
            Permission::new("telemetry", "read"),
            Permission::new("command", "execute"),
        ])
    }

    /// Device operator - can control devices but not delete
    pub fn device_operator() -> Role {
        Role::new("device_operator", "Can operate devices").add_permissions(vec![
            Permission::new("device", "read"),
            Permission::new("device", "execute"),
            Permission::new("telemetry", "read"),
            Permission::new("command", "execute"),
        ])
    }

    /// Monitor - read-only access
    pub fn monitor() -> Role {
        Role::new("monitor", "Read-only access").add_permissions(vec![
            Permission::new("device", "read"),
            Permission::new("telemetry", "read"),
        ])
    }

    /// AI agent - typical AI agent permissions
    pub fn ai_agent() -> Role {
        Role::new("ai_agent", "AI agent permissions").add_permissions(vec![
            Permission::new("device", "read"),
            Permission::new("device", "execute"),
            Permission::new("telemetry", "read"),
            Permission::new("command", "execute"),
            Permission::new("event", "subscribe"),
        ])
    }
}

/// RBAC manager for checking permissions
pub struct RbacManager {
    /// Role definitions
    roles: HashMap<String, Role>,
    /// User/agent role assignments
    assignments: HashMap<String, HashSet<String>>,
}

impl RbacManager {
    /// Create a new RBAC manager with default roles
    pub fn new() -> Self {
        let mut manager = Self {
            roles: HashMap::new(),
            assignments: HashMap::new(),
        };

        // Register default roles
        manager.register_role(Roles::admin());
        manager.register_role(Roles::device_manager());
        manager.register_role(Roles::device_operator());
        manager.register_role(Roles::monitor());
        manager.register_role(Roles::ai_agent());

        manager
    }

    /// Register a new role
    pub fn register_role(&mut self, role: Role) {
        self.roles.insert(role.name.clone(), role);
    }

    /// Assign a role to a user/agent
    pub fn assign_role(
        &mut self,
        entity_id: impl Into<String>,
        role_name: impl Into<String>,
    ) -> Result<()> {
        let entity_id = entity_id.into();
        let role_name = role_name.into();

        // Verify role exists
        if !self.roles.contains_key(&role_name) {
            return Err(UaipError::InvalidParameter(format!(
                "Role '{}' does not exist",
                role_name
            )));
        }

        self.assignments
            .entry(entity_id)
            .or_default()
            .insert(role_name);

        Ok(())
    }

    /// Remove a role from a user/agent
    pub fn revoke_role(&mut self, entity_id: &str, role_name: &str) {
        if let Some(roles) = self.assignments.get_mut(entity_id) {
            roles.remove(role_name);
        }
    }

    /// Get all roles for an entity
    pub fn get_entity_roles(&self, entity_id: &str) -> Vec<&Role> {
        self.assignments
            .get(entity_id)
            .map(|role_names| {
                role_names
                    .iter()
                    .filter_map(|name| self.roles.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if entity has a specific permission
    pub fn has_permission(&self, entity_id: &str, permission: &Permission) -> bool {
        let roles = self.get_entity_roles(entity_id);

        roles.iter().any(|role| role.has_permission(permission))
    }

    /// Check if entity has a specific permission (string format)
    pub fn check_permission(&self, entity_id: &str, permission_str: &str) -> Result<bool> {
        let permission = Permission::parse(permission_str)?;
        Ok(self.has_permission(entity_id, &permission))
    }

    /// Require permission or return error
    pub fn require_permission(&self, entity_id: &str, permission_str: &str) -> Result<()> {
        if self.check_permission(entity_id, permission_str)? {
            Ok(())
        } else {
            Err(UaipError::AuthorizationFailed(format!(
                "Permission '{}' denied for entity '{}'",
                permission_str, entity_id
            )))
        }
    }

    /// Get all permissions for an entity
    pub fn get_entity_permissions(&self, entity_id: &str) -> HashSet<Permission> {
        let roles = self.get_entity_roles(entity_id);
        let mut permissions = HashSet::new();

        for role in roles {
            permissions.extend(role.permissions.clone());
        }

        permissions
    }
}

impl Default for RbacManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_creation() {
        let perm = Permission::new("device", "read");
        assert_eq!(perm.resource, "device");
        assert_eq!(perm.action, "read");
    }

    #[test]
    fn test_permission_from_string() {
        let perm = Permission::parse("device:write").unwrap();
        assert_eq!(perm.resource, "device");
        assert_eq!(perm.action, "write");
    }

    #[test]
    fn test_permission_to_string() {
        let perm = Permission::new("telemetry", "read");
        assert_eq!(perm.to_string_repr(), "telemetry:read");
    }

    #[test]
    fn test_permission_wildcard_matching() {
        let specific = Permission::new("device", "read");
        let wildcard_resource = Permission::new("*", "read");
        let wildcard_action = Permission::new("device", "*");
        let wildcard_all = Permission::new("*", "*");

        assert!(specific.matches(&wildcard_resource));
        assert!(specific.matches(&wildcard_action));
        assert!(specific.matches(&wildcard_all));
        assert!(specific.matches(&specific));

        let other = Permission::new("telemetry", "write");
        assert!(!specific.matches(&other));
    }

    #[test]
    fn test_role_creation() {
        let role = Role::new("test_role", "Test role")
            .add_permission(Permission::new("device", "read"))
            .add_permission(Permission::new("device", "write"));

        assert_eq!(role.name, "test_role");
        assert_eq!(role.permissions.len(), 2);
    }

    #[test]
    fn test_role_has_permission() {
        let role = Role::new("operator", "Operator")
            .add_permission(Permission::new("device", "read"))
            .add_permission(Permission::new("device", "execute"));

        assert!(role.has_permission(&Permission::new("device", "read")));
        assert!(role.has_permission(&Permission::new("device", "execute")));
        assert!(!role.has_permission(&Permission::new("device", "delete")));
    }

    #[test]
    fn test_rbac_manager() {
        let mut rbac = RbacManager::new();

        // Assign roles
        rbac.assign_role("agent_001", "device_operator").unwrap();
        rbac.assign_role("agent_002", "admin").unwrap();

        // Check permissions
        assert!(rbac.has_permission("agent_001", &Permission::new("device", "read")));
        assert!(rbac.has_permission("agent_001", &Permission::new("device", "execute")));
        assert!(!rbac.has_permission("agent_001", &Permission::new("device", "delete")));

        // Admin should have all permissions
        assert!(rbac.has_permission("agent_002", &Permission::new("device", "delete")));
        assert!(rbac.has_permission("agent_002", &Permission::new("system", "configure")));
    }

    #[test]
    fn test_rbac_check_permission() {
        let mut rbac = RbacManager::new();
        rbac.assign_role("agent_001", "monitor").unwrap();

        assert!(rbac.check_permission("agent_001", "device:read").unwrap());
        assert!(rbac
            .check_permission("agent_001", "telemetry:read")
            .unwrap());
        assert!(!rbac.check_permission("agent_001", "device:write").unwrap());
    }

    #[test]
    fn test_rbac_require_permission() {
        let mut rbac = RbacManager::new();
        rbac.assign_role("agent_001", "device_manager").unwrap();

        // Should succeed
        assert!(rbac.require_permission("agent_001", "device:write").is_ok());

        // Should fail
        let result = rbac.require_permission("agent_002", "device:read");
        assert!(result.is_err());
    }

    #[test]
    fn test_revoke_role() {
        let mut rbac = RbacManager::new();
        rbac.assign_role("agent_001", "device_operator").unwrap();

        assert!(rbac.has_permission("agent_001", &Permission::new("device", "read")));

        rbac.revoke_role("agent_001", "device_operator");
        assert!(!rbac.has_permission("agent_001", &Permission::new("device", "read")));
    }

    #[test]
    fn test_multiple_roles() {
        let mut rbac = RbacManager::new();
        rbac.assign_role("agent_001", "monitor").unwrap();
        rbac.assign_role("agent_001", "device_operator").unwrap();

        // Should have permissions from both roles
        assert!(rbac.has_permission("agent_001", &Permission::new("device", "read")));
        assert!(rbac.has_permission("agent_001", &Permission::new("device", "execute")));
    }
}
