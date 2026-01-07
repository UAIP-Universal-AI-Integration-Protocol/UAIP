-- Add missing columns to ai_agents table to match backend code expectations

ALTER TABLE ai_agents 
ADD COLUMN IF NOT EXISTS agent_type VARCHAR(50) DEFAULT 'Custom',
ADD COLUMN IF NOT EXISTS version VARCHAR(50) DEFAULT '1.0.0',
ADD COLUMN IF NOT EXISTS provider VARCHAR(100) DEFAULT 'Custom',
ADD COLUMN IF NOT EXISTS config JSONB DEFAULT '{}';

-- Create index for agent_type
CREATE INDEX IF NOT EXISTS idx_ai_agents_agent_type ON ai_agents(agent_type);
