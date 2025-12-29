//! Cupping session and score management service
//!
//! Implements SCA cupping protocol with 10 attributes.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Cupping service for managing cupping sessions and scores
#[derive(Clone)]
pub struct CuppingService {
    db: PgPool,
}

/// Database row for cupping session
#[derive(Debug, sqlx::FromRow)]
struct CuppingSessionRow {
    id: Uuid,
    business_id: Uuid,
    session_date: NaiveDate,
    cupper_name: String,
    location: Option<String>,
    notes: Option<String>,
    notes_th: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Database row for cupping sample
#[derive(Debug, sqlx::FromRow)]
struct CuppingSampleRow {
    id: Uuid,
    session_id: Uuid,
    lot_id: Uuid,
    sample_number: i32,
    fragrance_aroma: Decimal,
    flavor: Decimal,
    aftertaste: Decimal,
    acidity: Decimal,
    body: Decimal,
    balance: Decimal,
    uniformity: Decimal,
    clean_cup: Decimal,
    sweetness: Decimal,
    overall: Decimal,
    total_score: Decimal,
    tasting_notes: Option<String>,
    tasting_notes_th: Option<String>,
    defects_taint: i32,
    defects_fault: i32,
    final_score: Decimal,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Cupping session
#[derive(Debug, Clone, Serialize)]
pub struct CuppingSession {
    pub id: Uuid,
    pub business_id: Uuid,
    pub session_date: NaiveDate,
    pub cupper_name: String,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
    pub samples: Vec<CuppingSample>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cupping sample (individual lot evaluation)
#[derive(Debug, Clone, Serialize)]
pub struct CuppingSample {
    pub id: Uuid,
    pub session_id: Uuid,
    pub lot_id: Uuid,
    pub sample_number: i32,
    pub scores: CuppingScores,
    pub total_score: Decimal,
    pub tasting_notes: Option<String>,
    pub tasting_notes_th: Option<String>,
    pub defects: CuppingDefects,
    pub final_score: Decimal,
    pub classification: CoffeeClassification,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// SCA Cupping Protocol Scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuppingScores {
    pub fragrance_aroma: Decimal,
    pub flavor: Decimal,
    pub aftertaste: Decimal,
    pub acidity: Decimal,
    pub body: Decimal,
    pub balance: Decimal,
    pub uniformity: Decimal,
    pub clean_cup: Decimal,
    pub sweetness: Decimal,
    pub overall: Decimal,
}

/// Cupping defects
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CuppingDefects {
    pub taint_count: i32,  // 2 points each
    pub fault_count: i32,  // 4 points each
}

impl CuppingDefects {
    pub fn total_deduction(&self) -> Decimal {
        Decimal::from(self.taint_count * 2 + self.fault_count * 4)
    }
}

/// Coffee classification based on cupping score
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoffeeClassification {
    Outstanding,      // 90+
    Excellent,        // 85-89.99
    VeryGood,         // 80-84.99
    BelowSpecialty,   // <80
}

impl std::fmt::Display for CoffeeClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoffeeClassification::Outstanding => write!(f, "Outstanding"),
            CoffeeClassification::Excellent => write!(f, "Excellent"),
            CoffeeClassification::VeryGood => write!(f, "Very Good"),
            CoffeeClassification::BelowSpecialty => write!(f, "Below Specialty"),
        }
    }
}

/// Input for creating a cupping session
#[derive(Debug, Deserialize)]
pub struct CreateCuppingSessionInput {
    pub session_date: NaiveDate,
    pub cupper_name: String,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
}

/// Input for adding a cupping sample
#[derive(Debug, Deserialize)]
pub struct AddCuppingSampleInput {
    pub lot_id: Uuid,
    pub scores: CuppingScores,
    pub tasting_notes: Option<String>,
    pub tasting_notes_th: Option<String>,
    pub defects: Option<CuppingDefects>,
}

/// Cupping trend data
#[derive(Debug, Serialize)]
pub struct CuppingTrend {
    pub lot_id: Uuid,
    pub samples: Vec<CuppingSample>,
    pub average_score: Decimal,
    pub score_trend: ScoreTrend,
}

/// Score trend analysis
#[derive(Debug, Serialize)]
pub struct ScoreTrend {
    pub improving: bool,
    pub latest_score: Decimal,
    pub previous_score: Option<Decimal>,
    pub change: Option<Decimal>,
}

