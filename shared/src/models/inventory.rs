//! Inventory management models

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::LotStage;

/// An inventory transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryTransaction {
    pub id: Uuid,
    pub lot_id: Uuid,
    pub transaction_type: TransactionType,
    pub quantity_kg: Decimal,
    pub from_stage: Option<LotStage>,
    pub to_stage: Option<LotStage>,
    /// Buyer/supplier name for sales/purchases
    pub counterparty: Option<String>,
    pub unit_price: Option<Decimal>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Types of inventory transactions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    /// Stage transition (e.g., Cherry -> Parchment)
    StageTransition,
    Sale,
    Purchase,
    Adjustment,
    Loss,
}

/// Inventory alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryAlert {
    pub id: Uuid,
    pub business_id: Uuid,
    pub lot_id: Option<Uuid>,
    pub stage: Option<LotStage>,
    pub threshold_kg: Decimal,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

/// Inventory summary for a business
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySummary {
    pub business_id: Uuid,
    pub by_stage: Vec<StageInventory>,
    pub total_kg: Decimal,
    pub total_value: Option<Decimal>,
}

/// Inventory for a specific stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageInventory {
    pub stage: LotStage,
    pub quantity_kg: Decimal,
    pub lot_count: i32,
    pub value: Option<Decimal>,
}
