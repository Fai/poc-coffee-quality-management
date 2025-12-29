-- Cupping session and scores migration
-- Implements SCA cupping protocol with 10 attributes

-- Cupping sessions table
CREATE TABLE cupping_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    session_date DATE NOT NULL,
    cupper_name VARCHAR(255) NOT NULL,
    location VARCHAR(255),
    notes TEXT,
    notes_th TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Cupping samples table (individual lot evaluations within a session)
CREATE TABLE cupping_samples (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES cupping_sessions(id) ON DELETE CASCADE,
    lot_id UUID NOT NULL REFERENCES lots(id) ON DELETE CASCADE,
    sample_number INTEGER NOT NULL,
    -- SCA Cupping Scores (6.0-10.0 scale, 0.25 increments for most)
    fragrance_aroma DECIMAL(4, 2) NOT NULL,
    flavor DECIMAL(4, 2) NOT NULL,
    aftertaste DECIMAL(4, 2) NOT NULL,
    acidity DECIMAL(4, 2) NOT NULL,
    body DECIMAL(4, 2) NOT NULL,
    balance DECIMAL(4, 2) NOT NULL,
    uniformity DECIMAL(4, 2) NOT NULL,      -- 10 points max (2 per cup)
    clean_cup DECIMAL(4, 2) NOT NULL,       -- 10 points max (2 per cup)
    sweetness DECIMAL(4, 2) NOT NULL,       -- 10 points max (2 per cup)
    overall DECIMAL(4, 2) NOT NULL,
    -- Calculated total score
    total_score DECIMAL(5, 2) NOT NULL,
    -- Tasting notes
    tasting_notes TEXT,
    tasting_notes_th TEXT,
    -- Defects (subtracted from total)
    defects_taint INTEGER NOT NULL DEFAULT 0,    -- 2 points each
    defects_fault INTEGER NOT NULL DEFAULT 0,    -- 4 points each
    -- Final score after defect deduction
    final_score DECIMAL(5, 2) NOT NULL,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Ensure unique sample number within session
    UNIQUE(session_id, sample_number)
);

-- Indexes
CREATE INDEX idx_cupping_sessions_business_id ON cupping_sessions(business_id);
CREATE INDEX idx_cupping_sessions_session_date ON cupping_sessions(session_date);
CREATE INDEX idx_cupping_samples_session_id ON cupping_samples(session_id);
CREATE INDEX idx_cupping_samples_lot_id ON cupping_samples(lot_id);
CREATE INDEX idx_cupping_samples_final_score ON cupping_samples(final_score);

-- Triggers for updated_at
CREATE TRIGGER update_cupping_sessions_updated_at
    BEFORE UPDATE ON cupping_sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_cupping_samples_updated_at
    BEFORE UPDATE ON cupping_samples
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comments
COMMENT ON TABLE cupping_sessions IS 'Cupping sessions following SCA protocol';
COMMENT ON TABLE cupping_samples IS 'Individual lot evaluations within a cupping session';
COMMENT ON COLUMN cupping_samples.fragrance_aroma IS 'Fragrance/Aroma score (6.0-10.0)';
COMMENT ON COLUMN cupping_samples.uniformity IS 'Uniformity score - 2 points per cup, max 10';
COMMENT ON COLUMN cupping_samples.clean_cup IS 'Clean cup score - 2 points per cup, max 10';
COMMENT ON COLUMN cupping_samples.sweetness IS 'Sweetness score - 2 points per cup, max 10';
COMMENT ON COLUMN cupping_samples.defects_taint IS 'Number of taint defects (2 points each)';
COMMENT ON COLUMN cupping_samples.defects_fault IS 'Number of fault defects (4 points each)';
COMMENT ON COLUMN cupping_samples.final_score IS 'Total score minus defect deductions';
