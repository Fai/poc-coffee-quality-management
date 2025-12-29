-- Rollback initial schema

-- Drop triggers first
DROP TRIGGER IF EXISTS create_business_default_roles ON businesses;
DROP TRIGGER IF EXISTS update_notification_preferences_updated_at ON notification_preferences;
DROP TRIGGER IF EXISTS update_line_connections_updated_at ON line_connections;
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
DROP TRIGGER IF EXISTS update_roles_updated_at ON roles;
DROP TRIGGER IF EXISTS update_businesses_updated_at ON businesses;

-- Drop functions
DROP FUNCTION IF EXISTS create_default_roles();
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS audit_log;
DROP TABLE IF EXISTS notification_preferences;
DROP TABLE IF EXISTS refresh_tokens;
DROP TABLE IF EXISTS line_connections;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS role_permissions;
DROP TABLE IF EXISTS permissions;
DROP TABLE IF EXISTS roles;
DROP TABLE IF EXISTS businesses;

-- Drop extension (optional, may be used by other databases)
-- DROP EXTENSION IF EXISTS "uuid-ossp";
