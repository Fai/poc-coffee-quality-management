//! Lot management service for traceability and lot operations

use chrono::{DateTime, Datelike, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Lot service for managing coffee lots and traceability
#[derive(Clone)]
pub struct LotService {
    db: PgPool,
}

/// Lot stage in the supply chain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LotStage {
    Cherry,
    Parchment,
    GreenBean,
    RoastedBean,
    Sold,
}

impl LotStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            LotStage::Cherry => "cherry",
            LotStage::Parchment => "parchment",
            LotStage::GreenBean => "green_bean",
            LotStage::RoastedBean => "roasted_bean",
            LotStage::Sold => "sold",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "cherry" => Some(LotStage::Cherry),
            "parchment" => Some(LotStage::Parchment),
            "green_bean" => Some(LotStage::GreenBean),
            "roasted_bean" => Some(LotStage::RoastedBean),
            "sold" => Some(LotStage::Sold),
            _ => None,
        }
    }
}

/// Lot information
#[derive(Debug, Clone, Serialize)]
pub struct Lot {
    pub id: Uuid,
    pub business_id: Uuid,
    pub traceability_code: String,
    pub name: String,
    pub stage: String,
    pub current_weight_kg: Decimal,
    pub qr_code_url: Option<String>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lot source for blended lots
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LotSource {
    pub id: Uuid,
    pub lot_id: Uuid,
    pub source_lot_id: Uuid,
    pub proportion_percent: Decimal,
    pub created_at: DateTime<Utc>,
}

/// Lot with its sources
#[derive(Debug, Clone, Serialize)]
pub struct LotWithSources {
    #[serde(flatten)]
    pub lot: Lot,
    pub sources: Vec<LotSourceInfo>,
}

/// Source lot info for display
#[derive(Debug, Clone, Serialize)]
pub struct LotSourceInfo {
    pub source_lot_id: Uuid,
    pub source_traceability_code: String,
    pub source_name: String,
    pub proportion_percent: Decimal,
}

/// Input for creating a lot
#[derive(Debug, Deserialize)]
pub struct CreateLotInput {
    pub name: String,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
}

/// Input for blending lots
#[derive(Debug, Deserialize)]
pub struct BlendLotsInput {
    pub name: String,
    pub sources: Vec<BlendSourceInput>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
}

/// Source lot for blending
#[derive(Debug, Deserialize)]
pub struct BlendSourceInput {
    pub source_lot_id: Uuid,
    pub proportion_percent: Decimal,
}

/// Input for updating a lot
#[derive(Debug, Deserialize)]
pub struct UpdateLotInput {
    pub name: Option<String>,
    pub stage: Option<String>,
    pub current_weight_kg: Option<Decimal>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
}

impl LotService {
    /// Create a new LotService instance
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Generate unique traceability code: CQM-YYYY-BIZ-NNNN
    pub async fn generate_traceability_code(
        &self,
        business_id: Uuid,
        business_code: &str,
    ) -> AppResult<String> {
        let year = Utc::now().year();
        
        // Get next sequence number
        let sequence: i32 = sqlx::query_scalar(
            "SELECT get_next_lot_sequence($1, $2)"
        )
        .bind(business_id)
        .bind(year)
        .fetch_one(&self.db)
        .await?;

        Ok(format!("CQM-{}-{}-{:04}", year, business_code, sequence))
    }

    /// Get all lots for a business
    pub async fn get_lots(&self, business_id: Uuid) -> AppResult<Vec<Lot>> {
        let rows = sqlx::query_as::<_, (Uuid, Uuid, String, String, String, Decimal, Option<String>, Option<String>, Option<String>, DateTime<Utc>, DateTime<Utc>)>(
            r#"
            SELECT id, business_id, traceability_code, name, stage, current_weight_kg,
                   qr_code_url, notes, notes_th, created_at, updated_at
            FROM lots
            WHERE business_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(|r| Lot {
            id: r.0,
            business_id: r.1,
            traceability_code: r.2,
            name: r.3,
            stage: r.4,
            current_weight_kg: r.5,
            qr_code_url: r.6,
            notes: r.7,
            notes_th: r.8,
            created_at: r.9,
            updated_at: r.10,
        }).collect())
    }

