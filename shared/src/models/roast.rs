//! Roast profile models

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A roast session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoastSession {
    pub id: Uuid,
    pub lot_id: Uuid,
    pub roast_date: NaiveDate,
    pub roaster_name: String,
    pub equipment: String,
    pub green_bean_weight_kg: Decimal,
    pub profile: RoastProfile,
    pub result: Option<RoastResult>,
    pub created_at: DateTime<Utc>,
}

/// A roast profile (can be a template or ad-hoc)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoastProfile {
    /// None if ad-hoc, Some if from template
    pub id: Option<Uuid>,
    pub name: String,
    pub target_roast_level: RoastLevel,
    pub checkpoints: Vec<RoastCheckpoint>,
}

/// A checkpoint in the roast profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoastCheckpoint {
    pub time_seconds: i32,
    pub temperature_celsius: Decimal,
    pub event: Option<RoastEvent>,
}

/// Roast events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoastEvent {
    ChargeTemp,
    TurningPoint,
    FirstCrackStart,
    FirstCrackEnd,
    SecondCrackStart,
    Drop,
}

/// Roast levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoastLevel {
    Light,
    MediumLight,
    Medium,
    MediumDark,
    Dark,
}

impl std::fmt::Display for RoastLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoastLevel::Light => write!(f, "Light"),
            RoastLevel::MediumLight => write!(f, "Medium Light"),
            RoastLevel::Medium => write!(f, "Medium"),
            RoastLevel::MediumDark => write!(f, "Medium Dark"),
            RoastLevel::Dark => write!(f, "Dark"),
        }
    }
}

/// Result of a roast session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoastResult {
    pub roasted_weight_kg: Decimal,
    pub weight_loss_percent: Decimal,
    pub total_time_seconds: i32,
    pub end_temperature_celsius: Decimal,
    pub roast_level: RoastLevel,
    /// Agtron or similar color reading
    pub color_reading: Option<Decimal>,
}

/// Calculate weight loss percentage
pub fn calculate_weight_loss(green_weight: Decimal, roasted_weight: Decimal) -> Decimal {
    if green_weight.is_zero() {
        Decimal::ZERO
    } else {
        ((green_weight - roasted_weight) / green_weight) * Decimal::from(100)
    }
}
