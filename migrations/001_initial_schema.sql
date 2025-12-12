-- UAIP Hub - Initial Database Schema
-- Creates core tables for authentication and device management

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- AI Agents Table (OAuth 2.0 Clients)
CREATE TABLE ai_agents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    client_id VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    client_secret_hash VARCHAR(255) NOT NULL,
    scopes TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_authenticated TIMESTAMPTZ,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_ai_agents_client_id ON ai_agents(client_id);
CREATE INDEX idx_ai_agents_active ON ai_agents(active);

-- Devices Table
CREATE TABLE devices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    device_id VARCHAR(255) UNIQUE NOT NULL,
    mac_address VARCHAR(17) UNIQUE NOT NULL,
    manufacturer VARCHAR(255) NOT NULL,
    model VARCHAR(255) NOT NULL,
    firmware_version VARCHAR(50),
    status VARCHAR(20) NOT NULL CHECK (status IN ('online', 'offline', 'error', 'maintenance', 'deactivated')),
    last_seen TIMESTAMPTZ,
    registered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    certificate_expiry TIMESTAMPTZ,
    configuration JSONB DEFAULT '{}',
    capabilities JSONB NOT NULL DEFAULT '[]',
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_devices_device_id ON devices(device_id);
CREATE INDEX idx_devices_status ON devices(status);
CREATE INDEX idx_devices_last_seen ON devices(last_seen);
CREATE INDEX idx_devices_mac_address ON devices(mac_address);

-- Certificates Table (X.509 for devices)
CREATE TABLE certificates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    device_id UUID REFERENCES devices(id) ON DELETE CASCADE,
    serial_number VARCHAR(255) UNIQUE NOT NULL,
    common_name VARCHAR(255) NOT NULL,
    issuer_cn VARCHAR(255) NOT NULL,
    public_key TEXT NOT NULL,
    fingerprint VARCHAR(255) NOT NULL,
    not_before TIMESTAMPTZ NOT NULL,
    not_after TIMESTAMPTZ NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ,
    revocation_reason VARCHAR(100)
);

CREATE INDEX idx_certificates_device_id ON certificates(device_id);
CREATE INDEX idx_certificates_serial_number ON certificates(serial_number);
CREATE INDEX idx_certificates_fingerprint ON certificates(fingerprint);
CREATE INDEX idx_certificates_not_after ON certificates(not_after);
CREATE INDEX idx_certificates_revoked ON certificates(revoked_at) WHERE revoked_at IS NOT NULL;

-- Sessions Table (for both devices and AI agents)
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id VARCHAR(255) UNIQUE NOT NULL,
    entity_id UUID NOT NULL,
    entity_type VARCHAR(20) NOT NULL CHECK (entity_type IN ('device', 'ai_agent')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_heartbeat TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_sessions_session_id ON sessions(session_id);
CREATE INDEX idx_sessions_entity ON sessions(entity_id, entity_type);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX idx_sessions_last_heartbeat ON sessions(last_heartbeat);

-- Message Log Table (for QoS and audit)
CREATE TABLE message_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    message_id VARCHAR(255) UNIQUE NOT NULL,
    correlation_id VARCHAR(255),
    sender_id VARCHAR(255) NOT NULL,
    recipient_id VARCHAR(255) NOT NULL,
    action VARCHAR(50) NOT NULL,
    qos_level SMALLINT NOT NULL CHECK (qos_level IN (0, 1, 2)),
    priority VARCHAR(20) NOT NULL CHECK (priority IN ('low', 'normal', 'high', 'critical')),
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'delivered', 'failed', 'expired')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    delivered_at TIMESTAMPTZ,
    payload JSONB,
    error_message TEXT
);

CREATE INDEX idx_message_log_message_id ON message_log(message_id);
CREATE INDEX idx_message_log_correlation_id ON message_log(correlation_id);
CREATE INDEX idx_message_log_sender ON message_log(sender_id);
CREATE INDEX idx_message_log_recipient ON message_log(recipient_id);
CREATE INDEX idx_message_log_status ON message_log(status);
CREATE INDEX idx_message_log_created_at ON message_log(created_at);

-- Audit Log Table
CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    entity_id VARCHAR(255) NOT NULL,
    entity_type VARCHAR(20) NOT NULL,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50),
    resource_id VARCHAR(255),
    success BOOLEAN NOT NULL,
    error_message TEXT,
    ip_address INET,
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp);
CREATE INDEX idx_audit_log_entity ON audit_log(entity_id, entity_type);
CREATE INDEX idx_audit_log_action ON audit_log(action);
CREATE INDEX idx_audit_log_resource ON audit_log(resource_type, resource_id);

-- Function to clean expired sessions
CREATE OR REPLACE FUNCTION clean_expired_sessions()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM sessions
    WHERE expires_at < NOW();

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Function to update device last_seen on heartbeat
CREATE OR REPLACE FUNCTION update_device_last_seen()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE devices
    SET last_seen = NEW.last_heartbeat
    WHERE id = NEW.entity_id AND NEW.entity_type = 'device';

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update device last_seen on session heartbeat
CREATE TRIGGER trg_session_heartbeat_device
    AFTER UPDATE OF last_heartbeat ON sessions
    FOR EACH ROW
    WHEN (NEW.entity_type = 'device')
    EXECUTE FUNCTION update_device_last_seen();

-- Comments for documentation
COMMENT ON TABLE ai_agents IS 'OAuth 2.0 clients (AI agents) that can authenticate and access the API';
COMMENT ON TABLE devices IS 'Registered IoT devices with their capabilities and status';
COMMENT ON TABLE certificates IS 'X.509 certificates for device authentication';
COMMENT ON TABLE sessions IS 'Active sessions for both devices and AI agents';
COMMENT ON TABLE message_log IS 'Message history for QoS tracking and audit';
COMMENT ON TABLE audit_log IS 'Security audit log for all critical operations';