    /// Get a lot by ID with its sources
    pub async fn get_lot_with_sources(
        &self,
        business_id: Uuid,
        lot_id: Uuid,
    ) -> AppResult<LotWithSources> {
        // Get lot
        let row = sqlx::query_as::<_, (Uuid, Uuid, String, String, String, Decimal, Option<String>, Option<String>, Option<String>, DateTime<Utc>, DateTime<Utc>)>(
            r#"
            SELECT id, business_id, traceability_code, name, stage, current_weight_kg,
                   qr_code_url, notes, notes_th, created_at, updated_at
            FROM lots
            WHERE id = $1 AND business_id = $2
            "#,
        )
        .bind(lot_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Lot".to_string()))?;

        let lot = Lot {
            id: row.0,
            business_id: row.1,
            traceability_code: row.2,
            name: row.3,
            stage: row.4,
            current_weight_kg: row.5,
            qr_code_url: row.6,
            notes: row.7,
            notes_th: row.8,
            created_at: row.9,
            updated_at: row.10,
        };

        // Get sources
        let sources = sqlx::query_as::<_, (Uuid, String, String, Decimal)>(
            r#"
            SELECT ls.source_lot_id, l.traceability_code, l.name, ls.proportion_percent
            FROM lot_sources ls
            JOIN lots l ON l.id = ls.source_lot_id
            WHERE ls.lot_id = $1
            ORDER BY ls.proportion_percent DESC
            "#,
        )
        .bind(lot_id)
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(|r| LotSourceInfo {
            source_lot_id: r.0,
            source_traceability_code: r.1,
            source_name: r.2,
            proportion_percent: r.3,
        })
        .collect();

        Ok(LotWithSources { lot, sources })
    }

    /// Create a new lot (internal use - typically created via harvest)
    pub async fn create_lot(
        &self,
        business_id: Uuid,
        business_code: &str,
        input: CreateLotInput,
    ) -> AppResult<Lot> {
        // Validate input
        if input.name.trim().is_empty() {
            return Err(AppError::Validation {
                field: "name".to_string(),
                message: "Lot name cannot be empty".to_string(),
                message_th: "ชื่อล็อตไม่สามารถว่างได้".to_string(),
            });
        }

        // Generate traceability code
        let traceability_code = self.generate_traceability_code(business_id, business_code).await?;

        // Generate QR code URL
        let qr_code_url = format!("https://trace.coffeeqm.com/{}", traceability_code);

        // Create lot
        let row = sqlx::query_as::<_, (Uuid, Uuid, String, String, String, Decimal, Option<String>, Option<String>, Option<String>, DateTime<Utc>, DateTime<Utc>)>(
            r#"
            INSERT INTO lots (business_id, traceability_code, name, stage, qr_code_url, notes, notes_th)
            VALUES ($1, $2, $3, 'cherry', $4, $5, $6)
            RETURNING id, business_id, traceability_code, name, stage, current_weight_kg,
                      qr_code_url, notes, notes_th, created_at, updated_at
            "#,
        )
        .bind(business_id)
        .bind(&traceability_code)
        .bind(&input.name)
        .bind(&qr_code_url)
        .bind(&input.notes)
        .bind(&input.notes_th)
        .fetch_one(&self.db)
        .await?;

        Ok(Lot {
            id: row.0,
            business_id: row.1,
            traceability_code: row.2,
            name: row.3,
            stage: row.4,
            current_weight_kg: row.5,
            qr_code_url: row.6,
            notes: row.7,
            notes_th: row.8,
            created_at: row.9,
            updated_at: row.10,
        })
    }

    /// Blend multiple lots into a new lot
    pub async fn blend_lots(
        &self,
        business_id: Uuid,
        business_code: &str,
        input: BlendLotsInput,
    ) -> AppResult<LotWithSources> {
        // Validate input
        if input.name.trim().is_empty() {
            return Err(AppError::Validation {
                field: "name".to_string(),
                message: "Lot name cannot be empty".to_string(),
                message_th: "ชื่อล็อตไม่สามารถว่างได้".to_string(),
            });
        }

        if input.sources.is_empty() {
            return Err(AppError::Validation {
                field: "sources".to_string(),
                message: "At least one source lot is required".to_string(),
                message_th: "ต้องมีล็อตต้นทางอย่างน้อยหนึ่งล็อต".to_string(),
            });
        }

        // Validate proportions sum to 100
        let total_proportion: Decimal = input.sources.iter()
            .map(|s| s.proportion_percent)
            .sum();
        
        if total_proportion != Decimal::from(100) {
            return Err(AppError::Validation {
                field: "sources".to_string(),
                message: format!("Source proportions must sum to 100%, got {}%", total_proportion),
                message_th: format!("สัดส่วนต้นทางต้องรวมกันเป็น 100% ได้ {}%", total_proportion),
            });
        }

        // Validate all source lots exist and belong to business
        let mut total_weight = Decimal::ZERO;
        for source in &input.sources {
            let source_lot = sqlx::query_as::<_, (Decimal, String)>(
                "SELECT current_weight_kg, stage FROM lots WHERE id = $1 AND business_id = $2"
            )
            .bind(source.source_lot_id)
            .bind(business_id)
            .fetch_optional(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Source lot {}", source.source_lot_id)))?;

            // Calculate weighted contribution
            total_weight += source_lot.0 * source.proportion_percent / Decimal::from(100);
        }

        // Start transaction
        let mut tx = self.db.begin().await?;

        // Generate traceability code
        let traceability_code = self.generate_traceability_code(business_id, business_code).await?;
        let qr_code_url = format!("https://trace.coffeeqm.com/{}", traceability_code);

        // Create new blended lot
        let lot_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO lots (business_id, traceability_code, name, stage, current_weight_kg, qr_code_url, notes, notes_th)
            VALUES ($1, $2, $3, 'cherry', $4, $5, $6, $7)
            RETURNING id
            "#,
        )
        .bind(business_id)
        .bind(&traceability_code)
        .bind(&input.name)
        .bind(total_weight)
        .bind(&qr_code_url)
        .bind(&input.notes)
        .bind(&input.notes_th)
        .fetch_one(&mut *tx)
        .await?;

        // Add source references
        for source in &input.sources {
            sqlx::query(
                r#"
                INSERT INTO lot_sources (lot_id, source_lot_id, proportion_percent)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(lot_id)
            .bind(source.source_lot_id)
            .bind(source.proportion_percent)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        // Return the created lot with sources
        self.get_lot_with_sources(business_id, lot_id).await
    }

    /// Update a lot
    pub async fn update_lot(
        &self,
        business_id: Uuid,
        lot_id: Uuid,
        input: UpdateLotInput,
    ) -> AppResult<Lot> {
        // Check if lot exists
        let existing = sqlx::query_as::<_, (String, String, Decimal, Option<String>, Option<String>)>(
            "SELECT name, stage, current_weight_kg, notes, notes_th FROM lots WHERE id = $1 AND business_id = $2"
        )
        .bind(lot_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Lot".to_string()))?;

        // Validate stage if provided
        if let Some(ref stage) = input.stage {
            if LotStage::from_str(stage).is_none() {
                return Err(AppError::Validation {
                    field: "stage".to_string(),
                    message: "Invalid lot stage".to_string(),
                    message_th: "สถานะล็อตไม่ถูกต้อง".to_string(),
                });
            }
        }

        // Update lot
        let name = input.name.unwrap_or(existing.0);
        let stage = input.stage.unwrap_or(existing.1);
        let current_weight_kg = input.current_weight_kg.unwrap_or(existing.2);
        let notes = input.notes.or(existing.3);
        let notes_th = input.notes_th.or(existing.4);

        let row = sqlx::query_as::<_, (Uuid, Uuid, String, String, String, Decimal, Option<String>, Option<String>, Option<String>, DateTime<Utc>, DateTime<Utc>)>(
            r#"
            UPDATE lots
            SET name = $1, stage = $2, current_weight_kg = $3, notes = $4, notes_th = $5
            WHERE id = $6
            RETURNING id, business_id, traceability_code, name, stage, current_weight_kg,
                      qr_code_url, notes, notes_th, created_at, updated_at
            "#,
        )
        .bind(&name)
        .bind(&stage)
        .bind(current_weight_kg)
        .bind(&notes)
        .bind(&notes_th)
        .bind(lot_id)
        .fetch_one(&self.db)
        .await?;

        Ok(Lot {
            id: row.0,
            business_id: row.1,
            traceability_code: row.2,
            name: row.3,
            stage: row.4,
            current_weight_kg: row.5,
            qr_code_url: row.6,
            notes: row.7,
            notes_th: row.8,
            created_at: row.9,
            updated_at: row.10,
        })
    }

    /// Get lot by traceability code (public access for QR code)
    pub async fn get_lot_by_code(&self, traceability_code: &str) -> AppResult<Lot> {
        let row = sqlx::query_as::<_, (Uuid, Uuid, String, String, String, Decimal, Option<String>, Option<String>, Option<String>, DateTime<Utc>, DateTime<Utc>)>(
            r#"
            SELECT id, business_id, traceability_code, name, stage, current_weight_kg,
                   qr_code_url, notes, notes_th, created_at, updated_at
            FROM lots
            WHERE traceability_code = $1
            "#,
        )
        .bind(traceability_code)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Lot".to_string()))?;

        Ok(Lot {
            id: row.0,
            business_id: row.1,
            traceability_code: row.2,
            name: row.3,
            stage: row.4,
            current_weight_kg: row.5,
            qr_code_url: row.6,
            notes: row.7,
            notes_th: row.8,
            created_at: row.9,
            updated_at: row.10,
        })
    }
}
