# UAIP Database Migrations

This directory contains SQL migration files for the UAIP Hub database schema.

## Migration Files

1. **001_initial_schema.sql** - Core tables for authentication and device management
   - `ai_agents` - OAuth 2.0 clients (AI agents)
   - `devices` - IoT device registry
   - `certificates` - X.509 certificates for devices
   - `sessions` - Active sessions (devices + AI agents)
   - `message_log` - Message history for QoS/audit
   - `audit_log` - Security audit trail

2. **002_rbac_tables.sql** - Role-Based Access Control
   - `roles` - Role definitions
   - `permissions` - Permission definitions
   - `role_permissions` - Role-permission mappings
   - `entity_roles` - Entity role assignments
   - Default roles: admin, device_manager, device_operator, monitor, ai_agent

## Running Migrations

### Using psql (Docker)

```bash
# Ensure PostgreSQL is running
docker-compose up -d postgres

# Run all migrations in order
docker exec -i uaip-postgres psql -U uaip -d uaip < migrations/001_initial_schema.sql
docker exec -i uaip-postgres psql -U uaip -d uaip < migrations/002_rbac_tables.sql
```

### Using sqlx-cli

```bash
# Install sqlx-cli (one time)
cargo install sqlx-cli --no-default-features --features postgres

# Set database URL
export DATABASE_URL="postgresql://uaip:uaip_password_dev@localhost:5432/uaip"

# Run migrations
sqlx migrate run
```

### Verify Migrations

```bash
# Connect to database
docker exec -it uaip-postgres psql -U uaip -d uaip

# List tables
\dt

# Check roles
SELECT * FROM roles;

# Check permissions
SELECT * FROM permissions;

# Check role-permission mappings
SELECT r.name, p.resource, p.action
FROM roles r
JOIN role_permissions rp ON r.id = rp.role_id
JOIN permissions p ON rp.permission_id = p.id
ORDER BY r.name, p.resource, p.action;
```

## Database Functions

### clean_expired_sessions()
Removes expired sessions from the sessions table.

```sql
SELECT clean_expired_sessions();
```

### has_permission(entity_id, entity_type, resource, action)
Checks if an entity has a specific permission.

```sql
SELECT has_permission(
    'uuid-here'::UUID,
    'ai_agent',
    'device',
    'read'
);
```

### get_entity_permissions(entity_id, entity_type)
Returns all permissions for an entity.

```sql
SELECT * FROM get_entity_permissions(
    'uuid-here'::UUID,
    'ai_agent'
);
```

## Schema Diagram

```
ai_agents
  ├─→ entity_roles ─→ roles ─→ role_permissions ─→ permissions
  └─→ sessions

devices
  ├─→ certificates
  ├─→ entity_roles ─→ roles ─→ role_permissions ─→ permissions
  └─→ sessions

message_log  (tracking)
audit_log    (security)
```

## Indexes

All tables have appropriate indexes for:
- Primary keys (UUID)
- Foreign keys
- Frequently queried columns (status, timestamps, etc.)
- Unique constraints (client_id, device_id, serial_number, etc.)

## Triggers

- `trg_session_heartbeat_device` - Updates device.last_seen on session heartbeat

## Notes

- All timestamps use `TIMESTAMPTZ` for timezone awareness
- UUIDs are generated using `uuid_generate_v4()`
- JSONB columns for flexible metadata storage
- Check constraints for enum-like fields (status, qos_level, etc.)
- CASCADE deletes for foreign keys where appropriate