impl CuppingService {
    /// Create a new CuppingService instance
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Create a new cupping session
    pub async fn create_session(
        &self,
        business_id: Uuid,
        input: CreateCuppingSessionInput,
    ) -> AppResult<CuppingSession> {
        // Validate input
        if input.cupper_name.trim().is_empty() {
            return Err(AppError::Validation {
                field: "cupper_name".to_string(),
                message: "Cupper name is required".to_string(),
                message_th: "ต้องระบุชื่อผู้ชิม".to_string(),
            });
        }

        let row = sqlx::query_as::<_, CuppingSessionRow>(
            r#"
            INSERT INTO cupping_sessions (business_id, session_date, cupper_name, location, notes, notes_th)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, business_id, session_date, cupper_name, location, notes, notes_th, created_at, updated_at
            "#,
        )
        .bind(business_id)
        .bind(input.session_date)
        .bind(&input.cupper_name)
        .bind(&input.location)
        .bind(&input.notes)
        .bind(&input.notes_th)
        .fetch_one(&self.db)
        .await?;

        Ok(CuppingSession {
            id: row.id,
            business_id: row.business_id,
            session_date: row.session_date,
            cupper_name: row.cupper_name,
            location: row.location,
            notes: row.notes,
            notes_th: row.notes_th,
            samples: vec![],
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    /// Add a sample to a cupping session
    pub async fn add_sample(
        &self,
        business_id: Uuid,
        session_id: Uuid,
        input: AddCuppingSampleInput,
    ) -> AppResult<CuppingSample> {
        // Validate session exists and belongs to business
        self.validate_session_access(business_id, session_id).await?;

        // Validate lot exists and belongs to business
        self.validate_lot_access(business_id, input.lot_id).await?;

        // Validate scores
        self.validate_scores(&input.scores)?;

        // Calculate total score
        let total_score = Self::calculate_total_score(&input.scores);

        // Get defects
        let defects = input.defects.unwrap_or_default();

        // Calculate final score
        let final_score = total_score - defects.total_deduction();

        // Get next sample number
        let sample_number = sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(MAX(sample_number), 0) + 1 FROM cupping_samples WHERE session_id = $1",
        )
        .bind(session_id)
        .fetch_one(&self.db)
        .await? as i32;

        let row = sqlx::query_as::<_, CuppingSampleRow>(
            r#"
            INSERT INTO cupping_samples (
                session_id, lot_id, sample_number,
                fragrance_aroma, flavor, aftertaste, acidity, body, balance,
                uniformity, clean_cup, sweetness, overall,
                total_score, tasting_notes, tasting_notes_th,
                defects_taint, defects_fault, final_score
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            RETURNING id, session_id, lot_id, sample_number,
                      fragrance_aroma, flavor, aftertaste, acidity, body, balance,
                      uniformity, clean_cup, sweetness, overall,
                      total_score, tasting_notes, tasting_notes_th,
                      defects_taint, defects_fault, final_score,
                      created_at, updated_at
            "#,
        )
        .bind(session_id)
        .bind(input.lot_id)
        .bind(sample_number)
        .bind(input.scores.fragrance_aroma)
        .bind(input.scores.flavor)
        .bind(input.scores.aftertaste)
        .bind(input.scores.acidity)
        .bind(input.scores.body)
        .bind(input.scores.balance)
        .bind(input.scores.uniformity)
        .bind(input.scores.clean_cup)
        .bind(input.scores.sweetness)
        .bind(input.scores.overall)
        .bind(total_score)
        .bind(&input.tasting_notes)
        .bind(&input.tasting_notes_th)
        .bind(defects.taint_count)
        .bind(defects.fault_count)
        .bind(final_score)
        .fetch_one(&self.db)
        .await?;

        Ok(self.row_to_sample(row))
    }

    /// Get a cupping session with all samples
    pub async fn get_session(
        &self,
        business_id: Uuid,
        session_id: Uuid,
    ) -> AppResult<CuppingSession> {
        let session_row = sqlx::query_as::<_, CuppingSessionRow>(
            r#"
            SELECT id, business_id, session_date, cupper_name, location, notes, notes_th, created_at, updated_at
            FROM cupping_sessions
            WHERE id = $1 AND business_id = $2
            "#,
        )
        .bind(session_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Cupping session".to_string()))?;

        let sample_rows = sqlx::query_as::<_, CuppingSampleRow>(
            r#"
            SELECT id, session_id, lot_id, sample_number,
                   fragrance_aroma, flavor, aftertaste, acidity, body, balance,
                   uniformity, clean_cup, sweetness, overall,
                   total_score, tasting_notes, tasting_notes_th,
                   defects_taint, defects_fault, final_score,
                   created_at, updated_at
            FROM cupping_samples
            WHERE session_id = $1
            ORDER BY sample_number
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.db)
        .await?;

        let samples: Vec<CuppingSample> = sample_rows
            .into_iter()
            .map(|r| self.row_to_sample(r))
            .collect();

        Ok(CuppingSession {
            id: session_row.id,
            business_id: session_row.business_id,
            session_date: session_row.session_date,
            cupper_name: session_row.cupper_name,
            location: session_row.location,
            notes: session_row.notes,
            notes_th: session_row.notes_th,
            samples,
            created_at: session_row.created_at,
            updated_at: session_row.updated_at,
        })
    }

    /// List all cupping sessions for a business
    pub async fn list_sessions(&self, business_id: Uuid) -> AppResult<Vec<CuppingSession>> {
        let session_rows = sqlx::query_as::<_, CuppingSessionRow>(
            r#"
            SELECT id, business_id, session_date, cupper_name, location, notes, notes_th, created_at, updated_at
            FROM cupping_sessions
            WHERE business_id = $1
            ORDER BY session_date DESC, created_at DESC
            "#,
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        let mut sessions = Vec::new();
        for row in session_rows {
            let sample_rows = sqlx::query_as::<_, CuppingSampleRow>(
                r#"
                SELECT id, session_id, lot_id, sample_number,
                       fragrance_aroma, flavor, aftertaste, acidity, body, balance,
                       uniformity, clean_cup, sweetness, overall,
                       total_score, tasting_notes, tasting_notes_th,
                       defects_taint, defects_fault, final_score,
                       created_at, updated_at
                FROM cupping_samples
                WHERE session_id = $1
                ORDER BY sample_number
                "#,
            )
            .bind(row.id)
            .fetch_all(&self.db)
            .await?;

            let samples: Vec<CuppingSample> = sample_rows
                .into_iter()
                .map(|r| self.row_to_sample(r))
                .collect();

            sessions.push(CuppingSession {
                id: row.id,
                business_id: row.business_id,
                session_date: row.session_date,
                cupper_name: row.cupper_name,
                location: row.location,
                notes: row.notes,
                notes_th: row.notes_th,
                samples,
                created_at: row.created_at,
                updated_at: row.updated_at,
            });
        }

        Ok(sessions)
    }

    /// Get cupping history for a lot
    pub async fn get_lot_cupping_history(
        &self,
        business_id: Uuid,
        lot_id: Uuid,
    ) -> AppResult<Vec<CuppingSample>> {
        let rows = sqlx::query_as::<_, CuppingSampleRow>(
            r#"
            SELECT cs.id, cs.session_id, cs.lot_id, cs.sample_number,
                   cs.fragrance_aroma, cs.flavor, cs.aftertaste, cs.acidity, cs.body, cs.balance,
                   cs.uniformity, cs.clean_cup, cs.sweetness, cs.overall,
                   cs.total_score, cs.tasting_notes, cs.tasting_notes_th,
                   cs.defects_taint, cs.defects_fault, cs.final_score,
                   cs.created_at, cs.updated_at
            FROM cupping_samples cs
            JOIN cupping_sessions s ON s.id = cs.session_id
            WHERE cs.lot_id = $1 AND s.business_id = $2
            ORDER BY s.session_date DESC, cs.created_at DESC
            "#,
        )
        .bind(lot_id)
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(|r| self.row_to_sample(r)).collect())
    }

    /// Get cupping trend for a lot
    pub async fn get_lot_cupping_trend(
        &self,
        business_id: Uuid,
        lot_id: Uuid,
    ) -> AppResult<CuppingTrend> {
        let samples = self.get_lot_cupping_history(business_id, lot_id).await?;

        if samples.is_empty() {
            return Err(AppError::NotFound("Cupping samples for lot".to_string()));
        }

        let total: Decimal = samples.iter().map(|s| s.final_score).sum();
        let average_score = total / Decimal::from(samples.len());

        let latest = &samples[0];
        let previous = samples.get(1);

        let score_trend = ScoreTrend {
            improving: previous
                .map(|p| latest.final_score > p.final_score)
                .unwrap_or(false),
            latest_score: latest.final_score,
            previous_score: previous.map(|p| p.final_score),
            change: previous.map(|p| latest.final_score - p.final_score),
        };

        Ok(CuppingTrend {
            lot_id,
            samples,
            average_score,
            score_trend,
        })
    }

    /// Calculate total cupping score from individual scores
    pub fn calculate_total_score(scores: &CuppingScores) -> Decimal {
        scores.fragrance_aroma
            + scores.flavor
            + scores.aftertaste
            + scores.acidity
            + scores.body
            + scores.balance
            + scores.uniformity
            + scores.clean_cup
            + scores.sweetness
            + scores.overall
    }

    /// Classify coffee based on final cupping score
    pub fn classify_by_score(score: Decimal) -> CoffeeClassification {
        if score >= Decimal::from(90) {
            CoffeeClassification::Outstanding
        } else if score >= Decimal::from(85) {
            CoffeeClassification::Excellent
        } else if score >= Decimal::from(80) {
            CoffeeClassification::VeryGood
        } else {
            CoffeeClassification::BelowSpecialty
        }
    }

    /// Validate cupping scores are within valid ranges
    fn validate_scores(&self, scores: &CuppingScores) -> AppResult<()> {
        let min = Decimal::from(0);
        let max_standard = Decimal::from(10);

        // Standard attributes (6.0-10.0 scale)
        let standard_scores = [
            ("fragrance_aroma", scores.fragrance_aroma),
            ("flavor", scores.flavor),
            ("aftertaste", scores.aftertaste),
            ("acidity", scores.acidity),
            ("body", scores.body),
            ("balance", scores.balance),
            ("overall", scores.overall),
        ];

        for (name, score) in standard_scores {
            if score < min || score > max_standard {
                return Err(AppError::Validation {
                    field: name.to_string(),
                    message: format!("{} must be between 0 and 10", name),
                    message_th: format!("{} ต้องอยู่ระหว่าง 0 ถึง 10", name),
                });
            }
        }

        // Cup-based attributes (0-10, 2 points per cup)
        let cup_scores = [
            ("uniformity", scores.uniformity),
            ("clean_cup", scores.clean_cup),
            ("sweetness", scores.sweetness),
        ];

        for (name, score) in cup_scores {
            if score < min || score > max_standard {
                return Err(AppError::Validation {
                    field: name.to_string(),
                    message: format!("{} must be between 0 and 10", name),
                    message_th: format!("{} ต้องอยู่ระหว่าง 0 ถึง 10", name),
                });
            }
        }

        Ok(())
    }

    /// Validate session access
    async fn validate_session_access(
        &self,
        business_id: Uuid,
        session_id: Uuid,
    ) -> AppResult<()> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM cupping_sessions WHERE id = $1 AND business_id = $2)",
        )
        .bind(session_id)
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        if !exists {
            return Err(AppError::NotFound("Cupping session".to_string()));
        }

        Ok(())
    }

    /// Validate lot access
    async fn validate_lot_access(&self, business_id: Uuid, lot_id: Uuid) -> AppResult<()> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM lots WHERE id = $1 AND business_id = $2)",
        )
        .bind(lot_id)
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        if !exists {
            return Err(AppError::NotFound("Lot".to_string()));
        }

        Ok(())
    }

    /// Convert database row to CuppingSample
    fn row_to_sample(&self, row: CuppingSampleRow) -> CuppingSample {
        let scores = CuppingScores {
            fragrance_aroma: row.fragrance_aroma,
            flavor: row.flavor,
            aftertaste: row.aftertaste,
            acidity: row.acidity,
            body: row.body,
            balance: row.balance,
            uniformity: row.uniformity,
            clean_cup: row.clean_cup,
            sweetness: row.sweetness,
            overall: row.overall,
        };

        let defects = CuppingDefects {
            taint_count: row.defects_taint,
            fault_count: row.defects_fault,
        };

        let classification = Self::classify_by_score(row.final_score);

        CuppingSample {
            id: row.id,
            session_id: row.session_id,
            lot_id: row.lot_id,
            sample_number: row.sample_number,
            scores,
            total_score: row.total_score,
            tasting_notes: row.tasting_notes,
            tasting_notes_th: row.tasting_notes_th,
            defects,
            final_score: row.final_score,
            classification,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
