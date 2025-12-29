//! Processing models

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A processing record for a lot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingRecord {
    pub id: Uuid,
    pub lot_id: Uuid,
    pub method: ProcessingMethod,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub responsible_person: String,
    pub fermentation: Option<FermentationLog>,
    pub drying: Option<DryingLog>,
    pub final_moisture_percent: Option<Decimal>,
    pub green_bean_weight_kg: Option<Decimal>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Coffee processing methods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingMethod {
    Natural,
    Washed,
    /// Honey process with mucilage percentage
    Honey { mucilage_percent: i32 },
    WetHulled,
    /// Anaerobic fermentation with duration
    Anaerobic { hours: i32 },
    Custom(String),
}

impl std::fmt::Display for ProcessingMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessingMethod::Natural => write!(f, "Natural"),
            ProcessingMethod::Washed => write!(f, "Washed"),
            ProcessingMethod::Honey { mucilage_percent } => {
                write!(f, "Honey ({}% mucilage)", mucilage_percent)
            }
            ProcessingMethod::WetHulled => write!(f, "Wet Hulled"),
            ProcessingMethod::Anaerobic { hours } => write!(f, "Anaerobic ({}h)", hours),
            ProcessingMethod::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Fermentation log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FermentationLog {
    pub duration_hours: i32,
    pub temperature_readings: Vec<TemperatureReading>,
    pub ph_readings: Vec<PhReading>,
}

/// Temperature reading during fermentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureReading {
    pub timestamp: DateTime<Utc>,
    pub temperature_celsius: Decimal,
}

/// pH reading during fermentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhReading {
    pub timestamp: DateTime<Utc>,
    pub ph_value: Decimal,
}

/// Drying log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DryingLog {
    pub method: DryingMethod,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub target_moisture_percent: Decimal,
    pub moisture_readings: Vec<MoistureReading>,
}

/// Drying methods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DryingMethod {
    RaisedBed,
    Patio,
    Mechanical,
    Greenhouse,
    Custom(String),
}

/// Moisture reading during drying
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoistureReading {
    pub timestamp: DateTime<Utc>,
    pub moisture_percent: Decimal,
}

/// Calculate processing yield
pub fn calculate_processing_yield(cherry_weight: Decimal, green_bean_weight: Decimal) -> Decimal {
    if cherry_weight.is_zero() {
        Decimal::ZERO
    } else {
        (green_bean_weight / cherry_weight) * Decimal::from(100)
    }
}
