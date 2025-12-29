//! Inventory management service for tracking stock movements and alerts

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Inventory service for managing stock transactions and alerts
#[derive(Clone)]
pub struct InventoryService {
    db: PgPool,
}

/// Inventory transaction types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "inventory_transaction_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    HarvestIn,
    ProcessingOut,
    ProcessingIn,
    RoastingOut,
    RoastingIn,
    Sale,
    Purchase,
    Adjustment,
    Transfer,
    Sample,
    Return,
}

impl TransactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionType::HarvestIn => "harvest_in",
            TransactionType::ProcessingOut => "processing_out",
            TransactionType::ProcessingIn => "processing_in",
            TransactionType::RoastingOut => "roasting_out",
            TransactionType::RoastingIn => "roasting_in",
            TransactionType::Sale => "sale",
            TransactionType::Purchase => "purchase",
            TransactionType::Adjustment => "adjustment",
            TransactionType::Transfer => "transfer",
            TransactionType::Sample => "sample",
            TransactionType::Return => "return",
        }
    }
}

/// Transaction direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionDirection {
    In,
    Out,
}

impl TransactionDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionDirection::In => "in",
            TransactionDirection::Out => "out",
        }
    }
}

/// Inventory transaction record
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct InventoryTransaction {
    pub id: Uuid,
    pub business_id: Uuid,
    pub lot_id: Uuid,
    pub transaction_type: TransactionType,
    pub quantity_kg: Decimal,
    pub direction: String,
    pub stage: String,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub counterparty_name: Option<String>,
    pub counterparty_contact: Option<String>,
    pub unit_price: Option<Decimal>,
    pub total_price: Option<Decimal>,
    pub currency: String,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
    pub transaction_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

/// Input for recording inventory transaction
#[derive(Debug, Deserialize)]
pub struct RecordTransactionInput {
    pub lot_id: Uuid,
    pub transaction_type: TransactionType,
    pub quantity_kg: Decimal,
    pub direction: TransactionDirection,
    pub stage: String,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub counterparty_name: Option<String>,
    pub counterparty_contact: Option<String>,
    pub unit_price: Option<Decimal>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
    pub transaction_date: Option<NaiveDate>,
}

/// Inventory balance for a lot
#[derive(Debug, Clone, Serialize)]
pub struct InventoryBalance {
    pub lot_id: Uuid,
    pub lot_name: String,
    pub traceability_code: String,
    pub stage: String,
    pub balance_kg: Decimal,
    pub total_in_kg: Decimal,
    pub total_out_kg: Decimal,
}

/// Row for balance query
#[derive(Debug, FromRow)]
struct BalanceRow {
    id: Uuid,
    name: String,
    traceability_code: String,
    stage: String,
    total_in: Decimal,
    total_out: Decimal,
}

/// Inventory alert configuration
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct InventoryAlert {
    pub id: Uuid,
    pub business_id: Uuid,
    pub lot_id: Option<Uuid>,
    pub stage: Option<String>,
    pub threshold_kg: Decimal,
    pub is_active: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub notify_email: bool,
    pub notify_line: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating inventory alert
#[derive(Debug, Deserialize)]
pub struct CreateAlertInput {
    pub lot_id: Option<Uuid>,
    pub stage: Option<String>,
    pub threshold_kg: Decimal,
    pub notify_email: Option<bool>,
    pub notify_line: Option<bool>,
}

/// Input for updating inventory alert
#[derive(Debug, Deserialize)]
pub struct UpdateAlertInput {
    pub threshold_kg: Option<Decimal>,
    pub is_active: Option<bool>,
    pub notify_email: Option<bool>,
    pub notify_line: Option<bool>,
}

/// Inventory valuation for a lot
#[derive(Debug, Clone, Serialize)]
pub struct InventoryValuation {
    pub lot_id: Uuid,
    pub lot_name: String,
    pub traceability_code: String,
    pub stage: String,
    pub quantity_kg: Decimal,
    pub unit_cost: Decimal,
    pub total_value: Decimal,
    pub currency: String,
}

/// Inventory summary by stage
#[derive(Debug, Clone, Serialize)]
pub struct InventorySummary {
    pub stage: String,
    pub total_quantity_kg: Decimal,
    pub lot_count: i64,
    pub total_value: Option<Decimal>,
    pub currency: String,
}

/// Row for triggered alert query
#[derive(Debug, FromRow)]
struct TriggeredAlertRow {
    id: Uuid,
    business_id: Uuid,
    lot_id: Option<Uuid>,
    stage: Option<String>,
    threshold_kg: Decimal,
    is_active: bool,
    last_triggered_at: Option<DateTime<Utc>>,
    notify_email: bool,
    notify_line: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    current_balance: Decimal,
}

impl InventoryService {
    /// Create a new InventoryService instance
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Record an inventory transaction
    pub async fn record_transaction(
        &self,
        business_id: Uuid,
        user_id: Uuid,
        input: RecordTransactionInput,
    ) -> AppResult<InventoryTransaction> {
        // Validate quantity
        if input.quantity_kg <= Decimal::ZERO {
            return Err(AppError::Validation {
                field: "quantity_kg".to_string(),
                message: "Quantity must be positive".to_string(),
                message_th: "ปริมาณต้องเป็นค่าบวก".to_string(),
            });
        }

        // Validate lot belongs to business
        let lot_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM lots WHERE id = $1 AND business_id = $2)"
        )
        .bind(input.lot_id)
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        if !lot_exists {
            return Err(AppError::NotFound("Lot".to_string()));
        }

        // Calculate total price if unit price provided
        let total_price = input.unit_price.map(|up| up * input.quantity_kg);
        let currency = input.currency.unwrap_or_else(|| "THB".to_string());
        let transaction_date = input.transaction_date.unwrap_or_else(|| Utc::now().date_naive());

        let transaction = sqlx::query_as::<_, InventoryTransaction>(
            r#"
            INSERT INTO inventory_transactions (
                business_id, lot_id, transaction_type, quantity_kg, direction, stage,
                reference_type, reference_id, counterparty_name, counterparty_contact,
                unit_price, total_price, currency, notes, notes_th, transaction_date, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            RETURNING id, business_id, lot_id, transaction_type, quantity_kg, direction, stage,
                      reference_type, reference_id, counterparty_name, counterparty_contact,
                      unit_price, total_price, currency, notes, notes_th, transaction_date,
                      created_at, created_by
            "#,
        )
        .bind(business_id)
        .bind(input.lot_id)
        .bind(input.transaction_type)
        .bind(input.quantity_kg)
        .bind(input.direction.as_str())
        .bind(&input.stage)
        .bind(&input.reference_type)
        .bind(input.reference_id)
        .bind(&input.counterparty_name)
        .bind(&input.counterparty_contact)
        .bind(input.unit_price)
        .bind(total_price)
        .bind(&currency)
        .bind(&input.notes)
        .bind(&input.notes_th)
        .bind(transaction_date)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(transaction)
    }

    /// Get inventory balance for a lot
    pub async fn get_balance(&self, business_id: Uuid, lot_id: Uuid) -> AppResult<InventoryBalance> {
        let row = sqlx::query_as::<_, BalanceRow>(
            r#"
            SELECT l.id, l.name, l.traceability_code, l.stage,
                   COALESCE(SUM(CASE WHEN it.direction = 'in' THEN it.quantity_kg ELSE 0 END), 0) as total_in,
                   COALESCE(SUM(CASE WHEN it.direction = 'out' THEN it.quantity_kg ELSE 0 END), 0) as total_out
            FROM lots l
            LEFT JOIN inventory_transactions it ON it.lot_id = l.id
            WHERE l.id = $1 AND l.business_id = $2
            GROUP BY l.id, l.name, l.traceability_code, l.stage
            "#,
        )
        .bind(lot_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Lot".to_string()))?;

        Ok(InventoryBalance {
            lot_id: row.id,
            lot_name: row.name,
            traceability_code: row.traceability_code,
            stage: row.stage,
            balance_kg: row.total_in - row.total_out,
            total_in_kg: row.total_in,
            total_out_kg: row.total_out,
        })
    }

    /// Get transactions for a lot
    pub async fn get_transactions(
        &self,
        business_id: Uuid,
        lot_id: Uuid,
    ) -> AppResult<Vec<InventoryTransaction>> {
        // Validate lot belongs to business
        let lot_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM lots WHERE id = $1 AND business_id = $2)"
        )
        .bind(lot_id)
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        if !lot_exists {
            return Err(AppError::NotFound("Lot".to_string()));
        }

        let transactions = sqlx::query_as::<_, InventoryTransaction>(
            r#"
            SELECT id, business_id, lot_id, transaction_type, quantity_kg, direction, stage,
                   reference_type, reference_id, counterparty_name, counterparty_contact,
                   unit_price, total_price, currency, notes, notes_th, transaction_date,
                   created_at, created_by
            FROM inventory_transactions
            WHERE lot_id = $1 AND business_id = $2
            ORDER BY transaction_date DESC, created_at DESC
            "#,
        )
        .bind(lot_id)
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(transactions)
    }

    /// List all transactions for a business
    pub async fn list_transactions(
        &self,
        business_id: Uuid,
    ) -> AppResult<Vec<InventoryTransaction>> {
        let transactions = sqlx::query_as::<_, InventoryTransaction>(
            r#"
            SELECT id, business_id, lot_id, transaction_type, quantity_kg, direction, stage,
                   reference_type, reference_id, counterparty_name, counterparty_contact,
                   unit_price, total_price, currency, notes, notes_th, transaction_date,
                   created_at, created_by
            FROM inventory_transactions
            WHERE business_id = $1
            ORDER BY transaction_date DESC, created_at DESC
            "#,
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(transactions)
    }


    /// Create an inventory alert
    pub async fn create_alert(
        &self,
        business_id: Uuid,
        input: CreateAlertInput,
    ) -> AppResult<InventoryAlert> {
        // Validate that either lot_id or stage is provided
        if input.lot_id.is_none() && input.stage.is_none() {
            return Err(AppError::Validation {
                field: "lot_id/stage".to_string(),
                message: "Either lot_id or stage must be provided".to_string(),
                message_th: "ต้องระบุ lot_id หรือ stage อย่างใดอย่างหนึ่ง".to_string(),
            });
        }

        // Validate threshold
        if input.threshold_kg <= Decimal::ZERO {
            return Err(AppError::Validation {
                field: "threshold_kg".to_string(),
                message: "Threshold must be positive".to_string(),
                message_th: "เกณฑ์ต้องเป็นค่าบวก".to_string(),
            });
        }

        // Validate lot belongs to business if provided
        if let Some(lot_id) = input.lot_id {
            let lot_exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM lots WHERE id = $1 AND business_id = $2)"
            )
            .bind(lot_id)
            .bind(business_id)
            .fetch_one(&self.db)
            .await?;

            if !lot_exists {
                return Err(AppError::NotFound("Lot".to_string()));
            }
        }

        let notify_email = input.notify_email.unwrap_or(true);
        let notify_line = input.notify_line.unwrap_or(true);

        let alert = sqlx::query_as::<_, InventoryAlert>(
            r#"
            INSERT INTO inventory_alerts (business_id, lot_id, stage, threshold_kg, notify_email, notify_line)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, business_id, lot_id, stage, threshold_kg, is_active, last_triggered_at,
                      notify_email, notify_line, created_at, updated_at
            "#,
        )
        .bind(business_id)
        .bind(input.lot_id)
        .bind(&input.stage)
        .bind(input.threshold_kg)
        .bind(notify_email)
        .bind(notify_line)
        .fetch_one(&self.db)
        .await?;

        Ok(alert)
    }

    /// Update an inventory alert
    pub async fn update_alert(
        &self,
        business_id: Uuid,
        alert_id: Uuid,
        input: UpdateAlertInput,
    ) -> AppResult<InventoryAlert> {
        // Check if alert exists
        let existing = sqlx::query_as::<_, (Decimal, bool, bool, bool)>(
            "SELECT threshold_kg, is_active, notify_email, notify_line FROM inventory_alerts WHERE id = $1 AND business_id = $2"
        )
        .bind(alert_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Alert".to_string()))?;

        let threshold_kg = input.threshold_kg.unwrap_or(existing.0);
        let is_active = input.is_active.unwrap_or(existing.1);
        let notify_email = input.notify_email.unwrap_or(existing.2);
        let notify_line = input.notify_line.unwrap_or(existing.3);

        // Validate threshold
        if threshold_kg <= Decimal::ZERO {
            return Err(AppError::Validation {
                field: "threshold_kg".to_string(),
                message: "Threshold must be positive".to_string(),
                message_th: "เกณฑ์ต้องเป็นค่าบวก".to_string(),
            });
        }

        let alert = sqlx::query_as::<_, InventoryAlert>(
            r#"
            UPDATE inventory_alerts
            SET threshold_kg = $1, is_active = $2, notify_email = $3, notify_line = $4
            WHERE id = $5
            RETURNING id, business_id, lot_id, stage, threshold_kg, is_active, last_triggered_at,
                      notify_email, notify_line, created_at, updated_at
            "#,
        )
        .bind(threshold_kg)
        .bind(is_active)
        .bind(notify_email)
        .bind(notify_line)
        .bind(alert_id)
        .fetch_one(&self.db)
        .await?;

        Ok(alert)
    }

    /// Delete an inventory alert
    pub async fn delete_alert(&self, business_id: Uuid, alert_id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            "DELETE FROM inventory_alerts WHERE id = $1 AND business_id = $2"
        )
        .bind(alert_id)
        .bind(business_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Alert".to_string()));
        }

        Ok(())
    }

    /// List alerts for a business
    pub async fn list_alerts(&self, business_id: Uuid) -> AppResult<Vec<InventoryAlert>> {
        let alerts = sqlx::query_as::<_, InventoryAlert>(
            r#"
            SELECT id, business_id, lot_id, stage, threshold_kg, is_active, last_triggered_at,
                   notify_email, notify_line, created_at, updated_at
            FROM inventory_alerts
            WHERE business_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(alerts)
    }

    /// Get triggered alerts (alerts where current balance is below threshold)
    pub async fn get_triggered_alerts(&self, business_id: Uuid) -> AppResult<Vec<(InventoryAlert, Decimal)>> {
        let rows = sqlx::query_as::<_, TriggeredAlertRow>(
            r#"
            SELECT ia.id, ia.business_id, ia.lot_id, ia.stage, ia.threshold_kg, ia.is_active,
                   ia.last_triggered_at, ia.notify_email, ia.notify_line, ia.created_at, ia.updated_at,
                   COALESCE(get_lot_inventory_balance(ia.lot_id), 0) as current_balance
            FROM inventory_alerts ia
            WHERE ia.business_id = $1 AND ia.is_active = true
            AND ia.lot_id IS NOT NULL
            AND COALESCE(get_lot_inventory_balance(ia.lot_id), 0) <= ia.threshold_kg
            "#,
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(|r| (
            InventoryAlert {
                id: r.id,
                business_id: r.business_id,
                lot_id: r.lot_id,
                stage: r.stage,
                threshold_kg: r.threshold_kg,
                is_active: r.is_active,
                last_triggered_at: r.last_triggered_at,
                notify_email: r.notify_email,
                notify_line: r.notify_line,
                created_at: r.created_at,
                updated_at: r.updated_at,
            },
            r.current_balance,
        )).collect())
    }

    /// Get inventory valuation for a lot
    pub async fn get_valuation(&self, business_id: Uuid, lot_id: Uuid) -> AppResult<InventoryValuation> {
        let balance = self.get_balance(business_id, lot_id).await?;

        // Calculate weighted average cost from purchase/harvest transactions
        let avg_cost = sqlx::query_scalar::<_, Option<Decimal>>(
            r#"
            SELECT CASE 
                WHEN SUM(quantity_kg) > 0 THEN SUM(total_price) / SUM(quantity_kg)
                ELSE 0
            END
            FROM inventory_transactions
            WHERE lot_id = $1 AND direction = 'in' AND unit_price IS NOT NULL
            "#,
        )
        .bind(lot_id)
        .fetch_one(&self.db)
        .await?
        .unwrap_or(Decimal::ZERO);

        let total_value = balance.balance_kg * avg_cost;

        Ok(InventoryValuation {
            lot_id: balance.lot_id,
            lot_name: balance.lot_name,
            traceability_code: balance.traceability_code,
            stage: balance.stage,
            quantity_kg: balance.balance_kg,
            unit_cost: avg_cost,
            total_value,
            currency: "THB".to_string(),
        })
    }

    /// Get inventory summary by stage
    pub async fn get_summary_by_stage(&self, business_id: Uuid) -> AppResult<Vec<InventorySummary>> {
        let rows = sqlx::query_as::<_, (String, Decimal, i64, Option<Decimal>)>(
            r#"
            SELECT l.stage,
                   COALESCE(SUM(
                       COALESCE((SELECT SUM(CASE WHEN direction = 'in' THEN quantity_kg ELSE -quantity_kg END)
                                 FROM inventory_transactions WHERE lot_id = l.id), 0)
                   ), 0) as total_quantity,
                   COUNT(DISTINCT l.id) as lot_count,
                   SUM(
                       COALESCE((SELECT SUM(CASE WHEN direction = 'in' THEN quantity_kg ELSE -quantity_kg END)
                                 FROM inventory_transactions WHERE lot_id = l.id), 0) *
                       COALESCE((SELECT CASE WHEN SUM(quantity_kg) > 0 THEN SUM(total_price) / SUM(quantity_kg) ELSE 0 END
                                 FROM inventory_transactions WHERE lot_id = l.id AND direction = 'in' AND unit_price IS NOT NULL), 0)
                   ) as total_value
            FROM lots l
            WHERE l.business_id = $1
            GROUP BY l.stage
            ORDER BY l.stage
            "#,
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(|r| InventorySummary {
            stage: r.0,
            total_quantity_kg: r.1,
            lot_count: r.2,
            total_value: r.3,
            currency: "THB".to_string(),
        }).collect())
    }
}
