-- Notification Service Migration
-- Supports LINE and in-app notifications with preference management

-- Notification types enum
CREATE TYPE notification_type AS ENUM (
    'low_inventory',
    'certification_expiring',
    'processing_milestone',
    'weather_alert',
    'harvest_reminder',
    'quality_alert',
    'system'
);

-- Notification channel enum
CREATE TYPE notification_channel AS ENUM (
    'line',
    'in_app',
    'email'
);

-- Notification status enum
CREATE TYPE notification_status AS ENUM (
    'pending',
    'sent',
    'failed',
    'read'
);

-- Notification queue table (pending notifications)
CREATE TABLE notification_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    -- Notification details
    notification_type notification_type NOT NULL,
    title VARCHAR(255) NOT NULL,
    title_th VARCHAR(255),
    message TEXT NOT NULL,
    message_th TEXT,
    
    -- Reference to related entity
    entity_type VARCHAR(50),
    entity_id UUID,
    
    -- Scheduling
    scheduled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    priority INT NOT NULL DEFAULT 0, -- Higher = more urgent
    
    -- Status
    status notification_status NOT NULL DEFAULT 'pending',
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Notification log table (sent notifications)
CREATE TABLE notification_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    -- Notification details
    notification_type notification_type NOT NULL,
    channel notification_channel NOT NULL,
    title VARCHAR(255) NOT NULL,
    title_th VARCHAR(255),
    message TEXT NOT NULL,
    message_th TEXT,
    
    -- Reference to related entity
    entity_type VARCHAR(50),
    entity_id UUID,
    
    -- Delivery status
    status notification_status NOT NULL DEFAULT 'sent',
    error_message TEXT,
    
    -- LINE specific
    line_message_id VARCHAR(255),
    
    -- Timestamps
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- In-app notifications table (for UI display)
CREATE TABLE in_app_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    -- Notification details
    notification_type notification_type NOT NULL,
    title VARCHAR(255) NOT NULL,
    title_th VARCHAR(255),
    message TEXT NOT NULL,
    message_th TEXT,
    
    -- Reference to related entity
    entity_type VARCHAR(50),
    entity_id UUID,
    action_url VARCHAR(500),
    
    -- Status
    is_read BOOLEAN NOT NULL DEFAULT false,
    is_dismissed BOOLEAN NOT NULL DEFAULT false,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    read_at TIMESTAMPTZ
);

-- Update notification_preferences table to add per-type settings
-- (The table already exists from initial migration, we add columns)
ALTER TABLE notification_preferences
ADD COLUMN IF NOT EXISTS low_inventory_enabled BOOLEAN NOT NULL DEFAULT true,
ADD COLUMN IF NOT EXISTS certification_expiring_enabled BOOLEAN NOT NULL DEFAULT true,
ADD COLUMN IF NOT EXISTS processing_milestone_enabled BOOLEAN NOT NULL DEFAULT true,
ADD COLUMN IF NOT EXISTS weather_alert_enabled BOOLEAN NOT NULL DEFAULT true,
ADD COLUMN IF NOT EXISTS harvest_reminder_enabled BOOLEAN NOT NULL DEFAULT true,
ADD COLUMN IF NOT EXISTS quality_alert_enabled BOOLEAN NOT NULL DEFAULT true;

-- Indexes
CREATE INDEX idx_notification_queue_user ON notification_queue(user_id);
CREATE INDEX idx_notification_queue_status ON notification_queue(status) WHERE status = 'pending';
CREATE INDEX idx_notification_queue_scheduled ON notification_queue(scheduled_at) WHERE status = 'pending';
CREATE INDEX idx_notification_log_user ON notification_log(user_id);
CREATE INDEX idx_notification_log_sent ON notification_log(sent_at);
CREATE INDEX idx_in_app_notifications_user ON in_app_notifications(user_id);
CREATE INDEX idx_in_app_notifications_unread ON in_app_notifications(user_id, is_read) WHERE is_read = false;

-- Function to check if notification type is enabled for user
CREATE OR REPLACE FUNCTION is_notification_enabled(
    p_user_id UUID,
    p_notification_type notification_type
)
RETURNS BOOLEAN AS $$
DECLARE
    v_enabled BOOLEAN;
