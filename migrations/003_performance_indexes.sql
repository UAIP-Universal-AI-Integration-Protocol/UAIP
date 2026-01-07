-- UAIP Hub - Performance Indexes
-- Optimized indexes for common query patterns

-- ============================================================================
-- DEVICES TABLE INDEXES
-- ============================================================================

-- Composite index for device type and status queries
-- Supports: SELECT * FROM devices WHERE device_type = ? AND status = ?
-- Common use case: Filtering devices by type and status
CREATE INDEX IF NOT EXISTS idx_devices_type_status
ON devices(manufacturer, status);

-- GIN index for JSONB capabilities field
-- Supports: SELECT * FROM devices WHERE capabilities @> '["temperature"]'
-- Common use case: Finding devices with specific capabilities
CREATE INDEX IF NOT EXISTS idx_devices_capabilities_gin
ON devices USING GIN (capabilities);

-- GIN index for JSONB metadata field
-- Supports: SELECT * FROM devices WHERE metadata @> '{"key": "value"}'
-- Common use case: Querying devices by metadata attributes
CREATE INDEX IF NOT EXISTS idx_devices_metadata_gin
ON devices USING GIN (metadata);

-- Partial index for active (non-deactivated) devices
-- Supports: SELECT * FROM devices WHERE status != 'deactivated'
-- Benefit: Smaller index, faster queries for active devices
CREATE INDEX IF NOT EXISTS idx_devices_active
ON devices(id, status, last_seen)
WHERE status != 'deactivated';

-- Composite index for manufacturer and model lookups
-- Supports: SELECT * FROM devices WHERE manufacturer = ? AND model = ?
CREATE INDEX IF NOT EXISTS idx_devices_manufacturer_model
ON devices(manufacturer, model);

-- Index for firmware version queries (useful for update targeting)
-- Supports: SELECT * FROM devices WHERE firmware_version = ?
CREATE INDEX IF NOT EXISTS idx_devices_firmware_version
ON devices(firmware_version)
WHERE firmware_version IS NOT NULL;

-- ============================================================================
-- MESSAGE_LOG TABLE INDEXES
-- ============================================================================

-- Composite index for message priority and creation time
-- Supports: SELECT * FROM message_log ORDER BY priority DESC, created_at
-- Common use case: Processing messages in priority order
CREATE INDEX IF NOT EXISTS idx_message_log_priority_created
ON message_log(priority DESC, created_at);

-- Partial index for undelivered messages
-- Supports: SELECT * FROM message_log WHERE status = 'pending'
-- Benefit: Fast retrieval of pending messages for delivery
CREATE INDEX IF NOT EXISTS idx_message_log_undelivered
ON message_log(id, priority, created_at)
WHERE status = 'pending';

-- Composite index for sender queries with time
-- Supports: SELECT * FROM message_log WHERE sender_id = ? ORDER BY created_at DESC
CREATE INDEX IF NOT EXISTS idx_message_log_sender_created
ON message_log(sender_id, created_at DESC);

-- Composite index for recipient queries with time
-- Supports: SELECT * FROM message_log WHERE recipient_id = ? ORDER BY created_at DESC
CREATE INDEX IF NOT EXISTS idx_message_log_recipient_created
ON message_log(recipient_id, created_at DESC);

-- Index for QoS level queries
-- Supports: SELECT * FROM message_log WHERE qos_level = ?
CREATE INDEX IF NOT EXISTS idx_message_log_qos
ON message_log(qos_level, status);

-- Partial index for failed messages
-- Supports: SELECT * FROM message_log WHERE status = 'failed'
CREATE INDEX IF NOT EXISTS idx_message_log_failed
ON message_log(id, created_at)
WHERE status = 'failed';

-- ============================================================================
-- AI_AGENTS TABLE INDEXES
-- ============================================================================

-- Index for active agents lookup
-- Supports: SELECT * FROM ai_agents WHERE active = true
CREATE INDEX IF NOT EXISTS idx_ai_agents_active_lookup
ON ai_agents(active, last_authenticated);

-- GIN index for scopes array
-- Supports: SELECT * FROM ai_agents WHERE scopes @> ARRAY['device:read']
CREATE INDEX IF NOT EXISTS idx_ai_agents_scopes_gin
ON ai_agents USING GIN (scopes);

-- GIN index for metadata JSONB
-- Supports: SELECT * FROM ai_agents WHERE metadata @> '{"key": "value"}'
CREATE INDEX IF NOT EXISTS idx_ai_agents_metadata_gin
ON ai_agents USING GIN (metadata);

-- ============================================================================
-- SESSIONS TABLE INDEXES
-- ============================================================================

-- Composite index for entity sessions
-- Supports: SELECT * FROM sessions WHERE entity_id = ? AND entity_type = ?
CREATE INDEX IF NOT EXISTS idx_sessions_entity_active
ON sessions(entity_id, entity_type, expires_at);

