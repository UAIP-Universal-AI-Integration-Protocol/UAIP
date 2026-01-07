-- UAIP Hub - Password Security Enhancements
-- Adds support for forcing password changes

ALTER TABLE ai_agents
ADD COLUMN IF NOT EXISTS require_password_change BOOLEAN NOT NULL DEFAULT FALSE;

-- Ensure admin user doesn't need to change password immediately (optional, or set to true for security)
UPDATE ai_agents
SET require_password_change = FALSE
WHERE client_id = 'admin@uaip.io';
