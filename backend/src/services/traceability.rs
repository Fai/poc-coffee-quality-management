//! Lot traceability service for public QR code landing pages
//!
//! Aggregates all lot data: farm, harvest, processing, grading, cupping, certifications

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Traceability service for public lot information
#[derive(Clone)]
pub struct TraceabilityService {
    db: PgPool,
}

/// Complete traceability view for a lot
#[derive(Debug, Serialize)]
pub struct TraceabilityView {
    pub lot: LotInfo,
    pub business: BusinessInfo,
    pub origin: Option<OriginInfo>,
    pub harvests: Vec<HarvestInfo>,
    pub processing: Option<ProcessingInfo>,
    pub grading: Option<GradingInfo>,
    pub cupping: Option<CuppingInfo>,
    pub sources: Vec<SourceLotInfo>,
    pub certifications: Vec<CertificationInfo>,
}

/// Basic lot information
#[derive(Debug, Serialize)]
pub struct LotInfo {
    pub traceability_code: String,
    pub name: String,
    pub stage: String,
    pub current_weight_kg: Decimal,
    pub qr_code_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Business information (limited for public view)
#[derive(Debug, Serialize)]
pub struct BusinessInfo {
    pub name: String,
    pub business_type: String,
    pub province: Option<String>,
}

/// Origin/farm information
#[derive(Debug, Serialize)]
pub struct OriginInfo {
    pub plot_name: String,
    pub varieties: Vec<String>,
    pub altitude_meters: Option<i32>,
    pub province: Option<String>,
    pub district: Option<String>,
}

/// Harvest information
#[derive(Debug, Serialize)]
pub struct HarvestInfo {
    pub harvest_date: NaiveDate,
    pub cherry_weight_kg: Decimal,
    pub ripeness_ripe_percent: i32,
    pub picker_name: Option<String>,
}

/// Processing information
#[derive(Debug, Serialize)]
pub struct ProcessingInfo {
    pub method: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub fermentation_hours: Option<i32>,
    pub drying_method: Option<String>,
    pub drying_days: Option<i32>,
    pub final_moisture_percent: Option<Decimal>,
    pub green_bean_weight_kg: Option<Decimal>,
    pub yield_percent: Option<Decimal>,
}

/// Grading information
#[derive(Debug, Serialize)]
pub struct GradingInfo {
    pub grading_date: NaiveDate,
    pub grade: String,
    pub total_defects: i32,
    pub moisture_percent: Option<Decimal>,
    pub screen_size_distribution: Option<serde_json::Value>,
}

/// Cupping information
#[derive(Debug, Serialize)]
pub struct CuppingInfo {
    pub session_date: NaiveDate,
    pub cupper_name: String,
    pub final_score: Decimal,
    pub classification: String,
    pub tasting_notes: Option<String>,
    pub tasting_notes_th: Option<String>,
}

/// Source lot info for blended lots
#[derive(Debug, Serialize)]
pub struct SourceLotInfo {
    pub traceability_code: String,
    pub name: String,
    pub proportion_percent: Decimal,
}

/// Certification info for traceability view
#[derive(Debug, Serialize, FromRow)]
pub struct CertificationInfo {
    pub certification_type: String,
    pub certification_name: String,
    pub certifying_body: String,
    pub certificate_number: String,
    pub scope: String,
    pub valid_until: NaiveDate,
}

impl TraceabilityService {
    /// Create a new TraceabilityService instance
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Get complete traceability view for a lot by traceability code
    pub async fn get_traceability_view(
        &self,
        traceability_code: &str,
        _language: Option<&str>,
    ) -> AppResult<TraceabilityView> {
        // Get lot basic info
        let lot_row = sqlx::query_as::<_, (Uuid, Uuid, String, String, String, Decimal, Option<String>, DateTime<Utc>)>(
            r#"
            SELECT id, business_id, traceability_code, name, stage, current_weight_kg, qr_code_url, created_at
            FROM lots
            WHERE traceability_code = $1
            "#,
        )
        .bind(traceability_code)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Lot".to_string()))?;

        let lot_id = lot_row.0;
        let business_id = lot_row.1;

        let lot = LotInfo {
            traceability_code: lot_row.2,
            name: lot_row.3,
            stage: lot_row.4,
            current_weight_kg: lot_row.5,
            qr_code_url: lot_row.6,
            created_at: lot_row.7,
        };

        // Get business info
        let business = self.get_business_info(business_id).await?;

        // Get origin info from harvests
        let origin = self.get_origin_info(lot_id).await?;

        // Get harvests
        let harvests = self.get_harvests(lot_id).await?;

        // Get processing info
        let processing = self.get_processing_info(lot_id).await?;

        // Get grading info
        let grading = self.get_grading_info(lot_id).await?;

        // Get cupping info
        let cupping = self.get_cupping_info(lot_id).await?;

        // Get source lots (for blended lots)
        let sources = self.get_source_lots(lot_id).await?;

        // Get certifications for the lot (based on business and plot)
        let plot_id = self.get_plot_id_from_lot(lot_id).await?;
        let certifications = self.get_certifications(business_id, plot_id).await?;

        Ok(TraceabilityView {
            lot,
            business,
            origin,
            harvests,
            processing,
            grading,
            cupping,
            sources,
            certifications,
        })
    }

