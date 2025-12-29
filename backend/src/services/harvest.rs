//! Harvest management service for recording and tracking harvests

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use super::lot::{CreateLotInput, LotService};

/// Harvest service for managing coffee harvests
#[derive(Clone)]
pub struct HarvestService {
    db: PgPool,
}

/// Harvest information
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Harvest {
    pub id: Uuid,
    pub lot_id: Uuid,
    pub plot_id: Uuid,
    pub business_id: Uuid,
    pub harvest_date: NaiveDate,
    pub picker_name: Option<String>,
    pub cherry_weight_kg: Decimal,
    pub underripe_percent: i32,
    pub ripe_percent: i32,
    pub overripe_percent: i32,
    pub weather_snapshot: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database row for harvest with lot info
#[derive(Debug, Clone, sqlx::FromRow)]
struct HarvestWithLotRow {
    pub id: Uuid,
    pub lot_id: Uuid,
    pub plot_id: Uuid,
    pub business_id: Uuid,
    pub harvest_date: NaiveDate,
    pub picker_name: Option<String>,
    pub cherry_weight_kg: Decimal,
    pub underripe_percent: i32,
    pub ripe_percent: i32,
    pub overripe_percent: i32,
    pub weather_snapshot: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub lot_traceability_code: String,
    pub lot_name: String,
    pub plot_name: String,
}

/// Harvest with lot info for API response
#[derive(Debug, Clone, Serialize)]
pub struct HarvestWithLot {
    pub id: Uuid,
    pub lot_id: Uuid,
    pub plot_id: Uuid,
    pub business_id: Uuid,
    pub harvest_date: NaiveDate,
    pub picker_name: Option<String>,
    pub cherry_weight_kg: Decimal,
    pub underripe_percent: i32,
    pub ripe_percent: i32,
    pub overripe_percent: i32,
    pub weather_snapshot: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub lot_traceability_code: String,
    pub lot_name: String,
    pub plot_name: String,
}

impl From<HarvestWithLotRow> for HarvestWithLot {
    fn from(row: HarvestWithLotRow) -> Self {
        Self {
            id: row.id,
            lot_id: row.lot_id,
            plot_id: row.plot_id,
            business_id: row.business_id,
            harvest_date: row.harvest_date,
            picker_name: row.picker_name,
            cherry_weight_kg: row.cherry_weight_kg,
            underripe_percent: row.underripe_percent,
            ripe_percent: row.ripe_percent,
            overripe_percent: row.overripe_percent,
            weather_snapshot: row.weather_snapshot,
            notes: row.notes,
            notes_th: row.notes_th,
            created_at: row.created_at,
            updated_at: row.updated_at,
            lot_traceability_code: row.lot_traceability_code,
            lot_name: row.lot_name,
            plot_name: row.plot_name,
        }
    }
}

/// Input for recording a harvest
#[derive(Debug, Deserialize)]
pub struct RecordHarvestInput {
    pub plot_id: Uuid,
    pub harvest_date: NaiveDate,
    pub picker_name: Option<String>,
    pub cherry_weight_kg: Decimal,
    pub underripe_percent: i32,
    pub ripe_percent: i32,
    pub overripe_percent: i32,
    pub weather_snapshot: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
    /// Optional: specify existing lot to add harvest to
    pub lot_id: Option<Uuid>,
    /// Optional: name for new lot (if lot_id not provided)
    pub lot_name: Option<String>,
}

/// Input for updating a harvest
#[derive(Debug, Deserialize)]
pub struct UpdateHarvestInput {
    pub harvest_date: Option<NaiveDate>,
    pub picker_name: Option<String>,
    pub cherry_weight_kg: Option<Decimal>,
    pub underripe_percent: Option<i32>,
    pub ripe_percent: Option<i32>,
    pub overripe_percent: Option<i32>,
    pub weather_snapshot: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
}

/// Ripeness assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipenessAssessment {
    pub underripe_percent: i32,
    pub ripe_percent: i32,
    pub overripe_percent: i32,
}

