//! Farm plot models

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::GpsCoordinates;

/// A coffee plot within a farm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plot {
    pub id: Uuid,
    pub business_id: Uuid,
    pub name: String,
    pub coordinates: Option<GpsCoordinates>,
    /// Area in rai (Thai unit: 1 rai = 1,600 m²)
    pub area_rai: Decimal,
    pub altitude_meters: Option<i32>,
    pub shade_coverage_percent: Option<i32>,
    pub varieties: Vec<PlotVariety>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A coffee variety planted in a plot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotVariety {
    pub variety: CoffeeVariety,
    pub planting_date: Option<NaiveDate>,
    pub tree_count: Option<i32>,
}

/// Coffee varieties commonly grown in Thailand
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoffeeVariety {
    Typica,
    Catimor,
    Catuai,
    Geisha,
    Bourbon,
    SL28,
    SL34,
    Caturra,
    /// Custom variety with name
    Custom(String),
}

impl std::fmt::Display for CoffeeVariety {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoffeeVariety::Typica => write!(f, "Typica"),
            CoffeeVariety::Catimor => write!(f, "Catimor"),
            CoffeeVariety::Catuai => write!(f, "Catuai"),
            CoffeeVariety::Geisha => write!(f, "Geisha"),
            CoffeeVariety::Bourbon => write!(f, "Bourbon"),
            CoffeeVariety::SL28 => write!(f, "SL28"),
            CoffeeVariety::SL34 => write!(f, "SL34"),
            CoffeeVariety::Caturra => write!(f, "Caturra"),
            CoffeeVariety::Custom(name) => write!(f, "{}", name),
        }
    }
}
