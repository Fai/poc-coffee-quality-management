//! Lot and traceability models

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A coffee lot tracked through the supply chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lot {
    pub id: Uuid,
    pub business_id: Uuid,
    /// Unique traceability code (e.g., "CQM-2024-DOI-0001")
    pub traceability_code: String,
    pub name: String,
    pub stage: LotStage,
    /// Source lots for blended lots
    pub source_lots: Vec<LotSource>,
    pub current_weight_kg: Decimal,
    pub qr_code_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Stage of a lot in the supply chain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LotStage {
    Cherry,
    Parchment,
    GreenBean,
    RoastedBean,
    Sold,
}

impl std::fmt::Display for LotStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LotStage::Cherry => write!(f, "Cherry"),
            LotStage::Parchment => write!(f, "Parchment"),
            LotStage::GreenBean => write!(f, "Green Bean"),
            LotStage::RoastedBean => write!(f, "Roasted Bean"),
            LotStage::Sold => write!(f, "Sold"),
        }
    }
}

/// Source lot reference for blended lots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotSource {
    pub source_lot_id: Uuid,
    /// Proportion of this source in the blend (0-100)
    pub proportion_percent: Decimal,
}

/// Generate a traceability code
pub fn generate_traceability_code(business_code: &str, year: i32, sequence: i32) -> String {
    format!("CQM-{}-{}-{:04}", year, business_code, sequence)
}
