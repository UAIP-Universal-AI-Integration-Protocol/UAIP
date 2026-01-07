-- Seed default admin user
-- Password is 'password123'
INSERT INTO ai_agents (id, client_id, name, client_secret_hash, scopes, active, created_at)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'admin@uaip.io',
    'Administrator',
    '$2b$12$ZhMItj3WAimtEu2RB9.DIudSQWPitE6y/N0aUAriEyTSCEJncpTcS',
    ARRAY['admin', 'device:read', 'device:write', 'ai:read', 'ai:write'],
    true,
    NOW()
) ON CONFLICT (client_id) DO NOTHING;
