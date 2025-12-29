-- Initial schema for Coffee Quality Management Platform
-- Creates core tables: businesses, users, roles, permissions

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================================
-- BUSINESSES
-- ============================================================================
CREATE TABLE businesses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    business_type VARCHAR(50) NOT NULL CHECK (business_type IN ('farmer', 'processor', 'roaster', 'multi')),
    business_code VARCHAR(10) NOT NULL UNIQUE, -- Used in traceability codes (e.g., "DOI", "CMI")
    phone VARCHAR(20),
    email VARCHAR(255),
    address TEXT,
    province VARCHAR(100),
    district VARCHAR(100),
    subdistrict VARCHAR(100),
    postal_code VARCHAR(10),
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    preferred_language VARCHAR(5) NOT NULL DEFAULT 'th' CHECK (preferred_language IN ('th', 'en')),
    timezone VARCHAR(50) NOT NULL DEFAULT 'Asia/Bangkok',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_businesses_business_code ON businesses(business_code);
CREATE INDEX idx_businesses_business_type ON businesses(business_type);

-- ============================================================================
-- ROLES
-- ============================================================================
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    name_th VARCHAR(100), -- Thai name for display
    description TEXT,
    description_th TEXT,
    is_system_role BOOLEAN NOT NULL DEFAULT FALSE, -- True for owner, manager, worker
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(business_id, name)
);

CREATE INDEX idx_roles_business_id ON roles(business_id);

-- ============================================================================
-- PERMISSIONS
-- ============================================================================
-- Available resources and actions
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    resource VARCHAR(50) NOT NULL,
    action VARCHAR(50) NOT NULL,
    description TEXT,
    description_th TEXT,
    UNIQUE(resource, action)
);

-- Seed default permissions
INSERT INTO permissions (resource, action, description, description_th) VALUES
    -- Plot permissions
    ('plot', 'view', 'View plots', 'ดูแปลง'),
    ('plot', 'create', 'Create plots', 'สร้างแปลง'),
    ('plot', 'edit', 'Edit plots', 'แก้ไขแปลง'),
    ('plot', 'delete', 'Delete plots', 'ลบแปลง'),
    -- Harvest permissions
    ('harvest', 'view', 'View harvests', 'ดูการเก็บเกี่ยว'),
    ('harvest', 'create', 'Record harvests', 'บันทึกการเก็บเกี่ยว'),
    ('harvest', 'edit', 'Edit harvests', 'แก้ไขการเก็บเกี่ยว'),
    ('harvest', 'delete', 'Delete harvests', 'ลบการเก็บเกี่ยว'),
    -- Processing permissions
    ('processing', 'view', 'View processing records', 'ดูการแปรรูป'),
    ('processing', 'create', 'Create processing records', 'สร้างการแปรรูป'),
    ('processing', 'edit', 'Edit processing records', 'แก้ไขการแปรรูป'),
    ('processing', 'delete', 'Delete processing records', 'ลบการแปรรูป'),
    -- Grading permissions
    ('grading', 'view', 'View gradings', 'ดูการเกรด'),
    ('grading', 'create', 'Create gradings', 'สร้างการเกรด'),
    ('grading', 'edit', 'Edit gradings', 'แก้ไขการเกรด'),
    ('grading', 'delete', 'Delete gradings', 'ลบการเกรด'),
    -- Cupping permissions
    ('cupping', 'view', 'View cupping sessions', 'ดูการคัปปิ้ง'),
    ('cupping', 'create', 'Create cupping sessions', 'สร้างการคัปปิ้ง'),
    ('cupping', 'edit', 'Edit cupping sessions', 'แก้ไขการคัปปิ้ง'),
    ('cupping', 'delete', 'Delete cupping sessions', 'ลบการคัปปิ้ง'),
    -- Inventory permissions
    ('inventory', 'view', 'View inventory', 'ดูสินค้าคงคลัง'),
    ('inventory', 'create', 'Create inventory transactions', 'สร้างรายการสินค้า'),
    ('inventory', 'edit', 'Edit inventory', 'แก้ไขสินค้าคงคลัง'),
    ('inventory', 'delete', 'Delete inventory records', 'ลบรายการสินค้า'),
    -- Roast profile permissions
    ('roast_profile', 'view', 'View roast profiles', 'ดูโปรไฟล์การคั่ว'),
    ('roast_profile', 'create', 'Create roast profiles', 'สร้างโปรไฟล์การคั่ว'),
    ('roast_profile', 'edit', 'Edit roast profiles', 'แก้ไขโปรไฟล์การคั่ว'),
    ('roast_profile', 'delete', 'Delete roast profiles', 'ลบโปรไฟล์การคั่ว'),
    -- Report permissions
    ('report', 'view', 'View reports', 'ดูรายงาน'),
    ('report', 'export', 'Export reports', 'ส่งออกรายงาน'),
    -- Certification permissions
    ('certification', 'view', 'View certifications', 'ดูใบรับรอง'),
    ('certification', 'create', 'Create certifications', 'สร้างใบรับรอง'),
    ('certification', 'edit', 'Edit certifications', 'แก้ไขใบรับรอง'),
    ('certification', 'delete', 'Delete certifications', 'ลบใบรับรอง'),
    -- User management permissions
    ('user', 'view', 'View users', 'ดูผู้ใช้'),
    ('user', 'create', 'Create users', 'สร้างผู้ใช้'),
    ('user', 'edit', 'Edit users', 'แก้ไขผู้ใช้'),
    ('user', 'delete', 'Delete users', 'ลบผู้ใช้'),
    -- Role management permissions
    ('role', 'view', 'View roles', 'ดูบทบาท'),
    ('role', 'create', 'Create roles', 'สร้างบทบาท'),
    ('role', 'edit', 'Edit roles', 'แก้ไขบทบาท'),
    ('role', 'delete', 'Delete roles', 'ลบบทบาท'),
    -- Business settings permissions
    ('business', 'view', 'View business settings', 'ดูการตั้งค่าธุรกิจ'),
    ('business', 'edit', 'Edit business settings', 'แก้ไขการตั้งค่าธุรกิจ');

