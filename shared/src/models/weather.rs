//! Weather data models

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::types::GpsCoordinates;

/// A weather snapshot at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherSnapshot {
    pub timestamp: DateTime<Utc>,
    pub location: GpsCoordinates,
    pub temperature_celsius: Decimal,
    pub humidity_percent: i32,
    pub precipitation_mm: Decimal,
    pub conditions: String,
}

/// Weather forecast for a location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherForecast {
    pub location: GpsCoordinates,
    pub forecasts: Vec<DailyForecast>,
}

/// Daily weather forecast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyForecast {
    pub date: NaiveDate,
    pub high_celsius: Decimal,
    pub low_celsius: Decimal,
    pub precipitation_probability: i32,
    pub precipitation_mm: Decimal,
    pub humidity_percent: i32,
    pub conditions: String,
}

/// Weather alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherAlert {
    pub location: GpsCoordinates,
    pub alert_type: WeatherAlertType,
    pub message: String,
    pub message_th: String,
    pub forecast_date: NaiveDate,
}

/// Types of weather alerts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WeatherAlertType {
    RainDuringHarvest,
    HighHumidity,
    ExtremeTemperature,
}

/// Recommended harvest window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarvestWindow {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub confidence: f32,
    pub reason: String,
    pub reason_th: String,
}