impl RipenessAssessment {
    /// Validate that ripeness percentages sum to 100
    pub fn validate(&self) -> Result<(), String> {
        let total = self.underripe_percent + self.ripe_percent + self.overripe_percent;
        if total != 100 {
            return Err(format!(
                "Ripeness percentages must sum to 100, got {}",
                total
            ));
        }
        if self.underripe_percent < 0 || self.ripe_percent < 0 || self.overripe_percent < 0 {
            return Err("Ripeness percentages cannot be negative".to_string());
        }
        if self.underripe_percent > 100 || self.ripe_percent > 100 || self.overripe_percent > 100 {
            return Err("Ripeness percentages cannot exceed 100".to_string());
        }
        Ok(())
    }
}

impl HarvestService {
    /// Create a new HarvestService instance
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Get all harvests for a business
    pub async fn get_harvests(&self, business_id: Uuid) -> AppResult<Vec<HarvestWithLot>> {
        let rows = sqlx::query_as::<_, HarvestWithLotRow>(
            r#"
            SELECT h.id, h.lot_id, h.plot_id, h.business_id, h.harvest_date, h.picker_name,
                   h.cherry_weight_kg, h.underripe_percent, h.ripe_percent, h.overripe_percent,
                   h.weather_snapshot, h.notes, h.notes_th, h.created_at, h.updated_at,
                   l.traceability_code as lot_traceability_code, l.name as lot_name, p.name as plot_name
            FROM harvests h
            JOIN lots l ON l.id = h.lot_id
            JOIN plots p ON p.id = h.plot_id
            WHERE h.business_id = $1
            ORDER BY h.harvest_date DESC
            "#,
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(HarvestWithLot::from).collect())
    }

    /// Get harvests for a specific lot
    pub async fn get_harvests_by_lot(
        &self,
        business_id: Uuid,
        lot_id: Uuid,
    ) -> AppResult<Vec<Harvest>> {
        let harvests = sqlx::query_as::<_, Harvest>(
            r#"
            SELECT id, lot_id, plot_id, business_id, harvest_date, picker_name,
                   cherry_weight_kg, underripe_percent, ripe_percent, overripe_percent,
                   weather_snapshot, notes, notes_th, created_at, updated_at
            FROM harvests
            WHERE lot_id = $1 AND business_id = $2
            ORDER BY harvest_date DESC
            "#,
        )
        .bind(lot_id)
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(harvests)
    }