-- Index for session cleanup by expiration
-- Supports: DELETE FROM sessions WHERE expires_at < NOW()
CREATE INDEX IF NOT EXISTS idx_sessions_expiry_cleanup
ON sessions(expires_at);

-- Index for heartbeat monitoring
-- Supports: SELECT * FROM sessions WHERE last_heartbeat < NOW() - INTERVAL '5 minutes'
CREATE INDEX IF NOT EXISTS idx_sessions_heartbeat_check
ON sessions(last_heartbeat, entity_type);

-- ============================================================================
-- CERTIFICATES TABLE INDEXES
-- ============================================================================

-- Index for certificate expiry monitoring
-- Supports: SELECT * FROM certificates WHERE not_after < NOW() + INTERVAL '30 days'
CREATE INDEX IF NOT EXISTS idx_certificates_expiry_warning
ON certificates(not_after, device_id)
WHERE revoked_at IS NULL;

-- Composite index for device certificate lookup
-- Supports: SELECT * FROM certificates WHERE device_id = ? AND revoked_at IS NULL
CREATE INDEX IF NOT EXISTS idx_certificates_device_valid
ON certificates(device_id, not_after)
WHERE revoked_at IS NULL;

-- ============================================================================
-- AUDIT_LOG TABLE INDEXES
-- ============================================================================

-- Composite index for entity audit queries
-- Supports: SELECT * FROM audit_log WHERE entity_id = ? ORDER BY timestamp DESC
CREATE INDEX IF NOT EXISTS idx_audit_log_entity_time
ON audit_log(entity_id, entity_type, timestamp DESC);

-- Index for action-based queries
-- Supports: SELECT * FROM audit_log WHERE action = ? AND success = ?
CREATE INDEX IF NOT EXISTS idx_audit_log_action_success
ON audit_log(action, success, timestamp DESC);

-- Partial index for failed operations
-- Supports: SELECT * FROM audit_log WHERE success = false
CREATE INDEX IF NOT EXISTS idx_audit_log_failures
ON audit_log(timestamp DESC, action, entity_id)
WHERE success = false;

-- Time-based index for retention cleanup
-- Supports: DELETE FROM audit_log WHERE timestamp < NOW() - INTERVAL '90 days'
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp_cleanup
ON audit_log(timestamp);

-- ============================================================================
-- RBAC INDEXES (from 002_rbac_tables.sql)
-- ============================================================================

-- Composite index for permission checks
-- Supports: Fast permission lookups for entities
CREATE INDEX IF NOT EXISTS idx_entity_roles_permission_check
ON entity_roles(entity_id, entity_type, role_id);

-- Index for role-based queries
-- Supports: SELECT * FROM entity_roles WHERE role_id = ?
CREATE INDEX IF NOT EXISTS idx_entity_roles_role_lookup
ON entity_roles(role_id, entity_type);

-- ============================================================================
-- STATISTICS AND COMMENTS
-- ============================================================================

-- Update table statistics for query planner
ANALYZE devices;
ANALYZE message_log;
ANALYZE ai_agents;
ANALYZE sessions;
ANALYZE certificates;
ANALYZE audit_log;
ANALYZE entity_roles;

-- Comments for documentation
COMMENT ON INDEX idx_devices_type_status IS 'Composite index for filtering devices by type and status';
COMMENT ON INDEX idx_devices_capabilities_gin IS 'GIN index for JSONB capabilities queries';
COMMENT ON INDEX idx_devices_metadata_gin IS 'GIN index for JSONB metadata queries';
COMMENT ON INDEX idx_devices_active IS 'Partial index for active (non-deactivated) devices';
COMMENT ON INDEX idx_message_log_priority_created IS 'Composite index for priority-based message processing';
COMMENT ON INDEX idx_message_log_undelivered IS 'Partial index for pending messages awaiting delivery';
COMMENT ON INDEX idx_sessions_entity_active IS 'Composite index for active entity sessions';
COMMENT ON INDEX idx_certificates_expiry_warning IS 'Index for monitoring certificate expiration';
COMMENT ON INDEX idx_audit_log_failures IS 'Partial index for failed operations audit trail';

-- Performance monitoring query examples:
--
-- 1. Check index usage:
--   SELECT schemaname, tablename, indexname, idx_scan, idx_tup_read, idx_tup_fetch
--   FROM pg_stat_user_indexes
--   WHERE schemaname = 'public'
--   ORDER BY idx_scan DESC;
--
-- 2. Find unused indexes:
--   SELECT schemaname, tablename, indexname, idx_scan
--   FROM pg_stat_user_indexes
--   WHERE schemaname = 'public' AND idx_scan = 0
--   ORDER BY pg_relation_size(indexrelid) DESC;
--
-- 3. Check table bloat:
--   SELECT schemaname, tablename,
--          pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
--   FROM pg_tables
--   WHERE schemaname = 'public'
--   ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
