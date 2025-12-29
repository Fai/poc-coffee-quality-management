-- Certification Management Migration
-- Supports tracking of Thai GAP, Organic Thailand, USDA Organic, Fair Trade, Rainforest Alliance, UTZ

-- Certification types enum
CREATE TYPE certification_type AS ENUM (
    'thai_gap',
    'organic_thailand',
    'usda_organic',
    'fair_trade',
    'rainforest_alliance',
    'utz',
    'other'
);

-- Certification scope enum (what the certification applies to)
CREATE TYPE certification_scope AS ENUM (
    'farm',
    'plot',
    'facility',
    'business'
);

-- Certifications table
CREATE TABLE certifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    -- Certification details
    certification_type certification_type NOT NULL,
    certification_name VARCHAR(255) NOT NULL,
    certification_body VARCHAR(255) NOT NULL,
    certificate_number VARCHAR(100) NOT NULL,
    
    -- Scope
    scope certification_scope NOT NULL DEFAULT 'business',
    plot_id UUID REFERENCES plots(id) ON DELETE SET NULL,
    
    -- Dates
    issue_date DATE NOT NULL,
    expiration_date DATE NOT NULL,
    
    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,
    
    -- Additional info
    notes TEXT,
    notes_th TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT valid_dates CHECK (expiration_date > issue_date)
);

-- Certification documents table
CREATE TABLE certification_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    certification_id UUID NOT NULL REFERENCES certifications(id) ON DELETE CASCADE,
    
    -- Document details
    document_type VARCHAR(50) NOT NULL, -- 'certificate', 'audit_report', 'checklist', 'other'
    document_name VARCHAR(255) NOT NULL,
    file_url TEXT NOT NULL,
    file_size_bytes BIGINT,
    mime_type VARCHAR(100),
    
    -- Timestamps
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    uploaded_by UUID REFERENCES users(id) ON DELETE SET NULL
);

-- Certification requirements/checklists table
CREATE TABLE certification_requirements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    certification_type certification_type NOT NULL,
    
    -- Requirement details
    requirement_code VARCHAR(50) NOT NULL,
    requirement_name VARCHAR(255) NOT NULL,
    requirement_name_th VARCHAR(255),
    description TEXT,
    description_th TEXT,
    category VARCHAR(100),
    
    -- Order for display
    display_order INT NOT NULL DEFAULT 0,
    
    -- Is this a critical requirement?
    is_critical BOOLEAN NOT NULL DEFAULT false,
    
    UNIQUE(certification_type, requirement_code)
);

-- Certification compliance tracking (per business)
CREATE TABLE certification_compliance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    certification_id UUID NOT NULL REFERENCES certifications(id) ON DELETE CASCADE,
    requirement_id UUID NOT NULL REFERENCES certification_requirements(id) ON DELETE CASCADE,
    
    -- Compliance status
    is_compliant BOOLEAN,
    compliance_notes TEXT,
    evidence_url TEXT,
    
    -- Verification
    verified_at TIMESTAMPTZ,
    verified_by UUID REFERENCES users(id) ON DELETE SET NULL,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(certification_id, requirement_id)
);

-- Certification expiration alerts table
CREATE TABLE certification_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    certification_id UUID NOT NULL REFERENCES certifications(id) ON DELETE CASCADE,
    
    -- Alert details
    alert_days_before INT NOT NULL, -- 90, 60, 30
    alert_sent_at TIMESTAMPTZ,
    
    -- Notification preferences
    notify_email BOOLEAN NOT NULL DEFAULT true,
    notify_line BOOLEAN NOT NULL DEFAULT true,
    
    UNIQUE(certification_id, alert_days_before)
);

-- Indexes
CREATE INDEX idx_certifications_business ON certifications(business_id);
CREATE INDEX idx_certifications_type ON certifications(certification_type);
CREATE INDEX idx_certifications_expiration ON certifications(expiration_date);
CREATE INDEX idx_certifications_active ON certifications(is_active) WHERE is_active = true;
CREATE INDEX idx_certification_documents_cert ON certification_documents(certification_id);
CREATE INDEX idx_certification_requirements_type ON certification_requirements(certification_type);
CREATE INDEX idx_certification_compliance_cert ON certification_compliance(certification_id);
CREATE INDEX idx_certification_alerts_cert ON certification_alerts(certification_id);

