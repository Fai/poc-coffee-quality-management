-- Migration: Add plots, harvests, and lots tables
-- Requirements: 2.1, 2.2, 3.1, 3.2, 7.1

-- ============================================================================
-- Plots Table (Farm Management)
-- ============================================================================

CREATE TABLE plots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    -- GPS coordinates stored as latitude/longitude
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    -- Area in rai (Thai unit: 1 rai = 1,600 sq meters)
    area_rai DECIMAL(10, 2),
    -- Altitude in meters above sea level
    altitude_meters INTEGER,
    -- Shade coverage percentage (0-100)
    shade_coverage_percent INTEGER CHECK (shade_coverage_percent >= 0 AND shade_coverage_percent <= 100),
    -- Additional notes
    notes TEXT,
    notes_th TEXT,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Ensure unique plot names within a business
    CONSTRAINT unique_plot_name_per_business UNIQUE (business_id, name)
);

-- Index for business queries
CREATE INDEX idx_plots_business_id ON plots(business_id);

-- ============================================================================
-- Plot Varieties Junction Table
-- ============================================================================

CREATE TABLE plot_varieties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plot_id UUID NOT NULL REFERENCES plots(id) ON DELETE CASCADE,
    variety VARCHAR(100) NOT NULL,
    variety_th VARCHAR(100),
    planting_date DATE,
    tree_count INTEGER,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Ensure unique variety per plot
    CONSTRAINT unique_variety_per_plot UNIQUE (plot_id, variety)
);

-- Index for plot queries
CREATE INDEX idx_plot_varieties_plot_id ON plot_varieties(plot_id);

-- ============================================================================
-- Lots Table (Traceability)
-- ============================================================================

CREATE TABLE lots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    -- Unique traceability code: CQM-YYYY-BIZ-NNNN
    traceability_code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    -- Current stage in the supply chain
    stage VARCHAR(50) NOT NULL DEFAULT 'cherry',
    -- Current weight in kg
    current_weight_kg DECIMAL(10, 3) NOT NULL DEFAULT 0,
    -- QR code URL for public traceability page
    qr_code_url VARCHAR(500),
    -- Notes
    notes TEXT,
    notes_th TEXT,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Validate stage values
    CONSTRAINT valid_lot_stage CHECK (stage IN ('cherry', 'parchment', 'green_bean', 'roasted_bean', 'sold'))
);

-- Indexes for lot queries
CREATE INDEX idx_lots_business_id ON lots(business_id);
CREATE INDEX idx_lots_traceability_code ON lots(traceability_code);
CREATE INDEX idx_lots_stage ON lots(stage);

-- ============================================================================
-- Lot Sources Table (For Blended Lots)
-- ============================================================================

CREATE TABLE lot_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lot_id UUID NOT NULL REFERENCES lots(id) ON DELETE CASCADE,
    source_lot_id UUID NOT NULL REFERENCES lots(id) ON DELETE RESTRICT,
    -- Proportion percentage (must sum to 100 for all sources of a lot)
    proportion_percent DECIMAL(5, 2) NOT NULL CHECK (proportion_percent > 0 AND proportion_percent <= 100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Ensure unique source per lot
    CONSTRAINT unique_source_per_lot UNIQUE (lot_id, source_lot_id),
    -- Prevent self-reference
    CONSTRAINT no_self_reference CHECK (lot_id != source_lot_id)
);

-- Index for lot source queries
CREATE INDEX idx_lot_sources_lot_id ON lot_sources(lot_id);
CREATE INDEX idx_lot_sources_source_lot_id ON lot_sources(source_lot_id);

-- ============================================================================
-- Harvests Table
-- ============================================================================

CREATE TABLE harvests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lot_id UUID NOT NULL REFERENCES lots(id) ON DELETE CASCADE,
    plot_id UUID NOT NULL REFERENCES plots(id) ON DELETE RESTRICT,
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    -- Harvest details
    harvest_date DATE NOT NULL,
    picker_name VARCHAR(255),
    -- Cherry weight in kg
    cherry_weight_kg DECIMAL(10, 3) NOT NULL CHECK (cherry_weight_kg > 0),
    -- Ripeness assessment (percentages must sum to 100)
    underripe_percent INTEGER NOT NULL DEFAULT 0 CHECK (underripe_percent >= 0 AND underripe_percent <= 100),
    ripe_percent INTEGER NOT NULL DEFAULT 0 CHECK (ripe_percent >= 0 AND ripe_percent <= 100),
    overripe_percent INTEGER NOT NULL DEFAULT 0 CHECK (overripe_percent >= 0 AND overripe_percent <= 100),
    -- Weather snapshot at harvest time (JSONB for flexibility)
    weather_snapshot JSONB,
    -- Notes
    notes TEXT,
    notes_th TEXT,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Ensure ripeness percentages sum to 100
    CONSTRAINT ripeness_sum_100 CHECK (underripe_percent + ripe_percent + overripe_percent = 100)
);

-- Indexes for harvest queries
CREATE INDEX idx_harvests_lot_id ON harvests(lot_id);
CREATE INDEX idx_harvests_plot_id ON harvests(plot_id);
CREATE INDEX idx_harvests_business_id ON harvests(business_id);
CREATE INDEX idx_harvests_date ON harvests(harvest_date);

-- ============================================================================
-- Media Table (Photos for plots, harvests, etc.)
-- ============================================================================

CREATE TABLE media (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    -- Entity reference (polymorphic)
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    -- File information
    file_type VARCHAR(50) NOT NULL,
    s3_key VARCHAR(500) NOT NULL,
    original_filename VARCHAR(255),
    file_size_bytes BIGINT,
    -- Metadata
    description TEXT,
    description_th TEXT,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Validate entity types
    CONSTRAINT valid_entity_type CHECK (entity_type IN ('plot', 'harvest', 'processing', 'grading', 'certification'))
);

-- Indexes for media queries
CREATE INDEX idx_media_business_id ON media(business_id);
CREATE INDEX idx_media_entity ON media(entity_type, entity_id);

-- ============================================================================
-- Lot Sequence Table (For generating unique lot numbers per business per year)
-- ============================================================================

CREATE TABLE lot_sequences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    year INTEGER NOT NULL,
    last_sequence INTEGER NOT NULL DEFAULT 0,
    
    CONSTRAINT unique_sequence_per_business_year UNIQUE (business_id, year)
);

-- ============================================================================
-- Function to generate next lot sequence number
-- ============================================================================

CREATE OR REPLACE FUNCTION get_next_lot_sequence(p_business_id UUID, p_year INTEGER)
RETURNS INTEGER AS $$
DECLARE
    v_sequence INTEGER;
BEGIN
    -- Insert or update the sequence
    INSERT INTO lot_sequences (business_id, year, last_sequence)
    VALUES (p_business_id, p_year, 1)
    ON CONFLICT (business_id, year)
    DO UPDATE SET last_sequence = lot_sequences.last_sequence + 1
    RETURNING last_sequence INTO v_sequence;
    
    RETURN v_sequence;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- Trigger to update updated_at timestamp
-- ============================================================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_plots_updated_at
    BEFORE UPDATE ON plots
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_lots_updated_at
    BEFORE UPDATE ON lots
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_harvests_updated_at
    BEFORE UPDATE ON harvests
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
