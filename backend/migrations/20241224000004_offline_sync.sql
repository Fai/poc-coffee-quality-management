-- Offline sync infrastructure for PWA support
-- Tracks changes for delta sync and handles conflicts

-- ============================================================================
-- SYNC LOG - Tracks all entity changes for delta sync
-- ============================================================================
CREATE TABLE sync_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    operation VARCHAR(20) NOT NULL CHECK (operation IN ('create', 'update', 'delete')),
    entity_version BIGINT NOT NULL DEFAULT 1,
    data JSONB, -- Snapshot of entity at time of change
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    synced_by_users UUID[] DEFAULT '{}'::UUID[] -- Track which users have synced this change
);

CREATE INDEX idx_sync_log_business_changed ON sync_log(business_id, changed_at);
CREATE INDEX idx_sync_log_entity ON sync_log(entity_type, entity_id);

-- ============================================================================
-- SYNC CONFLICTS - Stores conflicts for user resolution
-- ============================================================================
CREATE TABLE sync_conflicts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    local_version JSONB NOT NULL,
    local_changed_at TIMESTAMPTZ NOT NULL,
    server_version JSONB NOT NULL,
    server_changed_at TIMESTAMPTZ NOT NULL,
    server_entity_version BIGINT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'resolved_local', 'resolved_server', 'resolved_merged')),
    resolved_at TIMESTAMPTZ,
    resolved_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sync_conflicts_user_status ON sync_conflicts(user_id, status);

-- ============================================================================
-- SYNC STATE - Tracks last sync per user/device
-- ============================================================================
CREATE TABLE sync_state (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id VARCHAR(100) NOT NULL,
    last_sync_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_sync_version BIGINT NOT NULL DEFAULT 0,
    UNIQUE(user_id, device_id)
);

-- ============================================================================
-- ADD VERSION COLUMNS TO SYNCABLE ENTITIES
-- ============================================================================
ALTER TABLE plots ADD COLUMN IF NOT EXISTS entity_version BIGINT NOT NULL DEFAULT 1;
ALTER TABLE lots ADD COLUMN IF NOT EXISTS entity_version BIGINT NOT NULL DEFAULT 1;
ALTER TABLE harvests ADD COLUMN IF NOT EXISTS entity_version BIGINT NOT NULL DEFAULT 1;
ALTER TABLE processing_records ADD COLUMN IF NOT EXISTS entity_version BIGINT NOT NULL DEFAULT 1;
ALTER TABLE green_bean_grades ADD COLUMN IF NOT EXISTS entity_version BIGINT NOT NULL DEFAULT 1;
ALTER TABLE cupping_sessions ADD COLUMN IF NOT EXISTS entity_version BIGINT NOT NULL DEFAULT 1;
ALTER TABLE cupping_samples ADD COLUMN IF NOT EXISTS entity_version BIGINT NOT NULL DEFAULT 1;
ALTER TABLE inventory_transactions ADD COLUMN IF NOT EXISTS entity_version BIGINT NOT NULL DEFAULT 1;
ALTER TABLE roast_sessions ADD COLUMN IF NOT EXISTS entity_version BIGINT NOT NULL DEFAULT 1;

-- ============================================================================
-- GLOBAL SYNC VERSION SEQUENCE
-- ============================================================================
CREATE SEQUENCE IF NOT EXISTS global_sync_version START 1;

-- ============================================================================
-- FUNCTION: Log entity change to sync_log
-- ============================================================================
CREATE OR REPLACE FUNCTION log_entity_change()
RETURNS TRIGGER AS $$
DECLARE
    v_business_id UUID;
    v_operation VARCHAR(20);
    v_data JSONB;
    v_new_version BIGINT;
