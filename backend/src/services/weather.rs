//! Weather service for storing and retrieving weather data

use chrono::{DateTime, Duration, NaiveDate, Timelike, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::str::FromStr;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::external::weather::{CurrentWeather, WeatherClient, WeatherForecast};

/// Weather service for managing weather data
#[derive(Clone)]
pub struct WeatherService {
    db: PgPool,
    weather_client: Option<WeatherClient>,
}

/// Weather snapshot record
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct WeatherSnapshot {
    pub id: Uuid,
    pub business_id: Uuid,
    pub latitude: Decimal,
    pub longitude: Decimal,
    pub location_name: Option<String>,
    pub recorded_at: DateTime<Utc>,
    pub temperature_celsius: Decimal,
    pub feels_like_celsius: Option<Decimal>,
    pub humidity_percent: Option<i32>,
    pub pressure_hpa: Option<i32>,
    pub wind_speed_mps: Option<Decimal>,
    pub wind_direction_deg: Option<i32>,
    pub cloud_coverage_percent: Option<i32>,
    pub visibility_meters: Option<i32>,
    pub weather_condition: Option<String>,
    pub weather_description: Option<String>,
    pub weather_icon: Option<String>,
    pub rain_1h_mm: Option<Decimal>,
    pub rain_3h_mm: Option<Decimal>,
    pub sunrise: Option<DateTime<Utc>>,
    pub sunset: Option<DateTime<Utc>>,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

/// Input for storing weather snapshot
#[derive(Debug, Deserialize)]
pub struct StoreWeatherInput {
    pub latitude: Decimal,
    pub longitude: Decimal,
    pub location_name: Option<String>,
    pub recorded_at: Option<DateTime<Utc>>,
    pub temperature_celsius: Decimal,
    pub feels_like_celsius: Option<Decimal>,
    pub humidity_percent: Option<i32>,
    pub pressure_hpa: Option<i32>,
    pub wind_speed_mps: Option<Decimal>,
    pub wind_direction_deg: Option<i32>,
    pub cloud_coverage_percent: Option<i32>,
    pub visibility_meters: Option<i32>,
    pub weather_condition: Option<String>,
    pub weather_description: Option<String>,
    pub weather_icon: Option<String>,
    pub rain_1h_mm: Option<Decimal>,
    pub rain_3h_mm: Option<Decimal>,
    pub sunrise: Option<DateTime<Utc>>,
    pub sunset: Option<DateTime<Utc>>,
    pub source: Option<String>,
}

/// Weather alert configuration
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct WeatherAlert {
    pub id: Uuid,
    pub business_id: Uuid,
    pub plot_id: Option<Uuid>,
    pub alert_type: String,
    pub threshold_value: Option<Decimal>,
    pub threshold_unit: Option<String>,
    pub is_active: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub notify_email: bool,
    pub notify_line: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating weather alert
#[derive(Debug, Deserialize)]
pub struct CreateWeatherAlertInput {
    pub plot_id: Option<Uuid>,
    pub alert_type: String,
    pub threshold_value: Option<Decimal>,
    pub threshold_unit: Option<String>,
    pub notify_email: Option<bool>,
    pub notify_line: Option<bool>,
}

/// Cached weather forecast
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CachedForecast {
    pub id: Uuid,
    pub business_id: Uuid,
    pub latitude: Decimal,
    pub longitude: Decimal,
    pub location_name: Option<String>,
    pub timezone_offset_seconds: Option<i32>,
    pub forecasts: serde_json::Value,
    pub fetched_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl WeatherService {
    /// Create a new WeatherService instance
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            weather_client: None,
        }
    }

    /// Create a new WeatherService with weather API client
    pub fn with_client(db: PgPool, api_key: String) -> Self {
        Self {
            db,
            weather_client: Some(WeatherClient::new(api_key)),
        }
    }

    /// Store a weather snapshot
    pub async fn store_snapshot(
        &self,
        business_id: Uuid,
        input: StoreWeatherInput,
    ) -> AppResult<WeatherSnapshot> {
        let recorded_at = input.recorded_at.unwrap_or_else(Utc::now);
        let source = input.source.unwrap_or_else(|| "manual".to_string());

        let snapshot = sqlx::query_as::<_, WeatherSnapshot>(
            r#"
            INSERT INTO weather_snapshots (
                business_id, latitude, longitude, location_name, recorded_at,
                temperature_celsius, feels_like_celsius, humidity_percent, pressure_hpa,
                wind_speed_mps, wind_direction_deg, cloud_coverage_percent, visibility_meters,
                weather_condition, weather_description, weather_icon,
                rain_1h_mm, rain_3h_mm, sunrise, sunset, source
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            RETURNING id, business_id, latitude, longitude, location_name, recorded_at,
                      temperature_celsius, feels_like_celsius, humidity_percent, pressure_hpa,
                      wind_speed_mps, wind_direction_deg, cloud_coverage_percent, visibility_meters,
                      weather_condition, weather_description, weather_icon,
                      rain_1h_mm, rain_3h_mm, sunrise, sunset, source, created_at
            "#,
        )
        .bind(business_id)
        .bind(input.latitude)
        .bind(input.longitude)
        .bind(&input.location_name)
        .bind(recorded_at)
        .bind(input.temperature_celsius)
        .bind(input.feels_like_celsius)
        .bind(input.humidity_percent)
        .bind(input.pressure_hpa)
        .bind(input.wind_speed_mps)
        .bind(input.wind_direction_deg)
        .bind(input.cloud_coverage_percent)
        .bind(input.visibility_meters)
        .bind(&input.weather_condition)
        .bind(&input.weather_description)
        .bind(&input.weather_icon)
        .bind(input.rain_1h_mm)
        .bind(input.rain_3h_mm)
        .bind(input.sunrise)
        .bind(input.sunset)
        .bind(&source)
        .fetch_one(&self.db)
        .await?;

        Ok(snapshot)
    }

    /// Store weather from API response
    pub async fn store_from_api(
        &self,
        business_id: Uuid,
        weather: &CurrentWeather,
        latitude: Decimal,
        longitude: Decimal,
    ) -> AppResult<WeatherSnapshot> {
        let input = StoreWeatherInput {
            latitude,
            longitude,
            location_name: None,
            recorded_at: Some(weather.timestamp),
            temperature_celsius: weather.temperature_celsius,
            feels_like_celsius: Some(weather.feels_like_celsius),
            humidity_percent: Some(weather.humidity_percent),
            pressure_hpa: Some(weather.pressure_hpa),
            wind_speed_mps: Some(weather.wind_speed_mps),
            wind_direction_deg: Some(weather.wind_direction_deg),
            cloud_coverage_percent: Some(weather.cloud_coverage_percent),
            visibility_meters: Some(weather.visibility_meters),
            weather_condition: Some(weather.weather_condition.clone()),
            weather_description: Some(weather.weather_description.clone()),
            weather_icon: Some(weather.weather_icon.clone()),
            rain_1h_mm: weather.rain_1h_mm,
            rain_3h_mm: weather.rain_3h_mm,
            sunrise: Some(weather.sunrise),
            sunset: Some(weather.sunset),
            source: Some("openweathermap".to_string()),
        };

        self.store_snapshot(business_id, input).await
    }

    /// Get weather snapshot by ID
    pub async fn get_snapshot(
        &self,
        business_id: Uuid,
        snapshot_id: Uuid,
    ) -> AppResult<WeatherSnapshot> {
        let snapshot = sqlx::query_as::<_, WeatherSnapshot>(
            r#"
            SELECT id, business_id, latitude, longitude, location_name, recorded_at,
                   temperature_celsius, feels_like_celsius, humidity_percent, pressure_hpa,
                   wind_speed_mps, wind_direction_deg, cloud_coverage_percent, visibility_meters,
                   weather_condition, weather_description, weather_icon,
                   rain_1h_mm, rain_3h_mm, sunrise, sunset, source, created_at
            FROM weather_snapshots
            WHERE id = $1 AND business_id = $2
            "#,
        )
        .bind(snapshot_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Weather snapshot".to_string()))?;

        Ok(snapshot)
    }

    /// Get weather snapshots for a date range
    pub async fn get_snapshots_for_range(
        &self,
        business_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> AppResult<Vec<WeatherSnapshot>> {
        let snapshots = sqlx::query_as::<_, WeatherSnapshot>(
            r#"
            SELECT id, business_id, latitude, longitude, location_name, recorded_at,
                   temperature_celsius, feels_like_celsius, humidity_percent, pressure_hpa,
                   wind_speed_mps, wind_direction_deg, cloud_coverage_percent, visibility_meters,
                   weather_condition, weather_description, weather_icon,
                   rain_1h_mm, rain_3h_mm, sunrise, sunset, source, created_at
            FROM weather_snapshots
            WHERE business_id = $1
              AND recorded_at >= $2::date
              AND recorded_at < ($3::date + INTERVAL '1 day')
            ORDER BY recorded_at DESC
            "#,
        )
        .bind(business_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.db)
        .await?;

        Ok(snapshots)
    }

    /// Get weather snapshots near a location
    pub async fn get_snapshots_near_location(
        &self,
        business_id: Uuid,
        latitude: Decimal,
        longitude: Decimal,
        max_distance_km: Decimal,
        max_age_hours: i32,
    ) -> AppResult<Vec<WeatherSnapshot>> {
        let cutoff = Utc::now() - Duration::hours(max_age_hours as i64);

        let snapshots = sqlx::query_as::<_, WeatherSnapshot>(
            r#"
            SELECT id, business_id, latitude, longitude, location_name, recorded_at,
                   temperature_celsius, feels_like_celsius, humidity_percent, pressure_hpa,
                   wind_speed_mps, wind_direction_deg, cloud_coverage_percent, visibility_meters,
                   weather_condition, weather_description, weather_icon,
                   rain_1h_mm, rain_3h_mm, sunrise, sunset, source, created_at
            FROM weather_snapshots
            WHERE business_id = $1
              AND recorded_at > $2
              AND SQRT(
                  POWER((latitude - $3) * 111, 2) +
                  POWER((longitude - $4) * 102, 2)
              ) <= $5
            ORDER BY recorded_at DESC
            "#,
        )
        .bind(business_id)
        .bind(cutoff)
        .bind(latitude)
        .bind(longitude)
        .bind(max_distance_km)
        .fetch_all(&self.db)
        .await?;

        Ok(snapshots)
    }
}


impl WeatherService {
    /// Link weather snapshot to harvest
    pub async fn link_to_harvest(
        &self,
        business_id: Uuid,
        harvest_id: Uuid,
        snapshot_id: Uuid,
    ) -> AppResult<()> {
        // Validate harvest belongs to business
        let harvest_exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM harvests h
                JOIN lots l ON l.id = h.lot_id
                WHERE h.id = $1 AND l.business_id = $2
            )
            "#,
        )
        .bind(harvest_id)
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        if !harvest_exists {
            return Err(AppError::NotFound("Harvest".to_string()));
        }

        // Validate snapshot belongs to business
        let snapshot_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM weather_snapshots WHERE id = $1 AND business_id = $2)",
        )
        .bind(snapshot_id)
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        if !snapshot_exists {
            return Err(AppError::NotFound("Weather snapshot".to_string()));
        }

        sqlx::query("UPDATE harvests SET weather_snapshot_id = $1 WHERE id = $2")
            .bind(snapshot_id)
            .bind(harvest_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Get weather for a harvest
    pub async fn get_harvest_weather(
        &self,
        business_id: Uuid,
        harvest_id: Uuid,
    ) -> AppResult<Option<WeatherSnapshot>> {
        let snapshot = sqlx::query_as::<_, WeatherSnapshot>(
            r#"
            SELECT ws.id, ws.business_id, ws.latitude, ws.longitude, ws.location_name, ws.recorded_at,
                   ws.temperature_celsius, ws.feels_like_celsius, ws.humidity_percent, ws.pressure_hpa,
                   ws.wind_speed_mps, ws.wind_direction_deg, ws.cloud_coverage_percent, ws.visibility_meters,
                   ws.weather_condition, ws.weather_description, ws.weather_icon,
                   ws.rain_1h_mm, ws.rain_3h_mm, ws.sunrise, ws.sunset, ws.source, ws.created_at
            FROM weather_snapshots ws
            JOIN harvests h ON h.weather_snapshot_id = ws.id
            JOIN lots l ON l.id = h.lot_id
            WHERE h.id = $1 AND l.business_id = $2
            "#,
        )
        .bind(harvest_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(snapshot)
    }

    /// Fetch and store current weather from API
    pub async fn fetch_and_store_current(
        &self,
        business_id: Uuid,
        latitude: Decimal,
        longitude: Decimal,
    ) -> AppResult<WeatherSnapshot> {
        let client = self
            .weather_client
            .as_ref()
            .ok_or_else(|| AppError::Internal("Weather API client not configured".to_string()))?;

        let weather = client.get_current_weather(latitude, longitude).await?;
        self.store_from_api(business_id, &weather, latitude, longitude)
            .await
    }

    /// Cache forecast data
    pub async fn cache_forecast(
        &self,
        business_id: Uuid,
        forecast: &WeatherForecast,
    ) -> AppResult<CachedForecast> {
        let forecasts_json = serde_json::to_value(&forecast.forecasts)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Forecasts expire after 3 hours
        let expires_at = Utc::now() + Duration::hours(3);

        let cached = sqlx::query_as::<_, CachedForecast>(
            r#"
            INSERT INTO weather_forecasts (
                business_id, latitude, longitude, location_name, timezone_offset_seconds,
                forecasts, expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, business_id, latitude, longitude, location_name, timezone_offset_seconds,
                      forecasts, fetched_at, expires_at, created_at
            "#,
        )
        .bind(business_id)
        .bind(forecast.latitude)
        .bind(forecast.longitude)
        .bind(&forecast.location_name)
        .bind(forecast.timezone_offset_seconds)
        .bind(&forecasts_json)
        .bind(expires_at)
        .fetch_one(&self.db)
        .await?;

        Ok(cached)
    }

    /// Get cached forecast if not expired
    pub async fn get_cached_forecast(
        &self,
        business_id: Uuid,
        latitude: Decimal,
        longitude: Decimal,
    ) -> AppResult<Option<CachedForecast>> {
        let cached = sqlx::query_as::<_, CachedForecast>(
            r#"
            SELECT id, business_id, latitude, longitude, location_name, timezone_offset_seconds,
                   forecasts, fetched_at, expires_at, created_at
            FROM weather_forecasts
            WHERE business_id = $1
              AND ABS(latitude - $2) < 0.01
              AND ABS(longitude - $3) < 0.01
              AND expires_at > NOW()
            ORDER BY fetched_at DESC
            LIMIT 1
            "#,
        )
        .bind(business_id)
        .bind(latitude)
        .bind(longitude)
        .fetch_optional(&self.db)
        .await?;

        Ok(cached)
    }

    /// Fetch forecast (from cache or API)
    pub async fn get_forecast(
        &self,
        business_id: Uuid,
        latitude: Decimal,
        longitude: Decimal,
    ) -> AppResult<WeatherForecast> {
        // Check cache first
        if let Some(cached) = self.get_cached_forecast(business_id, latitude, longitude).await? {
            let forecasts = serde_json::from_value(cached.forecasts)
                .map_err(|e| AppError::Internal(e.to_string()))?;

            return Ok(WeatherForecast {
                location_name: cached.location_name.unwrap_or_default(),
                latitude: cached.latitude,
                longitude: cached.longitude,
                timezone_offset_seconds: cached.timezone_offset_seconds.unwrap_or(0),
                forecasts,
            });
        }

        // Fetch from API
        let client = self
            .weather_client
            .as_ref()
            .ok_or_else(|| AppError::Internal("Weather API client not configured".to_string()))?;

        let forecast = client.get_forecast(latitude, longitude).await?;

        // Cache the result
        let _ = self.cache_forecast(business_id, &forecast).await;

        Ok(forecast)
    }

    // ========================================================================
    // Weather Alerts
    // ========================================================================

    /// Create a weather alert
    pub async fn create_alert(
        &self,
        business_id: Uuid,
        input: CreateWeatherAlertInput,
    ) -> AppResult<WeatherAlert> {
        // Validate plot if provided
        if let Some(plot_id) = input.plot_id {
            let plot_exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM plots WHERE id = $1 AND business_id = $2)",
            )
            .bind(plot_id)
            .bind(business_id)
            .fetch_one(&self.db)
            .await?;

            if !plot_exists {
                return Err(AppError::NotFound("Plot".to_string()));
            }
        }

        // Validate alert type
        let valid_types = ["rain_forecast", "frost_warning", "heat_warning", "wind_warning"];
        if !valid_types.contains(&input.alert_type.as_str()) {
            return Err(AppError::Validation {
                field: "alert_type".to_string(),
                message: format!("Invalid alert type. Must be one of: {:?}", valid_types),
                message_th: format!("ประเภทการแจ้งเตือนไม่ถูกต้อง ต้องเป็นหนึ่งใน: {:?}", valid_types),
            });
        }

        let notify_email = input.notify_email.unwrap_or(true);
        let notify_line = input.notify_line.unwrap_or(true);

        let alert = sqlx::query_as::<_, WeatherAlert>(
            r#"
            INSERT INTO weather_alerts (
                business_id, plot_id, alert_type, threshold_value, threshold_unit,
                notify_email, notify_line
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, business_id, plot_id, alert_type, threshold_value, threshold_unit,
                      is_active, last_triggered_at, notify_email, notify_line, created_at, updated_at
            "#,
        )
        .bind(business_id)
        .bind(input.plot_id)
        .bind(&input.alert_type)
        .bind(input.threshold_value)
        .bind(&input.threshold_unit)
        .bind(notify_email)
        .bind(notify_line)
        .fetch_one(&self.db)
        .await?;

        Ok(alert)
    }

    /// List weather alerts for a business
    pub async fn list_alerts(&self, business_id: Uuid) -> AppResult<Vec<WeatherAlert>> {
        let alerts = sqlx::query_as::<_, WeatherAlert>(
            r#"
            SELECT id, business_id, plot_id, alert_type, threshold_value, threshold_unit,
                   is_active, last_triggered_at, notify_email, notify_line, created_at, updated_at
            FROM weather_alerts
            WHERE business_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(alerts)
    }

    /// Delete a weather alert
    pub async fn delete_alert(&self, business_id: Uuid, alert_id: Uuid) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM weather_alerts WHERE id = $1 AND business_id = $2")
            .bind(alert_id)
            .bind(business_id)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Weather alert".to_string()));
        }

        Ok(())
    }

    /// Check for rain alerts based on forecast
    pub async fn check_rain_alerts(
        &self,
        business_id: Uuid,
        forecast: &WeatherForecast,
    ) -> AppResult<Vec<(WeatherAlert, String)>> {
        let alerts = sqlx::query_as::<_, WeatherAlert>(
            r#"
            SELECT id, business_id, plot_id, alert_type, threshold_value, threshold_unit,
                   is_active, last_triggered_at, notify_email, notify_line, created_at, updated_at
            FROM weather_alerts
            WHERE business_id = $1 AND alert_type = 'rain_forecast' AND is_active = true
            "#,
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        let mut triggered = Vec::new();

        for alert in alerts {
            let threshold = alert.threshold_value.unwrap_or(Decimal::from(5)); // Default 5mm

            for item in &forecast.forecasts {
                if let Some(rain) = item.rain_3h_mm {
                    if rain >= threshold {
                        let message = format!(
                            "Rain forecast: {}mm expected at {}",
                            rain,
                            item.timestamp.format("%Y-%m-%d %H:%M")
                        );
                        triggered.push((alert.clone(), message));
                        break;
                    }
                }
            }
        }

        Ok(triggered)
    }

    // ========================================================================
    // Harvest Window Recommendations
    // ========================================================================

    /// Get harvest window recommendations based on weather forecast
    pub fn get_harvest_window_recommendations(
        &self,
        forecast: &WeatherForecast,
        ripeness_percent: Option<i32>,
    ) -> Vec<HarvestWindowRecommendation> {
        let mut recommendations = Vec::new();
        let ripeness = ripeness_percent.unwrap_or(80); // Default 80% ripe

        // Group forecasts by day
        let mut daily_forecasts: std::collections::HashMap<chrono::NaiveDate, Vec<&crate::external::weather::ForecastItem>> = 
            std::collections::HashMap::new();

        for item in &forecast.forecasts {
            let date = item.timestamp.date_naive();
            daily_forecasts.entry(date).or_default().push(item);
        }

        // Analyze each day
        let mut sorted_dates: Vec<_> = daily_forecasts.keys().collect();
        sorted_dates.sort();

        for date in sorted_dates {
            if let Some(items) = daily_forecasts.get(date) {
                let analysis = self.analyze_day_for_harvest(items, ripeness);
                recommendations.push(HarvestWindowRecommendation {
                    date: *date,
                    suitability: analysis.suitability,
                    score: analysis.score,
                    reasons: analysis.reasons,
                    reasons_th: analysis.reasons_th,
                    best_hours: analysis.best_hours,
                    warnings: analysis.warnings,
                    warnings_th: analysis.warnings_th,
                });
            }
        }

        recommendations
    }

    /// Analyze a day's forecast for harvest suitability
    fn analyze_day_for_harvest(
        &self,
        items: &[&crate::external::weather::ForecastItem],
        ripeness_percent: i32,
    ) -> DayAnalysis {
        let mut score = 100i32;
        let mut reasons = Vec::new();
        let mut reasons_th = Vec::new();
        let mut warnings = Vec::new();
        let mut warnings_th = Vec::new();
        let mut best_hours = Vec::new();

        // Check for rain
        let total_rain: Decimal = items
            .iter()
            .filter_map(|i| i.rain_3h_mm)
            .sum();
        
        let max_pop: Decimal = items
            .iter()
            .map(|i| i.pop)
            .max()
            .unwrap_or(Decimal::ZERO);

        if total_rain > Decimal::from(5) {
            score -= 40;
            warnings.push(format!("Heavy rain expected: {}mm", total_rain));
            warnings_th.push(format!("คาดว่าจะมีฝนตกหนัก: {}มม.", total_rain));
        } else if total_rain > Decimal::ZERO {
            score -= 20;
            warnings.push(format!("Light rain expected: {}mm", total_rain));
            warnings_th.push(format!("คาดว่าจะมีฝนตกเล็กน้อย: {}มม.", total_rain));
        }

        if max_pop > Decimal::from_str("0.7").unwrap_or(Decimal::ZERO) {
            score -= 15;
            warnings.push("High probability of precipitation".to_string());
            warnings_th.push("มีโอกาสฝนตกสูง".to_string());
        }

        // Check temperature
        let avg_temp: Decimal = items
            .iter()
            .map(|i| i.temperature_celsius)
            .sum::<Decimal>() / Decimal::from(items.len().max(1));

        if avg_temp > Decimal::from(32) {
            score -= 15;
            warnings.push("High temperature may affect cherry quality".to_string());
            warnings_th.push("อุณหภูมิสูงอาจส่งผลต่อคุณภาพเชอร์รี่".to_string());
        } else if avg_temp >= Decimal::from(20) && avg_temp <= Decimal::from(28) {
            score += 10;
            reasons.push("Ideal temperature for harvesting".to_string());
            reasons_th.push("อุณหภูมิเหมาะสมสำหรับการเก็บเกี่ยว".to_string());
        }

        // Check humidity
        let avg_humidity: i32 = items.iter().map(|i| i.humidity_percent).sum::<i32>() 
            / items.len().max(1) as i32;

        if avg_humidity > 85 {
            score -= 10;
            warnings.push("High humidity may cause mold issues".to_string());
            warnings_th.push("ความชื้นสูงอาจทำให้เกิดเชื้อรา".to_string());
        } else if avg_humidity >= 50 && avg_humidity <= 75 {
            score += 5;
            reasons.push("Good humidity levels".to_string());
            reasons_th.push("ระดับความชื้นดี".to_string());
        }

        // Check wind
        let max_wind: Decimal = items
            .iter()
            .map(|i| i.wind_speed_mps)
            .max()
            .unwrap_or(Decimal::ZERO);

        if max_wind > Decimal::from(10) {
            score -= 10;
            warnings.push("Strong winds may make harvesting difficult".to_string());
            warnings_th.push("ลมแรงอาจทำให้การเก็บเกี่ยวยากลำบาก".to_string());
        }

        // Consider ripeness
        if ripeness_percent >= 85 {
            score += 15;
            reasons.push("High ripeness - optimal harvest time".to_string());
            reasons_th.push("ความสุกสูง - เวลาเก็บเกี่ยวที่เหมาะสม".to_string());
        } else if ripeness_percent >= 70 {
            score += 5;
            reasons.push("Good ripeness level".to_string());
            reasons_th.push("ระดับความสุกดี".to_string());
        } else if ripeness_percent < 60 {
            score -= 20;
            warnings.push("Low ripeness - consider waiting".to_string());
            warnings_th.push("ความสุกต่ำ - ควรรอเพิ่มเติม".to_string());
        }

        // Find best hours (morning hours with no rain)
        for item in items {
            let hour = item.timestamp.hour();
            if hour >= 6 && hour <= 11 {
                let has_rain = item.rain_3h_mm.map(|r| r > Decimal::ZERO).unwrap_or(false);
                let low_pop = item.pop < Decimal::from_str("0.3").unwrap_or(Decimal::ZERO);
                if !has_rain && low_pop {
                    best_hours.push(format!("{:02}:00", hour));
                }
            }
        }

        if best_hours.is_empty() {
            // Fallback to afternoon if morning not suitable
            for item in items {
                let hour = item.timestamp.hour();
                if hour >= 14 && hour <= 17 {
                    let has_rain = item.rain_3h_mm.map(|r| r > Decimal::ZERO).unwrap_or(false);
                    if !has_rain {
                        best_hours.push(format!("{:02}:00", hour));
                    }
                }
            }
        }

        // Determine suitability
        let suitability = if score >= 80 {
            HarvestSuitability::Excellent
        } else if score >= 60 {
            HarvestSuitability::Good
        } else if score >= 40 {
            HarvestSuitability::Fair
        } else {
            HarvestSuitability::Poor
        };

        DayAnalysis {
            suitability,
            score: score.max(0).min(100),
            reasons,
            reasons_th,
            best_hours,
            warnings,
            warnings_th,
        }
    }
}

/// Harvest window recommendation
#[derive(Debug, Clone, Serialize)]
pub struct HarvestWindowRecommendation {
    pub date: chrono::NaiveDate,
    pub suitability: HarvestSuitability,
    pub score: i32,
    pub reasons: Vec<String>,
    pub reasons_th: Vec<String>,
    pub best_hours: Vec<String>,
    pub warnings: Vec<String>,
    pub warnings_th: Vec<String>,
}

/// Harvest suitability level
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum HarvestSuitability {
    Excellent,
    Good,
    Fair,
    Poor,
}

/// Internal day analysis result
struct DayAnalysis {
    suitability: HarvestSuitability,
    score: i32,
    reasons: Vec<String>,
    reasons_th: Vec<String>,
    best_hours: Vec<String>,
    warnings: Vec<String>,
    warnings_th: Vec<String>,
}
