-- Media Management and Streaming Tables
-- Adds support for video, audio, image, and document management with streaming capabilities

-- Media Files Table
CREATE TABLE IF NOT EXISTS media_files (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    filename VARCHAR(500) NOT NULL,
    media_type VARCHAR(20) NOT NULL CHECK (media_type IN ('video', 'audio', 'image', 'document')),
    format VARCHAR(20) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    size_bytes BIGINT NOT NULL,
    duration_secs DOUBLE PRECISION,
    width INTEGER,
    height INTEGER,
    codec_video VARCHAR(50),
    codec_audio VARCHAR(50),
    bitrate_kbps INTEGER,
    framerate_fps REAL,
    storage_path TEXT NOT NULL,
    url TEXT,
    thumbnail_url TEXT,
    tags TEXT[] DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'ready', 'failed', 'archived')),
    source_device_id UUID REFERENCES devices(id) ON DELETE SET NULL,
    access_level VARCHAR(20) NOT NULL DEFAULT 'private' CHECK (access_level IN ('private', 'restricted', 'internal', 'public')),
    uploaded_by_agent UUID REFERENCES ai_agents(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_media_files_media_type ON media_files(media_type);
CREATE INDEX IF NOT EXISTS idx_media_files_status ON media_files(status);
CREATE INDEX IF NOT EXISTS idx_media_files_uploaded_at ON media_files(uploaded_at);
CREATE INDEX IF NOT EXISTS idx_media_files_source_device ON media_files(source_device_id);
CREATE INDEX IF NOT EXISTS idx_media_files_tags ON media_files USING GIN(tags);
CREATE INDEX IF NOT EXISTS idx_media_files_metadata ON media_files USING GIN(metadata);

-- Streaming Configurations Table
CREATE TABLE IF NOT EXISTS stream_configs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    media_id UUID NOT NULL REFERENCES media_files(id) ON DELETE CASCADE,
    protocol VARCHAR(20) NOT NULL CHECK (protocol IN ('HLS', 'DASH', 'WEBRTC', 'RTMP', 'HTTP')),
    quality VARCHAR(20) NOT NULL DEFAULT 'auto' CHECK (quality IN ('auto', 'low', 'medium', 'high', 'fullhd', 'ultrahd')),
    adaptive BOOLEAN NOT NULL DEFAULT TRUE,
    segment_duration_secs REAL NOT NULL DEFAULT 6.0,
    stream_url TEXT,
    is_live BOOLEAN NOT NULL DEFAULT FALSE,
    buffer_secs REAL NOT NULL DEFAULT 30.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE INDEX IF NOT EXISTS idx_stream_configs_media_id ON stream_configs(media_id);
CREATE INDEX IF NOT EXISTS idx_stream_configs_protocol ON stream_configs(protocol);
CREATE INDEX IF NOT EXISTS idx_stream_configs_active ON stream_configs(active);

-- Media Processing Jobs Table
CREATE TABLE IF NOT EXISTS media_processing_jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    media_id UUID NOT NULL REFERENCES media_files(id) ON DELETE CASCADE,
    operation_type VARCHAR(50) NOT NULL,
    operation_config JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    progress REAL NOT NULL DEFAULT 0.0 CHECK (progress >= 0.0 AND progress <= 100.0),
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    output_media_id UUID REFERENCES media_files(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_media_jobs_media_id ON media_processing_jobs(media_id);
CREATE INDEX IF NOT EXISTS idx_media_jobs_status ON media_processing_jobs(status);
CREATE INDEX IF NOT EXISTS idx_media_jobs_created_at ON media_processing_jobs(created_at);

-- AI Agent Sessions Extended (for media and device interactions)
CREATE TABLE IF NOT EXISTS ai_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL REFERENCES ai_agents(id) ON DELETE CASCADE,
    state VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (state IN ('active', 'paused', 'terminating', 'terminated', 'error')),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    context_data JSONB DEFAULT '{}',
    commands_sent BIGINT DEFAULT 0,
    responses_received BIGINT DEFAULT 0,
    errors_count BIGINT DEFAULT 0,
    bytes_sent BIGINT DEFAULT 0,
    bytes_received BIGINT DEFAULT 0,
    avg_response_time_ms REAL DEFAULT 0.0
);

CREATE INDEX IF NOT EXISTS idx_ai_sessions_agent_id ON ai_sessions(agent_id);
CREATE INDEX IF NOT EXISTS idx_ai_sessions_state ON ai_sessions(state);
CREATE INDEX IF NOT EXISTS idx_ai_sessions_last_activity ON ai_sessions(last_activity_at);

-- AI Session Devices (Many-to-Many relationship)
CREATE TABLE IF NOT EXISTS ai_session_devices (
    session_id UUID NOT NULL REFERENCES ai_sessions(id) ON DELETE CASCADE,
    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (session_id, device_id)
);

CREATE INDEX IF NOT EXISTS idx_ai_session_devices_session ON ai_session_devices(session_id);
CREATE INDEX IF NOT EXISTS idx_ai_session_devices_device ON ai_session_devices(device_id);

-- Network Endpoints Configuration
CREATE TABLE IF NOT EXISTS network_endpoints (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) UNIQUE NOT NULL,
    endpoint_type VARCHAR(50) NOT NULL,
    protocol VARCHAR(20) NOT NULL CHECK (protocol IN ('http', 'https', 'tcp', 'udp', 'websocket', 'mqtt', 'modbus', 'opcua')),
    host VARCHAR(255) NOT NULL,
    port INTEGER NOT NULL CHECK (port > 0 AND port <= 65535),
    connection_string TEXT,
    auth_config JSONB,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    timeout_ms INTEGER NOT NULL DEFAULT 10000,
    max_retries INTEGER NOT NULL DEFAULT 3,
    retry_delay_ms INTEGER NOT NULL DEFAULT 1000,
    parameters JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_health_check TIMESTAMPTZ,
    health_status VARCHAR(20) DEFAULT 'unknown' CHECK (health_status IN ('healthy', 'degraded', 'unhealthy', 'unknown'))
);

