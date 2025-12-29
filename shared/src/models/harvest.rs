//! Harvest models

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::WeatherSnapshot;

/// A harvest record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Harvest {
    pub id: Uuid,
    pub lot_id: Uuid,
    pub plot_id: Uuid,
    pub harvest_date: NaiveDate,
    pub picker_name: Option<String>,
    pub cherry_weight_kg: Decimal,
    pub ripeness: RipenessAssessment,
    pub weather_snapshot: Option<WeatherSnapshot>,
    pub created_at: DateTime<Utc>,
}

/// Assessment of cherry ripeness in a harvest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipenessAssessment {
    /// Percentage of underripe cherries (0-100)
    pub underripe_percent: i32,
    /// Percentage of ripe cherries (0-100)
    pub ripe_percent: i32,
    /// Percentage of overripe cherries (0-100)
    pub overripe_percent: i32,
}

impl RipenessAssessment {
    /// Create a new ripeness assessment
    /// Returns None if percentages don't sum to 100
    pub fn new(underripe: i32, ripe: i32, overripe: i32) -> Option<Self> {
        if underripe + ripe + overripe == 100
            && underripe >= 0
            && ripe >= 0
            && overripe >= 0
        {
            Some(Self {
                underripe_percent: underripe,
                ripe_percent: ripe,
                overripe_percent: overripe,
            })
        } else {
            None
        }
    }

    /// Check if the assessment is valid (sums to 100)
    pub fn is_valid(&self) -> bool {
        self.underripe_percent + self.ripe_percent + self.overripe_percent == 100
            && self.underripe_percent >= 0
            && self.ripe_percent >= 0
            && self.overripe_percent >= 0
    }
}
