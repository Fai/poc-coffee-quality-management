-- Green bean grading migration
-- Implements SCA grading standards with optional AI defect detection

CREATE TABLE green_bean_grades (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lot_id UUID NOT NULL REFERENCES lots(id) ON DELETE CASCADE,
    grading_date DATE NOT NULL,
    grader_name VARCHAR(255) NOT NULL,
    sample_weight_grams DECIMAL(10, 2) NOT NULL,
    -- Defect counts
    category1_count INTEGER NOT NULL DEFAULT 0,
    category2_count INTEGER NOT NULL DEFAULT 0,
    defect_breakdown JSONB,
    -- AI detection results (optional)
    ai_detection JSONB,
    -- Quality metrics
    moisture_percent DECIMAL(5, 2) NOT NULL,
    density DECIMAL(10, 4),
    screen_size_distribution JSONB,
    -- Grade classification (calculated from defects)
    grade VARCHAR(50) NOT NULL,
    -- Notes
    notes TEXT,
    notes_th TEXT,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for common queries
CREATE INDEX idx_green_bean_grades_lot_id ON green_bean_grades(lot_id);
CREATE INDEX idx_green_bean_grades_grading_date ON green_bean_grades(grading_date);
CREATE INDEX idx_green_bean_grades_grade ON green_bean_grades(grade);

-- Trigger to update updated_at
CREATE TRIGGER update_green_bean_grades_updated_at
    BEFORE UPDATE ON green_bean_grades
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comments for documentation
COMMENT ON TABLE green_bean_grades IS 'Green bean grading records following SCA standards';
COMMENT ON COLUMN green_bean_grades.category1_count IS 'Primary defects (full black, full sour, etc.)';
COMMENT ON COLUMN green_bean_grades.category2_count IS 'Secondary defects (partial black, broken, etc.)';
COMMENT ON COLUMN green_bean_grades.defect_breakdown IS 'Detailed breakdown by defect type';
COMMENT ON COLUMN green_bean_grades.ai_detection IS 'AI-assisted defect detection results';
COMMENT ON COLUMN green_bean_grades.grade IS 'SCA grade: specialty_grade, premium_grade, exchange_grade, below_standard, off_grade';
