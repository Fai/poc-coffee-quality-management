-- Roast Profile Management Migration
-- Task 14: Roast session recording, profile templates, and cupping linkage

-- Roast profile templates table
CREATE TABLE IF NOT EXISTS roast_profile_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    name_th VARCHAR(255),
    description TEXT,
    description_th TEXT,
    -- Profile parameters
    target_first_crack_time_seconds INTEGER,
    target_first_crack_temp_celsius DECIMAL(5, 1),
    target_development_time_seconds INTEGER,
    target_end_temp_celsius DECIMAL(5, 1),
    target_total_time_seconds INTEGER,
    target_weight_loss_percent DECIMAL(5, 2),
    -- Temperature checkpoints as JSONB array
    -- Format: [{"time_seconds": 60, "temp_celsius": 150.0}, ...]
    temperature_profile JSONB DEFAULT '[]'::jsonb,
    -- Roast level
    roast_level VARCHAR(50), -- light, medium_light, medium, medium_dark, dark
    -- Equipment recommendations
    recommended_equipment VARCHAR(255),
    -- Metadata
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id)
);

-- Roast sessions table
CREATE TABLE IF NOT EXISTS roast_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    lot_id UUID NOT NULL REFERENCES lots(id) ON DELETE CASCADE,
    template_id UUID REFERENCES roast_profile_templates(id),
    -- Session details
    session_date DATE NOT NULL,
    roaster_name VARCHAR(255) NOT NULL,
    equipment VARCHAR(255),
    -- Input
    green_bean_weight_kg DECIMAL(10, 3) NOT NULL,
    initial_moisture_percent DECIMAL(5, 2),
    -- Temperature/time logging as JSONB array
    -- Format: [{"time_seconds": 0, "temp_celsius": 25.0, "notes": "charge"}, ...]
    temperature_log JSONB DEFAULT '[]'::jsonb,
    -- Key milestones
    charge_temp_celsius DECIMAL(5, 1),
    turning_point_time_seconds INTEGER,
    turning_point_temp_celsius DECIMAL(5, 1),
    first_crack_time_seconds INTEGER,
    first_crack_temp_celsius DECIMAL(5, 1),
    second_crack_time_seconds INTEGER,
    second_crack_temp_celsius DECIMAL(5, 1),
    drop_time_seconds INTEGER,
    drop_temp_celsius DECIMAL(5, 1),
    -- Output (filled on completion)
    roasted_weight_kg DECIMAL(10, 3),
    weight_loss_percent DECIMAL(5, 2),
    final_moisture_percent DECIMAL(5, 2),
    -- Development metrics
    development_time_seconds INTEGER,
    development_time_ratio DECIMAL(5, 2), -- DTR = development_time / total_time * 100
    -- Roast level assessment
    roast_level VARCHAR(50),
    color_value DECIMAL(5, 1), -- Agtron or similar scale
    -- Status
    status VARCHAR(50) NOT NULL DEFAULT 'in_progress', -- in_progress, completed, failed
    -- Notes
    notes TEXT,
    notes_th TEXT,
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id)
);

-- Link cupping samples to roast sessions
ALTER TABLE cupping_samples 
ADD COLUMN IF NOT EXISTS roast_session_id UUID REFERENCES roast_sessions(id);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_roast_profile_templates_business 
    ON roast_profile_templates(business_id);
CREATE INDEX IF NOT EXISTS idx_roast_profile_templates_active 
    ON roast_profile_templates(business_id, is_active) WHERE is_active = true;

CREATE INDEX IF NOT EXISTS idx_roast_sessions_business 
    ON roast_sessions(business_id);
CREATE INDEX IF NOT EXISTS idx_roast_sessions_lot 
    ON roast_sessions(lot_id);
CREATE INDEX IF NOT EXISTS idx_roast_sessions_template 
    ON roast_sessions(template_id);
CREATE INDEX IF NOT EXISTS idx_roast_sessions_date 
    ON roast_sessions(business_id, session_date DESC);
CREATE INDEX IF NOT EXISTS idx_roast_sessions_status 
    ON roast_sessions(business_id, status);

CREATE INDEX IF NOT EXISTS idx_cupping_samples_roast_session 
    ON cupping_samples(roast_session_id) WHERE roast_session_id IS NOT NULL;

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_roast_profile_templates_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_roast_profile_templates_updated_at
    BEFORE UPDATE ON roast_profile_templates
    FOR EACH ROW
    EXECUTE FUNCTION update_roast_profile_templates_updated_at();

CREATE OR REPLACE FUNCTION update_roast_sessions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_roast_sessions_updated_at
    BEFORE UPDATE ON roast_sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_roast_sessions_updated_at();

-- Function to calculate weight loss percentage
CREATE OR REPLACE FUNCTION calculate_weight_loss_percent(
    green_weight DECIMAL,
    roasted_weight DECIMAL
) RETURNS DECIMAL AS $$
BEGIN
    IF green_weight IS NULL OR green_weight <= 0 THEN
        RETURN NULL;
    END IF;
    IF roasted_weight IS NULL THEN
        RETURN NULL;
    END IF;
    RETURN ROUND(((green_weight - roasted_weight) / green_weight) * 100, 2);
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Function to calculate development time ratio
CREATE OR REPLACE FUNCTION calculate_dtr(
    development_time INTEGER,
    total_time INTEGER
) RETURNS DECIMAL AS $$
BEGIN
    IF total_time IS NULL OR total_time <= 0 THEN
        RETURN NULL;
    END IF;
    IF development_time IS NULL THEN
        RETURN NULL;
    END IF;
    RETURN ROUND((development_time::DECIMAL / total_time::DECIMAL) * 100, 2);
END;
$$ LANGUAGE plpgsql IMMUTABLE;
