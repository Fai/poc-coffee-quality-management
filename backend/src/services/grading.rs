//! Green bean grading service following SCA standards

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::services::lot::LotStage;
use shared::{
    classify_grade, AiDefectDetection, DefectBreakdown, DefectCount, GradeClassification,
    ScreenSizeDistribution,
};

/// Grading service for managing green bean quality grades
#[derive(Clone)]
pub struct GradingService {
    db: PgPool,
}

/// Database row for grading record
#[derive(Debug, sqlx::FromRow)]
struct GradingRow {
    id: Uuid,
    lot_id: Uuid,
    grading_date: NaiveDate,
    grader_name: String,
    sample_weight_grams: Decimal,
    category1_count: i32,
    category2_count: i32,
    defect_breakdown: Option<serde_json::Value>,
    ai_detection: Option<serde_json::Value>,
    moisture_percent: Decimal,
    density: Option<Decimal>,
    screen_size_distribution: Option<serde_json::Value>,
    grade: String,
    notes: Option<String>,
    notes_th: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<GradingRow> for GradingRecord {
    fn from(row: GradingRow) -> Self {
        let defect_breakdown: Option<DefectBreakdown> = row
            .defect_breakdown
            .and_then(|v| serde_json::from_value(v).ok());

        let ai_detection: Option<AiDefectDetection> =
            row.ai_detection.and_then(|v| serde_json::from_value(v).ok());

        let screen_size: Option<ScreenSizeDistribution> = row
            .screen_size_distribution
            .and_then(|v| serde_json::from_value(v).ok());

        GradingRecord {
            id: row.id,
            lot_id: row.lot_id,
            grading_date: row.grading_date,
            grader_name: row.grader_name,
            sample_weight_grams: row.sample_weight_grams,
            defects: DefectCount {
                category1_count: row.category1_count,
                category2_count: row.category2_count,
                defect_breakdown,
            },
            ai_detection,
            moisture_percent: row.moisture_percent,
            density: row.density,
            screen_size,
            grade: grade_from_str(&row.grade),
            notes: row.notes,
            notes_th: row.notes_th,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Grading record
#[derive(Debug, Clone, Serialize)]
pub struct GradingRecord {
    pub id: Uuid,
    pub lot_id: Uuid,
    pub grading_date: NaiveDate,
    pub grader_name: String,
    pub sample_weight_grams: Decimal,
    pub defects: DefectCount,
    pub ai_detection: Option<AiDefectDetection>,
    pub moisture_percent: Decimal,
    pub density: Option<Decimal>,
    pub screen_size: Option<ScreenSizeDistribution>,
    pub grade: GradeClassification,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for recording a grading
#[derive(Debug, Deserialize)]
pub struct RecordGradingInput {
    pub lot_id: Uuid,
    pub grading_date: NaiveDate,
    pub grader_name: String,
    pub sample_weight_grams: Decimal,
    pub category1_count: i32,
    pub category2_count: i32,
    pub defect_breakdown: Option<DefectBreakdown>,
    pub moisture_percent: Decimal,
    pub density: Option<Decimal>,
    pub screen_size: Option<ScreenSizeDistribution>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
}

/// Input for recording grading with AI detection
#[derive(Debug, Deserialize)]
pub struct RecordGradingWithAiInput {
    pub lot_id: Uuid,
    pub grading_date: NaiveDate,
    pub grader_name: String,
    pub sample_weight_grams: Decimal,
    pub ai_detection: AiDefectDetection,
    pub moisture_percent: Decimal,
    pub density: Option<Decimal>,
    pub screen_size: Option<ScreenSizeDistribution>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
}

/// Grading comparison result
#[derive(Debug, Serialize)]
pub struct GradingComparison {
    pub lot_id: Uuid,
    pub gradings: Vec<GradingRecord>,
    pub grade_trend: GradeTrend,
    pub defect_trend: DefectTrend,
}

/// Grade trend analysis
#[derive(Debug, Serialize)]
pub struct GradeTrend {
    pub improving: bool,
    pub latest_grade: GradeClassification,
    pub previous_grade: Option<GradeClassification>,
}

/// Defect trend analysis
#[derive(Debug, Serialize)]
pub struct DefectTrend {
    pub category1_change: i32,
    pub category2_change: i32,
    pub total_change: i32,
}

impl GradingService {
    /// Create a new GradingService instance
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Record a green bean grading (manual entry)
    pub async fn record_grading(
        &self,
        business_id: Uuid,
        input: RecordGradingInput,
    ) -> AppResult<GradingRecord> {
        // Validate lot exists and belongs to business
        self.validate_lot_for_grading(business_id, input.lot_id)
            .await?;

        // Validate input
        self.validate_grading_input(
            &input.grader_name,
            input.sample_weight_grams,
            input.category1_count,
            input.category2_count,
            input.moisture_percent,
        )?;

        // Calculate grade classification
        let defects = DefectCount {
            category1_count: input.category1_count,
            category2_count: input.category2_count,
            defect_breakdown: input.defect_breakdown.clone(),
        };
        let grade = classify_grade(&defects);

        // Serialize optional fields
        let defect_breakdown_json = input
            .defect_breakdown
            .as_ref()
            .map(|d| serde_json::to_value(d))
            .transpose()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let screen_size_json = input
            .screen_size
            .as_ref()
            .map(|s| serde_json::to_value(s))
            .transpose()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Insert grading record
        let row = sqlx::query_as::<_, GradingRow>(
            r#"
            INSERT INTO green_bean_grades (
                lot_id, grading_date, grader_name, sample_weight_grams,
                category1_count, category2_count, defect_breakdown,
                moisture_percent, density, screen_size_distribution, grade,
                notes, notes_th
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, lot_id, grading_date, grader_name, sample_weight_grams,
                      category1_count, category2_count, defect_breakdown, ai_detection,
                      moisture_percent, density, screen_size_distribution, grade,
                      notes, notes_th, created_at, updated_at
            "#,
        )
        .bind(input.lot_id)
        .bind(input.grading_date)
        .bind(&input.grader_name)
        .bind(input.sample_weight_grams)
        .bind(input.category1_count)
        .bind(input.category2_count)
        .bind(&defect_breakdown_json)
        .bind(input.moisture_percent)
        .bind(input.density)
        .bind(&screen_size_json)
        .bind(grade_to_str(&grade))
        .bind(&input.notes)
        .bind(&input.notes_th)
        .fetch_one(&self.db)
        .await?;

        Ok(row.into())
    }

    /// Record grading with AI-assisted defect detection
    pub async fn record_grading_with_ai(
        &self,
        business_id: Uuid,
        input: RecordGradingWithAiInput,
    ) -> AppResult<GradingRecord> {
        // Validate lot exists and belongs to business
        self.validate_lot_for_grading(business_id, input.lot_id)
            .await?;

        // Validate input
        self.validate_grading_input(
            &input.grader_name,
            input.sample_weight_grams,
            input.ai_detection.category1_count,
            input.ai_detection.category2_count,
            input.moisture_percent,
        )?;

        // Use AI detection counts for grade classification
        let defects = DefectCount {
            category1_count: input.ai_detection.category1_count,
            category2_count: input.ai_detection.category2_count,
            defect_breakdown: Some(input.ai_detection.defect_breakdown.clone()),
        };
        let grade = classify_grade(&defects);

        // Serialize fields
        let ai_detection_json = serde_json::to_value(&input.ai_detection)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let defect_breakdown_json = serde_json::to_value(&input.ai_detection.defect_breakdown)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let screen_size_json = input
            .screen_size
            .as_ref()
            .map(|s| serde_json::to_value(s))
            .transpose()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // Insert grading record
        let row = sqlx::query_as::<_, GradingRow>(
            r#"
            INSERT INTO green_bean_grades (
                lot_id, grading_date, grader_name, sample_weight_grams,
                category1_count, category2_count, defect_breakdown, ai_detection,
                moisture_percent, density, screen_size_distribution, grade,
                notes, notes_th
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id, lot_id, grading_date, grader_name, sample_weight_grams,
                      category1_count, category2_count, defect_breakdown, ai_detection,
                      moisture_percent, density, screen_size_distribution, grade,
                      notes, notes_th, created_at, updated_at
            "#,
        )
        .bind(input.lot_id)
        .bind(input.grading_date)
        .bind(&input.grader_name)
        .bind(input.sample_weight_grams)
        .bind(input.ai_detection.category1_count)
        .bind(input.ai_detection.category2_count)
        .bind(&defect_breakdown_json)
        .bind(&ai_detection_json)
        .bind(input.moisture_percent)
        .bind(input.density)
        .bind(&screen_size_json)
        .bind(grade_to_str(&grade))
        .bind(&input.notes)
        .bind(&input.notes_th)
        .fetch_one(&self.db)
        .await?;

        Ok(row.into())
    }

    /// Get grading record by ID
    pub async fn get_grading(
        &self,
        business_id: Uuid,
        grading_id: Uuid,
    ) -> AppResult<GradingRecord> {
        let row = sqlx::query_as::<_, GradingRow>(
            r#"
            SELECT g.id, g.lot_id, g.grading_date, g.grader_name, g.sample_weight_grams,
                   g.category1_count, g.category2_count, g.defect_breakdown, g.ai_detection,
                   g.moisture_percent, g.density, g.screen_size_distribution, g.grade,
                   g.notes, g.notes_th, g.created_at, g.updated_at
            FROM green_bean_grades g
            JOIN lots l ON l.id = g.lot_id
            WHERE g.id = $1 AND l.business_id = $2
            "#,
        )
        .bind(grading_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Grading record".to_string()))?;

        Ok(row.into())
    }

    /// Get grading history for a lot
    pub async fn get_grading_history(
        &self,
        business_id: Uuid,
        lot_id: Uuid,
    ) -> AppResult<Vec<GradingRecord>> {
        let rows = sqlx::query_as::<_, GradingRow>(
            r#"
            SELECT g.id, g.lot_id, g.grading_date, g.grader_name, g.sample_weight_grams,
                   g.category1_count, g.category2_count, g.defect_breakdown, g.ai_detection,
                   g.moisture_percent, g.density, g.screen_size_distribution, g.grade,
                   g.notes, g.notes_th, g.created_at, g.updated_at
            FROM green_bean_grades g
            JOIN lots l ON l.id = g.lot_id
            WHERE g.lot_id = $1 AND l.business_id = $2
            ORDER BY g.grading_date DESC, g.created_at DESC
            "#,
        )
        .bind(lot_id)
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// List all grading records for a business
    pub async fn list_gradings(&self, business_id: Uuid) -> AppResult<Vec<GradingRecord>> {
        let rows = sqlx::query_as::<_, GradingRow>(
            r#"
            SELECT g.id, g.lot_id, g.grading_date, g.grader_name, g.sample_weight_grams,
                   g.category1_count, g.category2_count, g.defect_breakdown, g.ai_detection,
                   g.moisture_percent, g.density, g.screen_size_distribution, g.grade,
                   g.notes, g.notes_th, g.created_at, g.updated_at
            FROM green_bean_grades g
            JOIN lots l ON l.id = g.lot_id
            WHERE l.business_id = $1
            ORDER BY g.grading_date DESC, g.created_at DESC
            "#,
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Get grading comparison for a lot
    pub async fn get_grading_comparison(
        &self,
        business_id: Uuid,
        lot_id: Uuid,
    ) -> AppResult<GradingComparison> {
        let gradings = self.get_grading_history(business_id, lot_id).await?;

        if gradings.is_empty() {
            return Err(AppError::NotFound("Grading records for lot".to_string()));
        }

        let latest = &gradings[0];
        let previous = gradings.get(1);

        let grade_trend = GradeTrend {
            improving: previous
                .map(|p| grade_rank(&latest.grade) > grade_rank(&p.grade))
                .unwrap_or(false),
            latest_grade: latest.grade.clone(),
            previous_grade: previous.map(|p| p.grade.clone()),
        };

        let defect_trend = if let Some(prev) = previous {
            DefectTrend {
                category1_change: latest.defects.category1_count - prev.defects.category1_count,
                category2_change: latest.defects.category2_count - prev.defects.category2_count,
                total_change: latest.defects.total() - prev.defects.total(),
            }
        } else {
            DefectTrend {
                category1_change: 0,
                category2_change: 0,
                total_change: 0,
            }
        };

        Ok(GradingComparison {
            lot_id,
            gradings,
            grade_trend,
            defect_trend,
        })
    }

    /// Validate lot exists and is in appropriate stage for grading
    async fn validate_lot_for_grading(
        &self,
        business_id: Uuid,
        lot_id: Uuid,
    ) -> AppResult<()> {
        let lot = sqlx::query_as::<_, (Uuid, String)>(
            "SELECT id, stage FROM lots WHERE id = $1 AND business_id = $2",
        )
        .bind(lot_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Lot".to_string()))?;

        // Grading is typically done on green beans
        if lot.1 != LotStage::GreenBean.as_str() {
            return Err(AppError::Validation {
                field: "lot_id".to_string(),
                message: format!(
                    "Lot should be in GreenBean stage for grading, current stage: {}",
                    lot.1
                ),
                message_th: format!(
                    "ล็อตควรอยู่ในสถานะกาแฟกะลาเพื่อการเกรด สถานะปัจจุบัน: {}",
                    lot.1
                ),
            });
        }

        Ok(())
    }

    /// Validate grading input fields
    fn validate_grading_input(
        &self,
        grader_name: &str,
        sample_weight: Decimal,
        category1: i32,
        category2: i32,
        moisture: Decimal,
    ) -> AppResult<()> {
        if grader_name.trim().is_empty() {
            return Err(AppError::Validation {
                field: "grader_name".to_string(),
                message: "Grader name is required".to_string(),
                message_th: "ต้องระบุชื่อผู้เกรด".to_string(),
            });
        }

        if sample_weight <= Decimal::ZERO {
            return Err(AppError::Validation {
                field: "sample_weight_grams".to_string(),
                message: "Sample weight must be positive".to_string(),
                message_th: "น้ำหนักตัวอย่างต้องเป็นค่าบวก".to_string(),
            });
        }

        if category1 < 0 {
            return Err(AppError::Validation {
                field: "category1_count".to_string(),
                message: "Category 1 defect count cannot be negative".to_string(),
                message_th: "จำนวนข้อบกพร่องประเภท 1 ต้องไม่ติดลบ".to_string(),
            });
        }

        if category2 < 0 {
            return Err(AppError::Validation {
                field: "category2_count".to_string(),
                message: "Category 2 defect count cannot be negative".to_string(),
                message_th: "จำนวนข้อบกพร่องประเภท 2 ต้องไม่ติดลบ".to_string(),
            });
        }

        if moisture < Decimal::ZERO || moisture > Decimal::from(100) {
            return Err(AppError::Validation {
                field: "moisture_percent".to_string(),
                message: "Moisture must be between 0 and 100%".to_string(),
                message_th: "ความชื้นต้องอยู่ระหว่าง 0 ถึง 100%".to_string(),
            });
        }

        Ok(())
    }
}

/// Convert GradeClassification to database string
fn grade_to_str(grade: &GradeClassification) -> &'static str {
    match grade {
        GradeClassification::SpecialtyGrade => "specialty_grade",
        GradeClassification::PremiumGrade => "premium_grade",
        GradeClassification::ExchangeGrade => "exchange_grade",
        GradeClassification::BelowStandard => "below_standard",
        GradeClassification::OffGrade => "off_grade",
    }
}

/// Convert database string to GradeClassification
fn grade_from_str(s: &str) -> GradeClassification {
    match s {
        "specialty_grade" => GradeClassification::SpecialtyGrade,
        "premium_grade" => GradeClassification::PremiumGrade,
        "exchange_grade" => GradeClassification::ExchangeGrade,
        "below_standard" => GradeClassification::BelowStandard,
        _ => GradeClassification::OffGrade,
    }
}

/// Get numeric rank for grade comparison (higher is better)
fn grade_rank(grade: &GradeClassification) -> i32 {
    match grade {
        GradeClassification::SpecialtyGrade => 5,
        GradeClassification::PremiumGrade => 4,
        GradeClassification::ExchangeGrade => 3,
        GradeClassification::BelowStandard => 2,
        GradeClassification::OffGrade => 1,
    }
}