    /// Get a harvest by ID
    pub async fn get_harvest(
        &self,
        business_id: Uuid,
        harvest_id: Uuid,
    ) -> AppResult<HarvestWithLot> {
        let row = sqlx::query_as::<_, HarvestWithLotRow>(
            r#"
            SELECT h.id, h.lot_id, h.plot_id, h.business_id, h.harvest_date, h.picker_name,
                   h.cherry_weight_kg, h.underripe_percent, h.ripe_percent, h.overripe_percent,
                   h.weather_snapshot, h.notes, h.notes_th, h.created_at, h.updated_at,
                   l.traceability_code as lot_traceability_code, l.name as lot_name, p.name as plot_name
            FROM harvests h
            JOIN lots l ON l.id = h.lot_id
            JOIN plots p ON p.id = h.plot_id
            WHERE h.id = $1 AND h.business_id = $2
            "#,
        )
        .bind(harvest_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Harvest".to_string()))?;

        Ok(HarvestWithLot::from(row))
    }

    /// Record a new harvest
    pub async fn record_harvest(
        &self,
        business_id: Uuid,
        business_code: &str,
        input: RecordHarvestInput,
    ) -> AppResult<HarvestWithLot> {
        // Validate ripeness
        let ripeness = RipenessAssessment {
            underripe_percent: input.underripe_percent,
            ripe_percent: input.ripe_percent,
            overripe_percent: input.overripe_percent,
        };
        ripeness.validate().map_err(|msg| AppError::Validation {
            field: "ripeness".to_string(),
            message: msg.clone(),
            message_th: format!("เปอร์เซ็นต์ความสุกไม่ถูกต้อง: {}", msg),
        })?;

        // Validate cherry weight
        if input.cherry_weight_kg <= Decimal::ZERO {
            return Err(AppError::Validation {
                field: "cherry_weight_kg".to_string(),
                message: "Cherry weight must be greater than 0".to_string(),
                message_th: "น้ำหนักเชอร์รี่ต้องมากกว่า 0".to_string(),
            });
        }

        // Validate plot exists and belongs to business
        let plot_name = sqlx::query_scalar::<_, String>(
            "SELECT name FROM plots WHERE id = $1 AND business_id = $2"
        )
        .bind(input.plot_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Plot".to_string()))?;

        // Start transaction
        let mut tx = self.db.begin().await?;

        // Get or create lot
        let lot_id = if let Some(existing_lot_id) = input.lot_id {
            // Validate lot exists and belongs to business
            let exists = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM lots WHERE id = $1 AND business_id = $2"
            )
            .bind(existing_lot_id)
            .bind(business_id)
            .fetch_one(&mut *tx)
            .await?;

            if exists == 0 {
                return Err(AppError::NotFound("Lot".to_string()));
            }
            existing_lot_id
        } else {
            // Create new lot
            let lot_name = input.lot_name.unwrap_or_else(|| {
                format!("{} - {}", plot_name, input.harvest_date)
            });

            let lot_service = LotService::new(self.db.clone());
            let lot = lot_service.create_lot(
                business_id,
                business_code,
                CreateLotInput {
                    name: lot_name,
                    notes: None,
                    notes_th: None,
                },
            ).await?;
            lot.id
        };

        // Create harvest
        let harvest_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO harvests (lot_id, plot_id, business_id, harvest_date, picker_name,
                                  cherry_weight_kg, underripe_percent, ripe_percent, overripe_percent,
                                  weather_snapshot, notes, notes_th)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id
            "#,
        )
        .bind(lot_id)
        .bind(input.plot_id)
        .bind(business_id)
        .bind(input.harvest_date)
        .bind(&input.picker_name)
        .bind(input.cherry_weight_kg)
        .bind(input.underripe_percent)
        .bind(input.ripe_percent)
        .bind(input.overripe_percent)
        .bind(&input.weather_snapshot)
        .bind(&input.notes)
        .bind(&input.notes_th)
        .fetch_one(&mut *tx)
        .await?;

        // Update lot weight
        sqlx::query(
            "UPDATE lots SET current_weight_kg = current_weight_kg + $1 WHERE id = $2"
        )
        .bind(input.cherry_weight_kg)
        .bind(lot_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Return the created harvest
        self.get_harvest(business_id, harvest_id).await
    }

    /// Update a harvest
    pub async fn update_harvest(
        &self,
        business_id: Uuid,
        harvest_id: Uuid,
        input: UpdateHarvestInput,
    ) -> AppResult<HarvestWithLot> {
        // Get existing harvest
        let existing = sqlx::query_as::<_, Harvest>(
            r#"
            SELECT id, lot_id, plot_id, business_id, harvest_date, picker_name,
                   cherry_weight_kg, underripe_percent, ripe_percent, overripe_percent,
                   weather_snapshot, notes, notes_th, created_at, updated_at
            FROM harvests
            WHERE id = $1 AND business_id = $2
            "#,
        )
        .bind(harvest_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Harvest".to_string()))?;

        // Prepare updated values
        let harvest_date = input.harvest_date.unwrap_or(existing.harvest_date);
        let picker_name = input.picker_name.or(existing.picker_name);
        let cherry_weight_kg = input.cherry_weight_kg.unwrap_or(existing.cherry_weight_kg);
        let underripe_percent = input.underripe_percent.unwrap_or(existing.underripe_percent);
        let ripe_percent = input.ripe_percent.unwrap_or(existing.ripe_percent);
        let overripe_percent = input.overripe_percent.unwrap_or(existing.overripe_percent);
        let weather_snapshot = input.weather_snapshot.or(existing.weather_snapshot);
        let notes = input.notes.or(existing.notes);
        let notes_th = input.notes_th.or(existing.notes_th);

        // Validate ripeness if any changed
        if input.underripe_percent.is_some() || input.ripe_percent.is_some() || input.overripe_percent.is_some() {
            let ripeness = RipenessAssessment {
                underripe_percent,
                ripe_percent,
                overripe_percent,
            };
            ripeness.validate().map_err(|msg| AppError::Validation {
                field: "ripeness".to_string(),
                message: msg.clone(),
                message_th: format!("เปอร์เซ็นต์ความสุกไม่ถูกต้อง: {}", msg),
            })?;
        }

        // Start transaction
        let mut tx = self.db.begin().await?;

        // Update lot weight if cherry weight changed
        if input.cherry_weight_kg.is_some() {
            let weight_diff = cherry_weight_kg - existing.cherry_weight_kg;
            sqlx::query(
                "UPDATE lots SET current_weight_kg = current_weight_kg + $1 WHERE id = $2"
            )
            .bind(weight_diff)
            .bind(existing.lot_id)
            .execute(&mut *tx)
            .await?;
        }

        // Update harvest
        sqlx::query(
            r#"
            UPDATE harvests
            SET harvest_date = $1, picker_name = $2, cherry_weight_kg = $3,
                underripe_percent = $4, ripe_percent = $5, overripe_percent = $6,
                weather_snapshot = $7, notes = $8, notes_th = $9
            WHERE id = $10
            "#,
        )
        .bind(harvest_date)
        .bind(&picker_name)
        .bind(cherry_weight_kg)
        .bind(underripe_percent)
        .bind(ripe_percent)
        .bind(overripe_percent)
        .bind(&weather_snapshot)
        .bind(&notes)
        .bind(&notes_th)
        .bind(harvest_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Return updated harvest
        self.get_harvest(business_id, harvest_id).await
    }

    /// Delete a harvest
    pub async fn delete_harvest(
        &self,
        business_id: Uuid,
        harvest_id: Uuid,
    ) -> AppResult<()> {
        // Get harvest to update lot weight
        let harvest = sqlx::query_as::<_, (Uuid, Decimal)>(
            "SELECT lot_id, cherry_weight_kg FROM harvests WHERE id = $1 AND business_id = $2"
        )
        .bind(harvest_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Harvest".to_string()))?;

        // Start transaction
        let mut tx = self.db.begin().await?;

        // Update lot weight
        sqlx::query(
            "UPDATE lots SET current_weight_kg = current_weight_kg - $1 WHERE id = $2"
        )
        .bind(harvest.1)
        .bind(harvest.0)
        .execute(&mut *tx)
        .await?;

        // Delete harvest
        sqlx::query("DELETE FROM harvests WHERE id = $1")
            .bind(harvest_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Calculate yield per rai for a plot
    pub fn calculate_yield_per_rai(
        total_cherry_weight_kg: Decimal,
        area_rai: Decimal,
    ) -> Option<Decimal> {
        if area_rai > Decimal::ZERO {
            Some(total_cherry_weight_kg / area_rai)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ripeness_validation_valid() {
        let ripeness = RipenessAssessment {
            underripe_percent: 10,
            ripe_percent: 80,
            overripe_percent: 10,
        };
        assert!(ripeness.validate().is_ok());
    }

    #[test]
    fn test_ripeness_validation_not_100() {
        let ripeness = RipenessAssessment {
            underripe_percent: 10,
            ripe_percent: 80,
            overripe_percent: 5,
        };
        assert!(ripeness.validate().is_err());
    }

    #[test]
    fn test_ripeness_validation_negative() {
        let ripeness = RipenessAssessment {
            underripe_percent: -10,
            ripe_percent: 100,
            overripe_percent: 10,
        };
        assert!(ripeness.validate().is_err());
    }

    #[test]
    fn test_yield_calculation() {
        let weight = Decimal::from(100);
        let area = Decimal::from(2);
        let yield_per_rai = HarvestService::calculate_yield_per_rai(weight, area);
        assert_eq!(yield_per_rai, Some(Decimal::from(50)));
    }

    #[test]
    fn test_yield_calculation_zero_area() {
        let weight = Decimal::from(100);
        let area = Decimal::ZERO;
        let yield_per_rai = HarvestService::calculate_yield_per_rai(weight, area);
        assert_eq!(yield_per_rai, None);
    }
}
