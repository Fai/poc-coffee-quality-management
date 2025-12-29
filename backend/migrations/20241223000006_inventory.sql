-- Inventory management migration
-- Tracks inventory transactions, stage transitions, and alerts

-- Transaction types
CREATE TYPE inventory_transaction_type AS ENUM (
    'harvest_in',       -- Cherry received from harvest
    'processing_out',   -- Cherry sent to processing
    'processing_in',    -- Green bean received from processing
    'roasting_out',     -- Green bean sent to roasting
    'roasting_in',      -- Roasted bean received from roasting
    'sale',             -- Sold to customer
    'purchase',         -- Purchased from supplier
    'adjustment',       -- Manual adjustment (loss, damage, etc.)
    'transfer',         -- Transfer between locations
    'sample',           -- Sample taken for grading/cupping
    'return'            -- Returned from customer
);

-- Inventory transactions table
CREATE TABLE inventory_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    lot_id UUID NOT NULL REFERENCES lots(id) ON DELETE CASCADE,
    transaction_type inventory_transaction_type NOT NULL,
    quantity_kg DECIMAL(10, 3) NOT NULL,
    -- Positive for IN, negative for OUT
    direction VARCHAR(3) NOT NULL CHECK (direction IN ('in', 'out')),
    -- Stage at time of transaction
    stage VARCHAR(50) NOT NULL,
    -- Reference to related record (processing_id, grading_id, etc.)
    reference_type VARCHAR(50),
    reference_id UUID,
    -- Counterparty for sales/purchases
    counterparty_name VARCHAR(255),
    counterparty_contact VARCHAR(255),
    -- Pricing info
    unit_price DECIMAL(10, 2),
    total_price DECIMAL(12, 2),
    currency VARCHAR(3) DEFAULT 'THB',
    -- Notes
    notes TEXT,
    notes_th TEXT,
    -- Transaction date (may differ from created_at)
    transaction_date DATE NOT NULL,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id)
);

-- Inventory alerts configuration
CREATE TABLE inventory_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    -- Can be lot-specific or stage-specific
    lot_id UUID REFERENCES lots(id) ON DELETE CASCADE,
    stage VARCHAR(50),
    -- Alert threshold
    threshold_kg DECIMAL(10, 3) NOT NULL,
    -- Alert status
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_triggered_at TIMESTAMPTZ,
    -- Notification settings
    notify_email BOOLEAN DEFAULT true,
    notify_line BOOLEAN DEFAULT true,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Either lot_id or stage must be set
    CONSTRAINT alert_target_check CHECK (lot_id IS NOT NULL OR stage IS NOT NULL)
);

-- Inventory snapshots for quick balance lookup
CREATE TABLE inventory_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    lot_id UUID NOT NULL REFERENCES lots(id) ON DELETE CASCADE,
    stage VARCHAR(50) NOT NULL,
    quantity_kg DECIMAL(10, 3) NOT NULL,
    -- Valuation
    unit_cost DECIMAL(10, 2),
    total_value DECIMAL(12, 2),
    currency VARCHAR(3) DEFAULT 'THB',
    -- Snapshot date
    snapshot_date DATE NOT NULL,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Unique constraint per lot/stage/date
    UNIQUE(lot_id, stage, snapshot_date)
);

-- Indexes
CREATE INDEX idx_inventory_transactions_business_id ON inventory_transactions(business_id);
CREATE INDEX idx_inventory_transactions_lot_id ON inventory_transactions(lot_id);
CREATE INDEX idx_inventory_transactions_type ON inventory_transactions(transaction_type);
CREATE INDEX idx_inventory_transactions_date ON inventory_transactions(transaction_date);
CREATE INDEX idx_inventory_transactions_stage ON inventory_transactions(stage);
CREATE INDEX idx_inventory_alerts_business_id ON inventory_alerts(business_id);
CREATE INDEX idx_inventory_alerts_lot_id ON inventory_alerts(lot_id);
CREATE INDEX idx_inventory_alerts_active ON inventory_alerts(is_active) WHERE is_active = true;
CREATE INDEX idx_inventory_snapshots_business_id ON inventory_snapshots(business_id);
CREATE INDEX idx_inventory_snapshots_lot_id ON inventory_snapshots(lot_id);
CREATE INDEX idx_inventory_snapshots_date ON inventory_snapshots(snapshot_date);

-- Triggers for updated_at
CREATE TRIGGER update_inventory_alerts_updated_at
    BEFORE UPDATE ON inventory_alerts
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to calculate current inventory balance for a lot
CREATE OR REPLACE FUNCTION get_lot_inventory_balance(p_lot_id UUID)
RETURNS DECIMAL(10, 3) AS $$
DECLARE
    v_balance DECIMAL(10, 3);
BEGIN
    SELECT COALESCE(SUM(
        CASE WHEN direction = 'in' THEN quantity_kg ELSE -quantity_kg END
    ), 0)
    INTO v_balance
    FROM inventory_transactions
    WHERE lot_id = p_lot_id;
    
    RETURN v_balance;
END;
$$ LANGUAGE plpgsql;

-- Function to check and trigger low inventory alerts
CREATE OR REPLACE FUNCTION check_inventory_alerts()
RETURNS TRIGGER AS $$
DECLARE
    v_balance DECIMAL(10, 3);
    v_alert RECORD;
BEGIN
    -- Get current balance for the lot
    v_balance := get_lot_inventory_balance(NEW.lot_id);
    
    -- Check lot-specific alerts
    FOR v_alert IN 
        SELECT * FROM inventory_alerts 
        WHERE lot_id = NEW.lot_id 
        AND is_active = true
        AND threshold_kg >= v_balance
    LOOP
        -- Update last triggered timestamp
        UPDATE inventory_alerts 
        SET last_triggered_at = NOW() 
        WHERE id = v_alert.id;
        
        -- Note: Actual notification would be handled by application layer
    END LOOP;
    
    -- Check stage-specific alerts
    FOR v_alert IN 
        SELECT * FROM inventory_alerts 
        WHERE business_id = NEW.business_id
        AND stage = NEW.stage
        AND lot_id IS NULL
        AND is_active = true
        AND threshold_kg >= v_balance
    LOOP
        UPDATE inventory_alerts 
        SET last_triggered_at = NOW() 
        WHERE id = v_alert.id;
    END LOOP;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to check alerts after inventory transaction
CREATE TRIGGER check_inventory_alerts_trigger
    AFTER INSERT ON inventory_transactions
    FOR EACH ROW
    EXECUTE FUNCTION check_inventory_alerts();

-- Comments
COMMENT ON TABLE inventory_transactions IS 'Records all inventory movements for lots';
COMMENT ON TABLE inventory_alerts IS 'Configuration for low inventory alerts';
COMMENT ON TABLE inventory_snapshots IS 'Point-in-time inventory snapshots for reporting';
COMMENT ON COLUMN inventory_transactions.direction IS 'in = adds to inventory, out = removes from inventory';
COMMENT ON COLUMN inventory_transactions.reference_type IS 'Type of related record (processing, grading, sale, etc.)';
COMMENT ON FUNCTION get_lot_inventory_balance IS 'Calculates current inventory balance for a lot';
