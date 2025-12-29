-- Processing records migration
-- Tracks coffee processing from cherry to green bean

-- Processing records table
CREATE TABLE processing_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lot_id UUID NOT NULL REFERENCES lots(id) ON DELETE CASCADE,
    method VARCHAR(50) NOT NULL,
    method_details JSONB, -- For honey mucilage_percent, anaerobic hours, etc.
    start_date DATE NOT NULL,
    end_date DATE,
    responsible_person VARCHAR(255) NOT NULL,
    fermentation_log JSONB, -- FermentationLog struct
    drying_log JSONB, -- DryingLog struct
    final_moisture_percent DECIMAL(5,2),
    green_bean_weight_kg DECIMAL(10,3),
    cherry_weight_kg DECIMAL(10,3), -- Snapshot of lot weight at processing start
    processing_yield_percent DECIMAL(5,2), -- Calculated: (green_bean / cherry) * 100
    notes TEXT,
    notes_th TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_processing_records_lot ON processing_records(lot_id);
CREATE INDEX idx_processing_records_start_date ON processing_records(start_date);
CREATE INDEX idx_processing_records_method ON processing_records(method);

-- Trigger for updated_at
CREATE TRIGGER update_processing_records_updated_at
    BEFORE UPDATE ON processing_records
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comments
COMMENT ON TABLE processing_records IS 'Coffee processing records from cherry to green bean';
COMMENT ON COLUMN processing_records.method IS 'Processing method: natural, washed, honey, wet_hulled, anaerobic, custom';
COMMENT ON COLUMN processing_records.method_details IS 'Additional method details (e.g., mucilage_percent for honey)';
COMMENT ON COLUMN processing_records.fermentation_log IS 'JSONB containing duration, temperature and pH readings';
COMMENT ON COLUMN processing_records.drying_log IS 'JSONB containing drying method, dates, and moisture readings';
COMMENT ON COLUMN processing_records.processing_yield_percent IS 'Yield = (green_bean_weight / cherry_weight) * 100';