BEGIN
    -- Determine operation type
    IF TG_OP = 'INSERT' THEN
        v_operation := 'create';
        v_data := to_jsonb(NEW);
        v_business_id := NEW.business_id;
        v_new_version := nextval('global_sync_version');
        NEW.entity_version := v_new_version;
    ELSIF TG_OP = 'UPDATE' THEN
        v_operation := 'update';
        v_data := to_jsonb(NEW);
        v_business_id := NEW.business_id;
        v_new_version := nextval('global_sync_version');
        NEW.entity_version := v_new_version;
    ELSIF TG_OP = 'DELETE' THEN
        v_operation := 'delete';
        v_data := to_jsonb(OLD);
        v_business_id := OLD.business_id;
        v_new_version := nextval('global_sync_version');
    END IF;

    -- Insert into sync_log
    INSERT INTO sync_log (business_id, entity_type, entity_id, operation, entity_version, data)
    VALUES (
        v_business_id,
        TG_TABLE_NAME,
        CASE WHEN TG_OP = 'DELETE' THEN OLD.id ELSE NEW.id END,
        v_operation,
        v_new_version,
        v_data
    );

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    ELSE
        RETURN NEW;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- TRIGGERS: Apply sync logging to syncable tables
-- ============================================================================
CREATE TRIGGER trg_plots_sync BEFORE INSERT OR UPDATE OR DELETE ON plots
    FOR EACH ROW EXECUTE FUNCTION log_entity_change();

CREATE TRIGGER trg_harvests_sync BEFORE INSERT OR UPDATE OR DELETE ON harvests
    FOR EACH ROW EXECUTE FUNCTION log_entity_change();

CREATE TRIGGER trg_processing_records_sync BEFORE INSERT OR UPDATE OR DELETE ON processing_records
    FOR EACH ROW EXECUTE FUNCTION log_entity_change();

CREATE TRIGGER trg_green_bean_grades_sync BEFORE INSERT OR UPDATE OR DELETE ON green_bean_grades
    FOR EACH ROW EXECUTE FUNCTION log_entity_change();

CREATE TRIGGER trg_cupping_sessions_sync BEFORE INSERT OR UPDATE OR DELETE ON cupping_sessions
    FOR EACH ROW EXECUTE FUNCTION log_entity_change();

CREATE TRIGGER trg_inventory_transactions_sync BEFORE INSERT OR UPDATE OR DELETE ON inventory_transactions
    FOR EACH ROW EXECUTE FUNCTION log_entity_change();

CREATE TRIGGER trg_roast_sessions_sync BEFORE INSERT OR UPDATE OR DELETE ON roast_sessions
    FOR EACH ROW EXECUTE FUNCTION log_entity_change();

-- ============================================================================
-- FUNCTION: Get changes since last sync
-- ============================================================================
CREATE OR REPLACE FUNCTION get_changes_since(
    p_business_id UUID,
    p_since_version BIGINT,
    p_limit INT DEFAULT 1000
)
RETURNS TABLE (
    entity_type VARCHAR(50),
    entity_id UUID,
    operation VARCHAR(20),
    entity_version BIGINT,
    data JSONB,
    changed_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        sl.entity_type,
        sl.entity_id,
        sl.operation,
        sl.entity_version,
        sl.data,
        sl.changed_at
    FROM sync_log sl
    WHERE sl.business_id = p_business_id
      AND sl.entity_version > p_since_version
    ORDER BY sl.entity_version ASC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- FUNCTION: Check for conflicts before applying client changes
-- ============================================================================
CREATE OR REPLACE FUNCTION check_sync_conflict(
    p_entity_type VARCHAR(50),
    p_entity_id UUID,
    p_client_version BIGINT
)
RETURNS TABLE (
    has_conflict BOOLEAN,
    server_version BIGINT,
    server_data JSONB
) AS $$
DECLARE
    v_current_version BIGINT;
    v_current_data JSONB;
BEGIN
    -- Get current server version
    EXECUTE format(
        'SELECT entity_version, to_jsonb(t.*) FROM %I t WHERE id = $1',
        p_entity_type
    ) INTO v_current_version, v_current_data USING p_entity_id;

    IF v_current_version IS NULL THEN
        -- Entity doesn't exist (might be deleted or new)
        RETURN QUERY SELECT FALSE, NULL::BIGINT, NULL::JSONB;
    ELSIF v_current_version > p_client_version THEN
        -- Conflict: server has newer version
        RETURN QUERY SELECT TRUE, v_current_version, v_current_data;
    ELSE
        -- No conflict
        RETURN QUERY SELECT FALSE, v_current_version, v_current_data;
    END IF;
END;
$$ LANGUAGE plpgsql;