CREATE INDEX IF NOT EXISTS idx_network_endpoints_name ON network_endpoints(name);
CREATE INDEX IF NOT EXISTS idx_network_endpoints_type ON network_endpoints(endpoint_type);
CREATE INDEX IF NOT EXISTS idx_network_endpoints_enabled ON network_endpoints(enabled);
CREATE INDEX IF NOT EXISTS idx_network_endpoints_health ON network_endpoints(health_status);

-- Orchestration Rules (extending automation capabilities)
CREATE TABLE IF NOT EXISTS orchestration_rules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    rule_definition JSONB NOT NULL,
    condition_mode VARCHAR(10) NOT NULL DEFAULT 'all' CHECK (condition_mode IN ('all', 'any')),
    priority INTEGER NOT NULL DEFAULT 0,
    cooldown_seconds INTEGER,
    last_executed TIMESTAMPTZ,
    execution_count BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by_agent UUID REFERENCES ai_agents(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_orchestration_rules_enabled ON orchestration_rules(enabled);
CREATE INDEX IF NOT EXISTS idx_orchestration_rules_priority ON orchestration_rules(priority DESC);
CREATE INDEX IF NOT EXISTS idx_orchestration_rules_created_by ON orchestration_rules(created_by_agent);

-- Workflows (for complex multi-step automations)
CREATE TABLE IF NOT EXISTS workflows (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    workflow_definition JSONB NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    state VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (state IN ('pending', 'running', 'completed', 'failed', 'paused', 'cancelled')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_by_agent UUID REFERENCES ai_agents(id) ON DELETE SET NULL,
    current_step_id VARCHAR(255),
    execution_context JSONB DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_workflows_state ON workflows(state);
CREATE INDEX IF NOT EXISTS idx_workflows_enabled ON workflows(enabled);
CREATE INDEX IF NOT EXISTS idx_workflows_created_by ON workflows(created_by_agent);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers for updated_at
DROP TRIGGER IF EXISTS trg_stream_configs_updated_at ON stream_configs;
CREATE TRIGGER trg_stream_configs_updated_at
    BEFORE UPDATE ON stream_configs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS trg_network_endpoints_updated_at ON network_endpoints;
CREATE TRIGGER trg_network_endpoints_updated_at
    BEFORE UPDATE ON network_endpoints
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS trg_orchestration_rules_updated_at ON orchestration_rules;
CREATE TRIGGER trg_orchestration_rules_updated_at
    BEFORE UPDATE ON orchestration_rules
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to clean up old media files
CREATE OR REPLACE FUNCTION archive_old_media(days INTEGER DEFAULT 90)
RETURNS INTEGER AS $$
DECLARE
    archived_count INTEGER;
BEGIN
    UPDATE media_files
    SET status = 'archived'
    WHERE uploaded_at < NOW() - (days || ' days')::INTERVAL
    AND status != 'archived'
    AND access_level = 'private';

    GET DIAGNOSTICS archived_count = ROW_COUNT;
    RETURN archived_count;
END;
$$ LANGUAGE plpgsql;

-- Function to cleanup expired AI sessions
CREATE OR REPLACE FUNCTION cleanup_expired_ai_sessions(timeout_hours INTEGER DEFAULT 24)
RETURNS INTEGER AS $$
DECLARE
    terminated_count INTEGER;
BEGIN
    UPDATE ai_sessions
    SET state = 'terminated'
    WHERE last_activity_at < NOW() - (timeout_hours || ' hours')::INTERVAL
    AND state = 'active';

    GET DIAGNOSTICS terminated_count = ROW_COUNT;
    RETURN terminated_count;
END;
$$ LANGUAGE plpgsql;

-- Comments
COMMENT ON TABLE media_files IS 'Stores metadata for all media files (video, audio, images, documents)';
COMMENT ON TABLE stream_configs IS 'Streaming configurations for media files';
COMMENT ON TABLE media_processing_jobs IS 'Background jobs for media transcoding, thumbnail generation, etc.';
COMMENT ON TABLE ai_sessions IS 'Extended AI agent sessions with interaction tracking';
COMMENT ON TABLE ai_session_devices IS 'Devices connected to AI agent sessions';
COMMENT ON TABLE network_endpoints IS 'Network endpoint configurations for adapters and services';
COMMENT ON TABLE orchestration_rules IS 'Automation rules for device orchestration';
COMMENT ON TABLE workflows IS 'Complex multi-step workflow definitions and executions';
