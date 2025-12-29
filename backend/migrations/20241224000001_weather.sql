-- Weather Integration Migration
-- Task 16: Weather data storage and harvest associations

-- Weather snapshots table
CREATE TABLE IF NOT EXISTS weather_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    -- Location
    latitude DECIMAL(10, 7) NOT NULL,
    longitude DECIMAL(10, 7) NOT NULL,
    location_name VARCHAR(255),
    -- Timestamp
    recorded_at TIMESTAMPTZ NOT NULL,
    -- Current conditions
    temperature_celsius DECIMAL(5, 2) NOT NULL,
    feels_like_celsius DECIMAL(5, 2),
    humidity_percent INTEGER,
    pressure_hpa INTEGER,
    wind_speed_mps DECIMAL(5, 2),
    wind_direction_deg INTEGER,
    cloud_coverage_percent INTEGER,
    visibility_meters INTEGER,
    -- Weather description
    weather_condition VARCHAR(50),
    weather_description VARCHAR(255),
    weather_icon VARCHAR(10),
    -- Precipitation
    rain_1h_mm DECIMAL(6, 2),
    rain_3h_mm DECIMAL(6, 2),
    -- Sun times
    sunrise TIMESTAMPTZ,
    sunset TIMESTAMPTZ,
    -- Source
    source VARCHAR(50) NOT NULL DEFAULT 'openweathermap',
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Weather alerts table
CREATE TABLE IF NOT EXISTS weather_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    plot_id UUID REFERENCES plots(id) ON DELETE CASCADE,
    -- Alert configuration
    alert_type VARCHAR(50) NOT NULL, -- rain_forecast, frost_warning, heat_warning, wind_warning
    threshold_value DECIMAL(10, 2),
    threshold_unit VARCHAR(20), -- mm, celsius, mps
    -- Alert status
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_triggered_at TIMESTAMPTZ,
    -- Notification preferences
    notify_email BOOLEAN NOT NULL DEFAULT true,
    notify_line BOOLEAN NOT NULL DEFAULT true,
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Link harvests to weather snapshots
ALTER TABLE harvests
ADD COLUMN IF NOT EXISTS weather_snapshot_id UUID REFERENCES weather_snapshots(id);

-- Weather forecast cache table (for storing forecast data)
CREATE TABLE IF NOT EXISTS weather_forecasts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    -- Location
    latitude DECIMAL(10, 7) NOT NULL,
    longitude DECIMAL(10, 7) NOT NULL,
    location_name VARCHAR(255),
    timezone_offset_seconds INTEGER,
    -- Forecast data as JSONB array
    forecasts JSONB NOT NULL DEFAULT '[]'::jsonb,
    -- Cache metadata
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_weather_snapshots_business 
    ON weather_snapshots(business_id);
CREATE INDEX IF NOT EXISTS idx_weather_snapshots_location 
    ON weather_snapshots(latitude, longitude);
CREATE INDEX IF NOT EXISTS idx_weather_snapshots_recorded 
    ON weather_snapshots(business_id, recorded_at DESC);

-- Spatial index for location queries (using btree on lat/lon for simplicity)
CREATE INDEX IF NOT EXISTS idx_weather_snapshots_coords 
    ON weather_snapshots(business_id, latitude, longitude);

CREATE INDEX IF NOT EXISTS idx_weather_alerts_business 
    ON weather_alerts(business_id);
CREATE INDEX IF NOT EXISTS idx_weather_alerts_plot 
    ON weather_alerts(plot_id) WHERE plot_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_weather_alerts_active 
    ON weather_alerts(business_id, is_active) WHERE is_active = true;

CREATE INDEX IF NOT EXISTS idx_harvests_weather 
    ON harvests(weather_snapshot_id) WHERE weather_snapshot_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_weather_forecasts_business 
    ON weather_forecasts(business_id);
CREATE INDEX IF NOT EXISTS idx_weather_forecasts_location 
    ON weather_forecasts(latitude, longitude);
CREATE INDEX IF NOT EXISTS idx_weather_forecasts_expires 
    ON weather_forecasts(expires_at);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_weather_alerts_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_weather_alerts_updated_at
    BEFORE UPDATE ON weather_alerts
    FOR EACH ROW
    EXECUTE FUNCTION update_weather_alerts_updated_at();

-- Function to find nearest weather snapshot for a location
CREATE OR REPLACE FUNCTION find_nearest_weather_snapshot(
    p_business_id UUID,
    p_latitude DECIMAL,
    p_longitude DECIMAL,
    p_max_distance_km DECIMAL DEFAULT 50,
    p_max_age_hours INTEGER DEFAULT 24
) RETURNS UUID AS $$
DECLARE
    v_snapshot_id UUID;
BEGIN
    -- Simple distance calculation using Haversine approximation
    -- For Thailand, 1 degree latitude ≈ 111 km, 1 degree longitude ≈ 102 km
    SELECT id INTO v_snapshot_id
    FROM weather_snapshots
    WHERE business_id = p_business_id
      AND recorded_at > NOW() - (p_max_age_hours || ' hours')::INTERVAL
      AND SQRT(
          POWER((latitude - p_latitude) * 111, 2) +
          POWER((longitude - p_longitude) * 102, 2)
      ) <= p_max_distance_km
    ORDER BY recorded_at DESC
    LIMIT 1;
    
    RETURN v_snapshot_id;
END;
$$ LANGUAGE plpgsql;
