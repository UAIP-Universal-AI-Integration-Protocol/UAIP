-- UAIP Hub - RBAC Tables
-- Role-Based Access Control for fine-grained permissions

-- Roles Table
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Permissions Table
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    resource VARCHAR(100) NOT NULL,
    action VARCHAR(100) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(resource, action)
);

-- Role Permissions (many-to-many)
CREATE TABLE role_permissions (
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID REFERENCES permissions(id) ON DELETE CASCADE,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (role_id, permission_id)
);

-- Entity Role Assignments (for both devices and AI agents)
CREATE TABLE entity_roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    entity_id UUID NOT NULL,
    entity_type VARCHAR(20) NOT NULL CHECK (entity_type IN ('device', 'ai_agent')),
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_by VARCHAR(255),
    expires_at TIMESTAMPTZ,
    UNIQUE(entity_id, entity_type, role_id)
);

CREATE INDEX idx_entity_roles_entity ON entity_roles(entity_id, entity_type);
CREATE INDEX idx_entity_roles_role ON entity_roles(role_id);
CREATE INDEX idx_entity_roles_expires_at ON entity_roles(expires_at);

-- Insert default roles
INSERT INTO roles (name, description) VALUES
    ('admin', 'Full system access - all permissions'),
    ('device_manager', 'Can manage all devices - create, read, update, delete'),
    ('device_operator', 'Can operate devices - read and execute commands'),
    ('monitor', 'Read-only access to devices and telemetry'),
    ('ai_agent', 'Standard AI agent permissions - read devices, execute commands, subscribe to events');

-- Insert default permissions
INSERT INTO permissions (resource, action, description) VALUES
    ('*', '*', 'Wildcard - all permissions'),
    ('device', 'read', 'Read device information'),
    ('device', 'write', 'Modify device configuration'),
    ('device', 'execute', 'Execute commands on devices'),
    ('device', 'delete', 'Delete/deactivate devices'),
    ('telemetry', 'read', 'Read telemetry data'),
    ('telemetry', 'write', 'Write telemetry data'),
    ('command', 'execute', 'Execute commands'),
    ('event', 'subscribe', 'Subscribe to events'),
    ('event', 'publish', 'Publish events'),
    ('system', 'configure', 'Configure system settings'),
    ('user', 'manage', 'Manage users and agents');

-- Assign permissions to roles

-- Admin: all permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'admin' AND p.resource = '*' AND p.action = '*';

-- Device Manager: full device management
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'device_manager'
AND (
    (p.resource = 'device' AND p.action IN ('read', 'write', 'execute', 'delete'))
    OR (p.resource = 'telemetry' AND p.action = 'read')
    OR (p.resource = 'command' AND p.action = 'execute')
);

-- Device Operator: operate devices
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'device_operator'
AND (
    (p.resource = 'device' AND p.action IN ('read', 'execute'))
    OR (p.resource = 'telemetry' AND p.action = 'read')
    OR (p.resource = 'command' AND p.action = 'execute')
);

-- Monitor: read-only
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'monitor'
AND (
    (p.resource = 'device' AND p.action = 'read')
    OR (p.resource = 'telemetry' AND p.action = 'read')
);

-- AI Agent: typical AI agent permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'ai_agent'
AND (
    (p.resource = 'device' AND p.action IN ('read', 'execute'))
    OR (p.resource = 'telemetry' AND p.action = 'read')
    OR (p.resource = 'command' AND p.action = 'execute')
    OR (p.resource = 'event' AND p.action = 'subscribe')
);

-- Function to check if entity has permission
CREATE OR REPLACE FUNCTION has_permission(
    p_entity_id UUID,
    p_entity_type VARCHAR(20),
    p_resource VARCHAR(100),
    p_action VARCHAR(100)
)
RETURNS BOOLEAN AS $$
DECLARE
    has_perm BOOLEAN;
BEGIN
    SELECT EXISTS (
        SELECT 1
        FROM entity_roles er
        JOIN role_permissions rp ON er.role_id = rp.role_id
        JOIN permissions perm ON rp.permission_id = perm.id
        WHERE er.entity_id = p_entity_id
          AND er.entity_type = p_entity_type
          AND (er.expires_at IS NULL OR er.expires_at > NOW())
          AND (
              -- Exact match
              (perm.resource = p_resource AND perm.action = p_action)
              -- Wildcard resource
              OR (perm.resource = '*' AND perm.action = p_action)
              -- Wildcard action
              OR (perm.resource = p_resource AND perm.action = '*')
              -- Wildcard all
              OR (perm.resource = '*' AND perm.action = '*')
          )
    ) INTO has_perm;

    RETURN has_perm;
END;
$$ LANGUAGE plpgsql;

-- Function to get all permissions for an entity
CREATE OR REPLACE FUNCTION get_entity_permissions(
    p_entity_id UUID,
    p_entity_type VARCHAR(20)
)
RETURNS TABLE (
    resource VARCHAR(100),
    action VARCHAR(100),
    description TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT DISTINCT p.resource, p.action, p.description
    FROM entity_roles er
    JOIN role_permissions rp ON er.role_id = rp.role_id
    JOIN permissions p ON rp.permission_id = p.id
    WHERE er.entity_id = p_entity_id
      AND er.entity_type = p_entity_type
      AND (er.expires_at IS NULL OR er.expires_at > NOW());
END;
$$ LANGUAGE plpgsql;

-- Comments
COMMENT ON TABLE roles IS 'Role definitions for RBAC';
COMMENT ON TABLE permissions IS 'Permission definitions (resource:action pairs)';
COMMENT ON TABLE role_permissions IS 'Mapping between roles and permissions';
COMMENT ON TABLE entity_roles IS 'Role assignments for entities (devices and AI agents)';
COMMENT ON FUNCTION has_permission IS 'Check if an entity has a specific permission';
COMMENT ON FUNCTION get_entity_permissions IS 'Get all permissions for an entity';
