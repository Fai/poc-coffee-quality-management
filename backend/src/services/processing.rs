//! Processing management service for coffee processing operations

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::services::lot::LotStage;
use shared::{DryingLog, FermentationLog, ProcessingMethod};

/// Processing service for managing coffee processing records
#[derive(Clone)]
pub struct ProcessingService {
    db: PgPool,
}

/// Database row for processing record
#[derive(Debug, sqlx::FromRow)]
struct ProcessingRow {
    id: Uuid,
    lot_id: Uuid,
    method: String,
    method_details: Option<serde_json::Value>,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
    responsible_person: String,
    fermentation_log: Option<serde_json::Value>,
    drying_log: Option<serde_json::Value>,
    final_moisture_percent: Option<Decimal>,
    green_bean_weight_kg: Option<Decimal>,
    cherry_weight_kg: Option<Decimal>,
    processing_yield_percent: Option<Decimal>,
    notes: Option<String>,
    notes_th: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ProcessingRow> for ProcessingRecord {
    fn from(row: ProcessingRow) -> Self {
        ProcessingRecord {
            id: row.id,
            lot_id: row.lot_id,
            method: row.method,
            method_details: row.method_details,
            start_date: row.start_date,
            end_date: row.end_date,
            responsible_person: row.responsible_person,
            fermentation_log: row.fermentation_log,
            drying_log: row.drying_log,
            final_moisture_percent: row.final_moisture_percent,
            green_bean_weight_kg: row.green_bean_weight_kg,
            cherry_weight_kg: row.cherry_weight_kg,
            processing_yield_percent: row.processing_yield_percent,
            notes: row.notes,
            notes_th: row.notes_th,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Processing record
#[derive(Debug, Clone, Serialize)]
pub struct ProcessingRecord {
    pub id: Uuid,
    pub lot_id: Uuid,
    pub method: String,
    pub method_details: Option<serde_json::Value>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub responsible_person: String,
    pub fermentation_log: Option<serde_json::Value>,
    pub drying_log: Option<serde_json::Value>,
    pub final_moisture_percent: Option<Decimal>,
    pub green_bean_weight_kg: Option<Decimal>,
    pub cherry_weight_kg: Option<Decimal>,
    pub processing_yield_percent: Option<Decimal>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for starting processing
#[derive(Debug, Deserialize)]
pub struct StartProcessingInput {
    pub lot_id: Uuid,
    pub method: ProcessingMethod,
    pub start_date: NaiveDate,
    pub responsible_person: String,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
}

/// Input for logging fermentation
#[derive(Debug, Deserialize)]
pub struct LogFermentationInput {
    pub fermentation_log: FermentationLog,
}

/// Input for logging drying
#[derive(Debug, Deserialize)]
pub struct LogDryingInput {
    pub drying_log: DryingLog,
}

/// Input for completing processing
#[derive(Debug, Deserialize)]
pub struct CompleteProcessingInput {
    pub end_date: NaiveDate,
    pub final_moisture_percent: Decimal,
    pub green_bean_weight_kg: Decimal,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
}

impl ProcessingService {
    /// Create a new ProcessingService instance
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Start processing for a lot
    pub async fn start_processing(
        &self,
        business_id: Uuid,
        input: StartProcessingInput,
    ) -> AppResult<ProcessingRecord> {
        // Validate lot exists and belongs to business
        let lot = sqlx::query_as::<_, (Uuid, String, Decimal)>(
            "SELECT id, stage, current_weight_kg FROM lots WHERE id = $1 AND business_id = $2",
        )
        .bind(input.lot_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Lot".to_string()))?;

        // Validate lot is in Cherry stage
        if lot.1 != LotStage::Cherry.as_str() {
            return Err(AppError::Validation {
                field: "lot_id".to_string(),
                message: format!(
                    "Lot must be in Cherry stage to start processing, current stage: {}",
                    lot.1
                ),
                message_th: format!(
                    "ล็อตต้องอยู่ในสถานะเชอร์รี่เพื่อเริ่มการแปรรูป สถานะปัจจุบัน: {}",
                    lot.1
                ),
            });
        }

        // Check if processing already exists for this lot
        let existing =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM processing_records WHERE lot_id = $1")
                .bind(input.lot_id)
                .fetch_one(&self.db)
                .await?;

        if existing > 0 {
            return Err(AppError::Validation {
                field: "lot_id".to_string(),
                message: "Processing record already exists for this lot".to_string(),
                message_th: "มีบันทึกการแปรรูปสำหรับล็อตนี้แล้ว".to_string(),
            });
        }

        // Validate responsible person
        if input.responsible_person.trim().is_empty() {
            return Err(AppError::Validation {
                field: "responsible_person".to_string(),
                message: "Responsible person is required".to_string(),
                message_th: "ต้องระบุผู้รับผิดชอบ".to_string(),
            });
        }

        // Convert method to string and details
        let (method_str, method_details) = method_to_db(&input.method);

        // Create processing record
        let row = sqlx::query_as::<_, ProcessingRow>(
            r#"
            INSERT INTO processing_records (lot_id, method, method_details, start_date, responsible_person, cherry_weight_kg, notes, notes_th)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, lot_id, method, method_details, start_date, end_date, responsible_person,
                      fermentation_log, drying_log, final_moisture_percent, green_bean_weight_kg,
                      cherry_weight_kg, processing_yield_percent, notes, notes_th, created_at, updated_at
            "#,
        )
        .bind(input.lot_id)
        .bind(&method_str)
        .bind(&method_details)
        .bind(input.start_date)
        .bind(&input.responsible_person)
        .bind(lot.2) // cherry_weight_kg from lot
        .bind(&input.notes)
        .bind(&input.notes_th)
        .fetch_one(&self.db)
        .await?;

        Ok(row.into())
    }

    /// Log fermentation data
    pub async fn log_fermentation(
        &self,
        business_id: Uuid,
        processing_id: Uuid,
        input: LogFermentationInput,
    ) -> AppResult<ProcessingRecord> {
        // Validate processing record exists and belongs to business
        self.validate_processing_access(business_id, processing_id)
            .await?;

        // Validate fermentation log
        if input.fermentation_log.duration_hours <= 0 {
            return Err(AppError::Validation {
                field: "duration_hours".to_string(),
                message: "Fermentation duration must be positive".to_string(),
                message_th: "ระยะเวลาหมักต้องเป็นค่าบวก".to_string(),
            });
        }

        // Update fermentation log
        let fermentation_json = serde_json::to_value(&input.fermentation_log)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let row = sqlx::query_as::<_, ProcessingRow>(
            r#"
            UPDATE processing_records
            SET fermentation_log = $1
            WHERE id = $2
            RETURNING id, lot_id, method, method_details, start_date, end_date, responsible_person,
                      fermentation_log, drying_log, final_moisture_percent, green_bean_weight_kg,
                      cherry_weight_kg, processing_yield_percent, notes, notes_th, created_at, updated_at
            "#,
        )
        .bind(&fermentation_json)
        .bind(processing_id)
        .fetch_one(&self.db)
        .await?;

        Ok(row.into())
    }

    /// Log drying data
    pub async fn log_drying(
        &self,
        business_id: Uuid,
        processing_id: Uuid,
        input: LogDryingInput,
    ) -> AppResult<ProcessingRecord> {
        // Validate processing record exists and belongs to business
        self.validate_processing_access(business_id, processing_id)
            .await?;

        // Validate drying log
        if input.drying_log.target_moisture_percent <= Decimal::ZERO {
            return Err(AppError::Validation {
                field: "target_moisture_percent".to_string(),
                message: "Target moisture must be positive".to_string(),
                message_th: "ความชื้นเป้าหมายต้องเป็นค่าบวก".to_string(),
            });
        }

        // Update drying log
        let drying_json = serde_json::to_value(&input.drying_log)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let row = sqlx::query_as::<_, ProcessingRow>(
            r#"
            UPDATE processing_records
            SET drying_log = $1
            WHERE id = $2
            RETURNING id, lot_id, method, method_details, start_date, end_date, responsible_person,
                      fermentation_log, drying_log, final_moisture_percent, green_bean_weight_kg,
                      cherry_weight_kg, processing_yield_percent, notes, notes_th, created_at, updated_at
            "#,
        )
        .bind(&drying_json)
        .bind(processing_id)
        .fetch_one(&self.db)
        .await?;

        Ok(row.into())
    }

    /// Complete processing and update lot stage
    pub async fn complete_processing(
        &self,
        business_id: Uuid,
        processing_id: Uuid,
        input: CompleteProcessingInput,
    ) -> AppResult<ProcessingRecord> {
        // Validate processing record exists and belongs to business
        let (lot_id, cherry_weight) = self
            .validate_processing_access(business_id, processing_id)
            .await?;

        // Validate final moisture (typical range 10-12%)
        if input.final_moisture_percent < Decimal::ZERO
            || input.final_moisture_percent > Decimal::from(100)
        {
            return Err(AppError::Validation {
                field: "final_moisture_percent".to_string(),
                message: "Final moisture must be between 0 and 100%".to_string(),
                message_th: "ความชื้นสุดท้ายต้องอยู่ระหว่าง 0 ถึง 100%".to_string(),
            });
        }

        // Validate green bean weight
        if input.green_bean_weight_kg <= Decimal::ZERO {
            return Err(AppError::Validation {
                field: "green_bean_weight_kg".to_string(),
                message: "Green bean weight must be positive".to_string(),
                message_th: "น้ำหนักกาแฟกะลาต้องเป็นค่าบวก".to_string(),
            });
        }

        // Calculate processing yield
        let processing_yield = if let Some(cherry) = cherry_weight {
            if cherry > Decimal::ZERO {
                Some((input.green_bean_weight_kg / cherry) * Decimal::from(100))
            } else {
                None
            }
        } else {
            None
        };

        // Start transaction
        let mut tx = self.db.begin().await?;

        // Update processing record
        let row = sqlx::query_as::<_, ProcessingRow>(
            r#"
            UPDATE processing_records
            SET end_date = $1, final_moisture_percent = $2, green_bean_weight_kg = $3,
                processing_yield_percent = $4, notes = COALESCE($5, notes), notes_th = COALESCE($6, notes_th)
            WHERE id = $7
            RETURNING id, lot_id, method, method_details, start_date, end_date, responsible_person,
                      fermentation_log, drying_log, final_moisture_percent, green_bean_weight_kg,
                      cherry_weight_kg, processing_yield_percent, notes, notes_th, created_at, updated_at
            "#,
        )
        .bind(input.end_date)
        .bind(input.final_moisture_percent)
        .bind(input.green_bean_weight_kg)
        .bind(processing_yield)
        .bind(&input.notes)
        .bind(&input.notes_th)
        .bind(processing_id)
        .fetch_one(&mut *tx)
        .await?;

        // Update lot stage to GreenBean and weight
        sqlx::query(
            r#"
            UPDATE lots
            SET stage = $1, current_weight_kg = $2
            WHERE id = $3
            "#,
        )
        .bind(LotStage::GreenBean.as_str())
        .bind(input.green_bean_weight_kg)
        .bind(lot_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(row.into())
    }

    /// Get processing record by ID
    pub async fn get_processing(
        &self,
        business_id: Uuid,
        processing_id: Uuid,
    ) -> AppResult<ProcessingRecord> {
        let row = sqlx::query_as::<_, ProcessingRow>(
            r#"
            SELECT p.id, p.lot_id, p.method, p.method_details, p.start_date, p.end_date, p.responsible_person,
                   p.fermentation_log, p.drying_log, p.final_moisture_percent, p.green_bean_weight_kg,
                   p.cherry_weight_kg, p.processing_yield_percent, p.notes, p.notes_th, p.created_at, p.updated_at
            FROM processing_records p
            JOIN lots l ON l.id = p.lot_id
            WHERE p.id = $1 AND l.business_id = $2
            "#,
        )
        .bind(processing_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Processing record".to_string()))?;

        Ok(row.into())
    }

    /// Get processing record by lot ID
    pub async fn get_processing_by_lot(
        &self,
        business_id: Uuid,
        lot_id: Uuid,
    ) -> AppResult<Option<ProcessingRecord>> {
        let row = sqlx::query_as::<_, ProcessingRow>(
            r#"
            SELECT p.id, p.lot_id, p.method, p.method_details, p.start_date, p.end_date, p.responsible_person,
                   p.fermentation_log, p.drying_log, p.final_moisture_percent, p.green_bean_weight_kg,
                   p.cherry_weight_kg, p.processing_yield_percent, p.notes, p.notes_th, p.created_at, p.updated_at
            FROM processing_records p
            JOIN lots l ON l.id = p.lot_id
            WHERE p.lot_id = $1 AND l.business_id = $2
            "#,
        )
        .bind(lot_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    /// List all processing records for a business
    pub async fn list_processing(&self, business_id: Uuid) -> AppResult<Vec<ProcessingRecord>> {
        let rows = sqlx::query_as::<_, ProcessingRow>(
            r#"
            SELECT p.id, p.lot_id, p.method, p.method_details, p.start_date, p.end_date, p.responsible_person,
                   p.fermentation_log, p.drying_log, p.final_moisture_percent, p.green_bean_weight_kg,
                   p.cherry_weight_kg, p.processing_yield_percent, p.notes, p.notes_th, p.created_at, p.updated_at
            FROM processing_records p
            JOIN lots l ON l.id = p.lot_id
            WHERE l.business_id = $1
            ORDER BY p.start_date DESC
            "#,
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Validate processing record access and return lot_id and cherry_weight
    async fn validate_processing_access(
        &self,
        business_id: Uuid,
        processing_id: Uuid,
    ) -> AppResult<(Uuid, Option<Decimal>)> {
        let row = sqlx::query_as::<_, (Uuid, Option<Decimal>)>(
            r#"
            SELECT p.lot_id, p.cherry_weight_kg
            FROM processing_records p
            JOIN lots l ON l.id = p.lot_id
            WHERE p.id = $1 AND l.business_id = $2
            "#,
        )
        .bind(processing_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Processing record".to_string()))?;

        Ok(row)
    }
}

/// Convert ProcessingMethod to database representation
fn method_to_db(method: &ProcessingMethod) -> (String, Option<serde_json::Value>) {
    match method {
        ProcessingMethod::Natural => ("natural".to_string(), None),
        ProcessingMethod::Washed => ("washed".to_string(), None),
        ProcessingMethod::Honey { mucilage_percent } => (
            "honey".to_string(),
            Some(serde_json::json!({ "mucilage_percent": mucilage_percent })),
        ),
        ProcessingMethod::WetHulled => ("wet_hulled".to_string(), None),
        ProcessingMethod::Anaerobic { hours } => (
            "anaerobic".to_string(),
            Some(serde_json::json!({ "hours": hours })),
        ),
        ProcessingMethod::Custom(name) => (
            "custom".to_string(),
            Some(serde_json::json!({ "name": name })),
        ),
    }
}

/// Calculate processing yield percentage
pub fn calculate_processing_yield(cherry_weight: Decimal, green_bean_weight: Decimal) -> Decimal {
    if cherry_weight.is_zero() {
        Decimal::ZERO
    } else {
        (green_bean_weight / cherry_weight) * Decimal::from(100)
    }
}