-- ============================================================================
-- ROLE_PERMISSIONS (Junction table)
-- ============================================================================
CREATE TABLE role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE INDEX idx_role_permissions_role_id ON role_permissions(role_id);

-- ============================================================================
-- USERS
-- ============================================================================
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id),
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    phone VARCHAR(20),
    preferred_language VARCHAR(5) NOT NULL DEFAULT 'th' CHECK (preferred_language IN ('th', 'en')),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(business_id, email)
);

CREATE INDEX idx_users_business_id ON users(business_id);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role_id ON users(role_id);

-- ============================================================================
-- LINE OAUTH CONNECTIONS
-- ============================================================================
CREATE TABLE line_connections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    line_user_id VARCHAR(255) NOT NULL UNIQUE,
    display_name VARCHAR(255),
    picture_url TEXT,
    access_token TEXT,
    refresh_token TEXT,
    token_expires_at TIMESTAMPTZ,
    connected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_line_connections_user_id ON line_connections(user_id);
CREATE INDEX idx_line_connections_line_user_id ON line_connections(line_user_id);

-- ============================================================================
-- REFRESH TOKENS
-- ============================================================================
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    device_info TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);

CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);

-- ============================================================================
-- NOTIFICATION PREFERENCES
-- ============================================================================
CREATE TABLE notification_preferences (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE UNIQUE,
    line_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    in_app_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    low_inventory_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    certification_expiry_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    processing_milestone_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    weather_alert_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    harvest_reminder_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    quality_alert_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- AUDIT LOG
-- ============================================================================
CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(50) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID,
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_log_business_id ON audit_log(business_id);
CREATE INDEX idx_audit_log_user_id ON audit_log(user_id);
CREATE INDEX idx_audit_log_resource ON audit_log(resource_type, resource_id);
CREATE INDEX idx_audit_log_created_at ON audit_log(created_at);

-- ============================================================================
-- FUNCTIONS
-- ============================================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply updated_at trigger to all relevant tables
CREATE TRIGGER update_businesses_updated_at BEFORE UPDATE ON businesses
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_roles_updated_at BEFORE UPDATE ON roles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_line_connections_updated_at BEFORE UPDATE ON line_connections
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_notification_preferences_updated_at BEFORE UPDATE ON notification_preferences
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to create default roles for a new business
CREATE OR REPLACE FUNCTION create_default_roles()
RETURNS TRIGGER AS $$
DECLARE
    owner_role_id UUID;
    manager_role_id UUID;
    worker_role_id UUID;
    perm RECORD;
BEGIN
    -- Create Owner role (all permissions)
    INSERT INTO roles (business_id, name, name_th, description, description_th, is_system_role)
    VALUES (NEW.id, 'owner', 'เจ้าของ', 'Full access to all features', 'เข้าถึงทุกฟีเจอร์')
    RETURNING id INTO owner_role_id;
    
    -- Grant all permissions to owner
    INSERT INTO role_permissions (role_id, permission_id)
    SELECT owner_role_id, id FROM permissions;
    
    -- Create Manager role (most permissions except role/business management)
    INSERT INTO roles (business_id, name, name_th, description, description_th, is_system_role)
    VALUES (NEW.id, 'manager', 'ผู้จัดการ', 'Manage operations and users', 'จัดการการดำเนินงานและผู้ใช้')
    RETURNING id INTO manager_role_id;
    
    -- Grant manager permissions (exclude role and business management)
    INSERT INTO role_permissions (role_id, permission_id)
    SELECT manager_role_id, id FROM permissions
    WHERE resource NOT IN ('role', 'business') OR action = 'view';
    
    -- Create Worker role (basic operational permissions)
    INSERT INTO roles (business_id, name, name_th, description, description_th, is_system_role)
    VALUES (NEW.id, 'worker', 'พนักงาน', 'Basic operational access', 'เข้าถึงการดำเนินงานพื้นฐาน')
    RETURNING id INTO worker_role_id;
    
    -- Grant worker permissions (view and create only for operational resources)
    INSERT INTO role_permissions (role_id, permission_id)
    SELECT worker_role_id, id FROM permissions
    WHERE (resource IN ('plot', 'harvest', 'processing', 'grading', 'cupping', 'inventory', 'roast_profile')
           AND action IN ('view', 'create'))
       OR (resource = 'report' AND action = 'view');
    
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Trigger to create default roles when a business is created
CREATE TRIGGER create_business_default_roles
    AFTER INSERT ON businesses
    FOR EACH ROW EXECUTE FUNCTION create_default_roles();

-- ============================================================================
-- COMMENTS
-- ============================================================================
COMMENT ON TABLE businesses IS 'Coffee businesses (farms, processors, roasters)';
COMMENT ON TABLE roles IS 'User roles with customizable permissions';
COMMENT ON TABLE permissions IS 'Available permissions for role-based access control';
COMMENT ON TABLE role_permissions IS 'Junction table linking roles to permissions';
COMMENT ON TABLE users IS 'User accounts belonging to businesses';
COMMENT ON TABLE line_connections IS 'LINE OAuth connections for users';
COMMENT ON TABLE refresh_tokens IS 'JWT refresh tokens for session management';
COMMENT ON TABLE notification_preferences IS 'User notification settings';
COMMENT ON TABLE audit_log IS 'Audit trail for all data changes';