BEGIN
    SELECT 
        CASE p_notification_type
            WHEN 'low_inventory' THEN low_inventory_enabled
            WHEN 'certification_expiring' THEN certification_expiring_enabled
            WHEN 'processing_milestone' THEN processing_milestone_enabled
            WHEN 'weather_alert' THEN weather_alert_enabled
            WHEN 'harvest_reminder' THEN harvest_reminder_enabled
            WHEN 'quality_alert' THEN quality_alert_enabled
            ELSE true
        END
    INTO v_enabled
    FROM notification_preferences
    WHERE user_id = p_user_id;
    
    RETURN COALESCE(v_enabled, true);
END;
$$ LANGUAGE plpgsql;

-- Function to get user's preferred notification channel
CREATE OR REPLACE FUNCTION get_notification_channel(p_user_id UUID)
RETURNS notification_channel AS $$
DECLARE
    v_line_connected BOOLEAN;
    v_line_enabled BOOLEAN;
BEGIN
    SELECT 
        np.line_enabled,
        lc.id IS NOT NULL
    INTO v_line_enabled, v_line_connected
    FROM notification_preferences np
    LEFT JOIN line_connections lc ON lc.user_id = np.user_id
    WHERE np.user_id = p_user_id;
    
    -- Use LINE if connected and enabled, otherwise in-app
    IF v_line_connected AND COALESCE(v_line_enabled, true) THEN
        RETURN 'line'::notification_channel;
    ELSE
        RETURN 'in_app'::notification_channel;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Function to queue a notification
CREATE OR REPLACE FUNCTION queue_notification(
    p_user_id UUID,
    p_business_id UUID,
    p_notification_type notification_type,
    p_title VARCHAR(255),
    p_title_th VARCHAR(255),
    p_message TEXT,
    p_message_th TEXT,
    p_entity_type VARCHAR(50) DEFAULT NULL,
    p_entity_id UUID DEFAULT NULL,
    p_priority INT DEFAULT 0
)
RETURNS UUID AS $$
DECLARE
    v_notification_id UUID;
BEGIN
    -- Check if notification type is enabled
    IF NOT is_notification_enabled(p_user_id, p_notification_type) THEN
        RETURN NULL;
    END IF;
    
    INSERT INTO notification_queue (
        user_id, business_id, notification_type,
        title, title_th, message, message_th,
        entity_type, entity_id, priority
    )
    VALUES (
        p_user_id, p_business_id, p_notification_type,
        p_title, p_title_th, p_message, p_message_th,
        p_entity_type, p_entity_id, p_priority
    )
    RETURNING id INTO v_notification_id;
    
    RETURN v_notification_id;
END;
$$ LANGUAGE plpgsql;

-- Function to mark notification as sent
CREATE OR REPLACE FUNCTION mark_notification_sent(
    p_queue_id UUID,
    p_channel notification_channel,
    p_line_message_id VARCHAR(255) DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    v_log_id UUID;
    v_notification RECORD;
BEGIN
    -- Get notification from queue
    SELECT * INTO v_notification
    FROM notification_queue
    WHERE id = p_queue_id AND status = 'pending';
    
    IF NOT FOUND THEN
        RETURN NULL;
    END IF;
    
    -- Insert into log
    INSERT INTO notification_log (
        user_id, business_id, notification_type, channel,
        title, title_th, message, message_th,
        entity_type, entity_id, line_message_id
    )
    VALUES (
        v_notification.user_id, v_notification.business_id,
        v_notification.notification_type, p_channel,
        v_notification.title, v_notification.title_th,
        v_notification.message, v_notification.message_th,
        v_notification.entity_type, v_notification.entity_id,
        p_line_message_id
    )
    RETURNING id INTO v_log_id;
    
    -- Also create in-app notification
    INSERT INTO in_app_notifications (
        user_id, business_id, notification_type,
        title, title_th, message, message_th,
        entity_type, entity_id
    )
    VALUES (
        v_notification.user_id, v_notification.business_id,
        v_notification.notification_type,
        v_notification.title, v_notification.title_th,
        v_notification.message, v_notification.message_th,
        v_notification.entity_type, v_notification.entity_id
    );
    
    -- Update queue status
    UPDATE notification_queue
    SET status = 'sent'
    WHERE id = p_queue_id;
    
    RETURN v_log_id;
END;
$$ LANGUAGE plpgsql;

-- Function to get unread notification count
CREATE OR REPLACE FUNCTION get_unread_notification_count(p_user_id UUID)
RETURNS INT AS $$
BEGIN
    RETURN (
        SELECT COUNT(*)::INT
        FROM in_app_notifications
        WHERE user_id = p_user_id
          AND is_read = false
          AND is_dismissed = false
    );
END;
$$ LANGUAGE plpgsql;