    async fn get_business_info(&self, business_id: Uuid) -> AppResult<BusinessInfo> {
        let row = sqlx::query_as::<_, (String, String, Option<String>)>(
            "SELECT name, business_type, province FROM businesses WHERE id = $1",
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        Ok(BusinessInfo {
            name: row.0,
            business_type: row.1,
            province: row.2,
        })
    }

    async fn get_origin_info(&self, lot_id: Uuid) -> AppResult<Option<OriginInfo>> {
        // Get plot info from first harvest
        let plot_row = sqlx::query_as::<_, (String, Option<i32>, Option<String>, Option<String>)>(
            r#"
            SELECT p.name, p.altitude_meters, p.province, p.district
            FROM harvests h
            JOIN plots p ON p.id = h.plot_id
            WHERE h.lot_id = $1
            LIMIT 1
            "#,
        )
        .bind(lot_id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(row) = plot_row {
            // Get varieties for the plot
            let varieties = sqlx::query_scalar::<_, String>(
                r#"
                SELECT pv.variety_name
                FROM harvests h
                JOIN plot_varieties pv ON pv.plot_id = h.plot_id
                WHERE h.lot_id = $1
                "#,
            )
            .bind(lot_id)
            .fetch_all(&self.db)
            .await
            .unwrap_or_default();

            Ok(Some(OriginInfo {
                plot_name: row.0,
                varieties,
                altitude_meters: row.1,
                province: row.2,
                district: row.3,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_harvests(&self, lot_id: Uuid) -> AppResult<Vec<HarvestInfo>> {
        let rows = sqlx::query_as::<_, (NaiveDate, Decimal, i32, Option<String>)>(
            r#"
            SELECT harvest_date, cherry_weight_kg, ripeness_ripe_percent, picker_name
            FROM harvests
            WHERE lot_id = $1
            ORDER BY harvest_date
            "#,
        )
        .bind(lot_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| HarvestInfo {
                harvest_date: r.0,
                cherry_weight_kg: r.1,
                ripeness_ripe_percent: r.2,
                picker_name: r.3,
            })
            .collect())
    }

    async fn get_processing_info(&self, lot_id: Uuid) -> AppResult<Option<ProcessingInfo>> {
        let row = sqlx::query_as::<_, (String, NaiveDate, Option<NaiveDate>, Option<serde_json::Value>, Option<serde_json::Value>, Option<Decimal>, Option<Decimal>, Option<Decimal>)>(
            r#"
            SELECT method, start_date, end_date, fermentation_log, drying_log,
                   final_moisture_percent, green_bean_weight_kg, yield_percent
            FROM processing_records
            WHERE lot_id = $1
            ORDER BY start_date DESC
            LIMIT 1
            "#,
        )
        .bind(lot_id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(r) = row {
            // Extract fermentation hours from log
            let fermentation_hours = r.3.as_ref().and_then(|log| {
                log.get("duration_hours").and_then(|v| v.as_i64()).map(|v| v as i32)
            });

            // Extract drying info from log
            let (drying_method, drying_days) = if let Some(log) = &r.4 {
                let method = log.get("method").and_then(|v| v.as_str()).map(String::from);
                let days = log.get("duration_days").and_then(|v| v.as_i64()).map(|v| v as i32);
                (method, days)
            } else {
                (None, None)
            };

            Ok(Some(ProcessingInfo {
                method: r.0,
                start_date: r.1,
                end_date: r.2,
                fermentation_hours,
                drying_method,
                drying_days,
                final_moisture_percent: r.5,
                green_bean_weight_kg: r.6,
                yield_percent: r.7,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_grading_info(&self, lot_id: Uuid) -> AppResult<Option<GradingInfo>> {
        let row = sqlx::query_as::<_, (NaiveDate, String, i32, Option<Decimal>, Option<serde_json::Value>)>(
            r#"
            SELECT grading_date, grade, 
                   (category1_full_black + category1_full_sour + category1_dried_cherry + 
                    category1_fungus + category1_foreign_matter + category1_severe_insect +
                    category2_partial_black + category2_partial_sour + category2_parchment +
                    category2_floater + category2_immature + category2_withered +
                    category2_shell + category2_broken + category2_hull + category2_husk) as total_defects,
                   moisture_percent, screen_size_distribution
            FROM green_bean_grades
            WHERE lot_id = $1
            ORDER BY grading_date DESC
            LIMIT 1
            "#,
        )
        .bind(lot_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|r| GradingInfo {
            grading_date: r.0,
            grade: r.1,
            total_defects: r.2,
            moisture_percent: r.3,
            screen_size_distribution: r.4,
        }))
    }

    async fn get_cupping_info(&self, lot_id: Uuid) -> AppResult<Option<CuppingInfo>> {
        let row = sqlx::query_as::<_, (NaiveDate, String, Decimal, Option<String>, Option<String>)>(
            r#"
            SELECT s.session_date, s.cupper_name, cs.final_score, cs.tasting_notes, cs.tasting_notes_th
            FROM cupping_samples cs
            JOIN cupping_sessions s ON s.id = cs.session_id
            WHERE cs.lot_id = $1
            ORDER BY s.session_date DESC
            LIMIT 1
            "#,
        )
        .bind(lot_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|r| {
            let classification = if r.2 >= Decimal::from(90) {
                "Outstanding"
            } else if r.2 >= Decimal::from(85) {
                "Excellent"
            } else if r.2 >= Decimal::from(80) {
                "Very Good"
            } else {
                "Below Specialty"
            };

            CuppingInfo {
                session_date: r.0,
                cupper_name: r.1,
                final_score: r.2,
                classification: classification.to_string(),
                tasting_notes: r.3,
                tasting_notes_th: r.4,
            }
        }))
    }

    async fn get_source_lots(&self, lot_id: Uuid) -> AppResult<Vec<SourceLotInfo>> {
        let rows = sqlx::query_as::<_, (String, String, Decimal)>(
            r#"
            SELECT l.traceability_code, l.name, ls.proportion_percent
            FROM lot_sources ls
            JOIN lots l ON l.id = ls.source_lot_id
            WHERE ls.lot_id = $1
            ORDER BY ls.proportion_percent DESC
            "#,
        )
        .bind(lot_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| SourceLotInfo {
                traceability_code: r.0,
                name: r.1,
                proportion_percent: r.2,
            })
            .collect())
    }

    /// Generate QR code URL for a lot
    pub fn generate_qr_code_url(traceability_code: &str, base_url: &str) -> String {
        format!("{}/trace/{}", base_url, traceability_code)
    }

    /// Get plot ID from lot's first harvest
    async fn get_plot_id_from_lot(&self, lot_id: Uuid) -> AppResult<Option<Uuid>> {
        let plot_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT plot_id FROM harvests WHERE lot_id = $1 LIMIT 1",
        )
        .bind(lot_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(plot_id)
    }

    /// Get active certifications for traceability view
    async fn get_certifications(
        &self,
        business_id: Uuid,
        plot_id: Option<Uuid>,
    ) -> AppResult<Vec<CertificationInfo>> {
        let today = Utc::now().date_naive();

        let certifications = sqlx::query_as::<_, CertificationInfo>(
            r#"
            SELECT 
                certification_type::TEXT as certification_type,
                certification_name,
                certification_body as certifying_body,
                certificate_number,
                scope::TEXT as scope,
                expiration_date as valid_until
            FROM certifications
            WHERE business_id = $1
              AND is_active = true
              AND expiration_date >= $2
              AND (
                  scope = 'business'
                  OR scope = 'farm'
                  OR (scope = 'plot' AND plot_id = $3)
                  OR scope = 'facility'
              )
            ORDER BY certification_type ASC
            "#,
        )
        .bind(business_id)
        .bind(today)
        .bind(plot_id)
        .fetch_all(&self.db)
        .await?;

        Ok(certifications)
    }
}