-- Trigger for updated_at
CREATE TRIGGER update_certifications_updated_at
    BEFORE UPDATE ON certifications
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_certification_compliance_updated_at
    BEFORE UPDATE ON certification_compliance
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Insert default requirements for Thai GAP
INSERT INTO certification_requirements (certification_type, requirement_code, requirement_name, requirement_name_th, category, display_order, is_critical) VALUES
('thai_gap', 'TG-01', 'Water source management', 'การจัดการแหล่งน้ำ', 'Water', 1, true),
('thai_gap', 'TG-02', 'Planting area management', 'การจัดการพื้นที่ปลูก', 'Land', 2, true),
('thai_gap', 'TG-03', 'Fertilizer usage records', 'บันทึกการใช้ปุ๋ย', 'Inputs', 3, false),
('thai_gap', 'TG-04', 'Pesticide usage records', 'บันทึกการใช้สารเคมี', 'Inputs', 4, true),
('thai_gap', 'TG-05', 'Harvest hygiene practices', 'สุขอนามัยในการเก็บเกี่ยว', 'Harvest', 5, true),
('thai_gap', 'TG-06', 'Post-harvest handling', 'การจัดการหลังการเก็บเกี่ยว', 'Post-harvest', 6, false),
('thai_gap', 'TG-07', 'Storage conditions', 'สภาพการเก็บรักษา', 'Storage', 7, false),
('thai_gap', 'TG-08', 'Worker safety training', 'การฝึกอบรมความปลอดภัยของคนงาน', 'Safety', 8, false);

-- Insert default requirements for Organic Thailand
INSERT INTO certification_requirements (certification_type, requirement_code, requirement_name, requirement_name_th, category, display_order, is_critical) VALUES
('organic_thailand', 'OT-01', 'No synthetic pesticides', 'ไม่ใช้สารเคมีสังเคราะห์', 'Inputs', 1, true),
('organic_thailand', 'OT-02', 'No synthetic fertilizers', 'ไม่ใช้ปุ๋ยเคมีสังเคราะห์', 'Inputs', 2, true),
('organic_thailand', 'OT-03', 'Buffer zones maintained', 'รักษาพื้นที่กันชน', 'Land', 3, true),
('organic_thailand', 'OT-04', 'Organic seed/seedlings', 'เมล็ดพันธุ์/กล้าอินทรีย์', 'Inputs', 4, false),
('organic_thailand', 'OT-05', 'Soil management plan', 'แผนการจัดการดิน', 'Land', 5, false),
('organic_thailand', 'OT-06', 'Record keeping', 'การบันทึกข้อมูล', 'Documentation', 6, true),
('organic_thailand', 'OT-07', 'Conversion period completed', 'ระยะเปลี่ยนผ่านเสร็จสิ้น', 'Certification', 7, true);

-- Function to check certification expiration and create alerts
CREATE OR REPLACE FUNCTION check_certification_expiration()
RETURNS TRIGGER AS $$
BEGIN
    -- Create alerts for 90, 60, 30 days before expiration
    INSERT INTO certification_alerts (certification_id, alert_days_before)
    VALUES 
        (NEW.id, 90),
        (NEW.id, 60),
        (NEW.id, 30)
    ON CONFLICT (certification_id, alert_days_before) DO NOTHING;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER create_certification_alerts
    AFTER INSERT ON certifications
    FOR EACH ROW
    EXECUTE FUNCTION check_certification_expiration();

-- Function to get certifications expiring soon
CREATE OR REPLACE FUNCTION get_expiring_certifications(
    p_business_id UUID,
    p_days_ahead INT DEFAULT 90
)
RETURNS TABLE (
    certification_id UUID,
    certification_name VARCHAR(255),
    certification_type certification_type,
    expiration_date DATE,
    days_until_expiration INT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        c.id,
        c.certification_name,
        c.certification_type,
        c.expiration_date,
        (c.expiration_date - CURRENT_DATE)::INT as days_until_expiration
    FROM certifications c
    WHERE c.business_id = p_business_id
      AND c.is_active = true
      AND c.expiration_date <= CURRENT_DATE + p_days_ahead
      AND c.expiration_date >= CURRENT_DATE
    ORDER BY c.expiration_date ASC;
END;
$$ LANGUAGE plpgsql;
