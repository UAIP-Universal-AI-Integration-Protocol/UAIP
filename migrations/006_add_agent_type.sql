-- UAIP Hub - Add agent_type to ai_agents
-- Distinguishes between 'Human' users and 'AI' agents

ALTER TABLE ai_agents 
ADD COLUMN IF NOT EXISTS agent_type VARCHAR(20) DEFAULT 'AI';

-- Fix any invalid or null data before applying constraint
UPDATE ai_agents 
SET agent_type = 'AI' 
WHERE agent_type IS NULL OR agent_type NOT IN ('Human', 'AI');

-- Drop constraint if exists to ensure idempotency
ALTER TABLE ai_agents
DROP CONSTRAINT IF EXISTS check_agent_type;

-- Add the constraint
ALTER TABLE ai_agents
ADD CONSTRAINT check_agent_type CHECK (agent_type IN ('Human', 'AI'));
