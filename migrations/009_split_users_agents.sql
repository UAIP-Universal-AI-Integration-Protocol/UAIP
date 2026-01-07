-- Split Users and AI Agents
-- Moves 'Human' agents to a new 'users' table

-- 1. Create users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'viewer',
    active BOOLEAN NOT NULL DEFAULT TRUE,
    require_password_change BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login TIMESTAMPTZ
);

-- 2. Migrate data
-- Map scopes to roles:
-- 'admin' scope -> 'admin' role
-- 'device:write' scope -> 'operator' role
-- else -> 'viewer' role
INSERT INTO users (id, email, name, password_hash, active, require_password_change, created_at, last_login, role)
SELECT 
    id, 
    client_id, -- email is stored in client_id for humans
    name, 
    client_secret_hash, -- password_hash is stored here
    active,
    require_password_change,
    created_at, 
    last_authenticated,
    CASE 
        WHEN 'admin' = ANY(scopes) THEN 'admin'
        WHEN 'device:write' = ANY(scopes) THEN 'operator'
        ELSE 'viewer'
    END
FROM ai_agents 
WHERE agent_type = 'Human';

-- 3. Update entity_roles reference
-- First, update the check constraint to allow 'user'
ALTER TABLE entity_roles DROP CONSTRAINT IF EXISTS entity_roles_entity_type_check;
ALTER TABLE entity_roles ADD CONSTRAINT entity_roles_entity_type_check 
    CHECK (entity_type IN ('device', 'ai_agent', 'user'));

-- Change entity_type from 'ai_agent' to 'user' for migrated records
UPDATE entity_roles 
SET entity_type = 'user' 
WHERE entity_id IN (SELECT id FROM users);

-- 4. Cleanup ai_agents
DELETE FROM ai_agents WHERE agent_type = 'Human';

-- 5. Drop columns not needed for AI agents anymore?
-- We keep them for now to avoid breaking other legacy code immediately, 
-- but 'require_password_change' is definitely a user-feature.
ALTER TABLE ai_agents DROP COLUMN IF EXISTS require_password_change;
