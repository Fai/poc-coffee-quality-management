//! Reporting service for analytics and data export
//! Provides harvest yield, quality trends, and processing efficiency reports

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;

/// Reporting service
#[derive(Clone)]
pub struct ReportingService {
    db: PgPool,
}

/// Harvest yield report entry
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct HarvestYieldReport {
    pub plot_id: Uuid,
    pub plot_name: String,
    pub variety: Option<String>,
    pub total_cherry_kg: Decimal,
    pub area_rai: Decimal,
    pub yield_kg_per_rai: Decimal,
    pub harvest_count: i64,
}

/// Quality trend data point
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct QualityTrendPoint {
    pub period: String,
    pub avg_cupping_score: Option<Decimal>,
    pub avg_defect_count: Option<Decimal>,
    pub sample_count: i64,
}

/// Processing efficiency report
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ProcessingEfficiencyReport {
    pub method: String,
    pub batch_count: i64,
    pub avg_yield_percent: Option<Decimal>,
    pub avg_processing_days: Option<Decimal>,
    pub total_cherry_kg: Decimal,
    pub total_green_bean_kg: Decimal,
}

/// Dashboard metrics
#[derive(Debug, Serialize)]
pub struct DashboardMetrics {
    pub total_lots: i64,
    pub active_lots: i64,
    pub total_inventory_kg: Decimal,
    pub avg_cupping_score: Option<Decimal>,
    pub pending_alerts: i64,
    pub recent_harvests: i64,
    pub expiring_certifications: i64,
}

/// Report filter parameters
#[derive(Debug, Deserialize)]
pub struct ReportFilter {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub plot_ids: Option<Vec<Uuid>>,
    pub varieties: Option<Vec<String>>,
    pub processing_methods: Option<Vec<String>>,
}

impl ReportingService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Get harvest yield report by plot
    pub async fn get_harvest_yield_report(
        &self,
        business_id: Uuid,
        filter: &ReportFilter,
    ) -> AppResult<Vec<HarvestYieldReport>> {
        let start = filter.start_date.unwrap_or(NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
        let end = filter.end_date.unwrap_or(NaiveDate::from_ymd_opt(2100, 12, 31).unwrap());

        let reports = sqlx::query_as::<_, HarvestYieldReport>(
            r#"
            SELECT 
                p.id as plot_id,
                p.name as plot_name,
                p.varieties->0->>'variety' as variety,
                COALESCE(SUM(h.cherry_weight_kg), 0) as total_cherry_kg,
                p.area_rai,
                CASE WHEN p.area_rai > 0 
                    THEN COALESCE(SUM(h.cherry_weight_kg), 0) / p.area_rai 
                    ELSE 0 
                END as yield_kg_per_rai,
                COUNT(h.id) as harvest_count
            FROM plots p
            LEFT JOIN harvests h ON h.plot_id = p.id 
                AND h.harvest_date BETWEEN $2 AND $3
            WHERE p.business_id = $1
            GROUP BY p.id, p.name, p.varieties, p.area_rai
            ORDER BY yield_kg_per_rai DESC
            "#,
        )
        .bind(business_id)
        .bind(start)
        .bind(end)
        .fetch_all(&self.db)
        .await?;

        Ok(reports)
    }

    /// Get quality trend report (cupping scores over time)
    pub async fn get_quality_trend_report(
        &self,
        business_id: Uuid,
        filter: &ReportFilter,
        group_by: &str, // "month", "quarter", "year"
    ) -> AppResult<Vec<QualityTrendPoint>> {
        let start = filter.start_date.unwrap_or(NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
        let end = filter.end_date.unwrap_or(NaiveDate::from_ymd_opt(2100, 12, 31).unwrap());

        let date_trunc = match group_by {
            "quarter" => "quarter",
            "year" => "year",
            _ => "month",
        };

        let query = format!(
            r#"
            SELECT 
                TO_CHAR(DATE_TRUNC('{}', cs.session_date), 'YYYY-MM') as period,
                AVG(csamp.total_score) as avg_cupping_score,
                AVG(g.category1_defects + g.category2_defects) as avg_defect_count,
                COUNT(DISTINCT csamp.id) as sample_count
            FROM cupping_sessions cs
            JOIN cupping_samples csamp ON csamp.session_id = cs.id
            LEFT JOIN lots l ON l.id = csamp.lot_id
            LEFT JOIN green_bean_grades g ON g.lot_id = l.id
            WHERE cs.business_id = $1
              AND cs.session_date BETWEEN $2 AND $3
            GROUP BY DATE_TRUNC('{}', cs.session_date)
            ORDER BY period ASC
            "#,
            date_trunc, date_trunc
        );

        let trends = sqlx::query_as::<_, QualityTrendPoint>(&query)
            .bind(business_id)
            .bind(start)
            .bind(end)
            .fetch_all(&self.db)
            .await?;

        Ok(trends)
    }

    /// Get processing efficiency report
    pub async fn get_processing_efficiency_report(
        &self,
        business_id: Uuid,
        filter: &ReportFilter,
    ) -> AppResult<Vec<ProcessingEfficiencyReport>> {
        let start = filter.start_date.unwrap_or(NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
        let end = filter.end_date.unwrap_or(NaiveDate::from_ymd_opt(2100, 12, 31).unwrap());

        let reports = sqlx::query_as::<_, ProcessingEfficiencyReport>(
            r#"
            SELECT 
                pr.method,
                COUNT(*) as batch_count,
                AVG(
                    CASE WHEN l.current_weight_kg > 0 AND h_agg.total_cherry > 0
                        THEN (pr.green_bean_weight_kg / h_agg.total_cherry) * 100
                        ELSE NULL
                    END
                ) as avg_yield_percent,
                AVG(pr.end_date - pr.start_date) as avg_processing_days,
                COALESCE(SUM(h_agg.total_cherry), 0) as total_cherry_kg,
                COALESCE(SUM(pr.green_bean_weight_kg), 0) as total_green_bean_kg
            FROM processing_records pr
            JOIN lots l ON l.id = pr.lot_id
            JOIN (
                SELECT lot_id, SUM(cherry_weight_kg) as total_cherry
                FROM harvests
                GROUP BY lot_id
            ) h_agg ON h_agg.lot_id = l.id
            WHERE l.business_id = $1
              AND pr.start_date BETWEEN $2 AND $3
              AND pr.end_date IS NOT NULL
            GROUP BY pr.method
            ORDER BY batch_count DESC
            "#,
        )
        .bind(business_id)
        .bind(start)
        .bind(end)
        .fetch_all(&self.db)
        .await?;

        Ok(reports)
    }

    /// Get dashboard metrics
    pub async fn get_dashboard_metrics(&self, business_id: Uuid) -> AppResult<DashboardMetrics> {
        // Total and active lots
        let lot_counts: (i64, i64) = sqlx::query_as(
            r#"
            SELECT 
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE stage NOT IN ('sold', 'disposed')) as active
            FROM lots WHERE business_id = $1
            "#,
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        // Total inventory
        let inventory_kg: Decimal = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(
                CASE WHEN transaction_type IN ('harvest_in', 'processing_in', 'roasting_in', 'purchase', 'return')
                    THEN quantity_kg
                    ELSE -quantity_kg
                END
            ), 0)
            FROM inventory_transactions it
            JOIN lots l ON l.id = it.lot_id
            WHERE l.business_id = $1
            "#,
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        // Average cupping score (last 30 days)
        let avg_score: Option<Decimal> = sqlx::query_scalar(
            r#"
            SELECT AVG(csamp.total_score)
            FROM cupping_samples csamp
            JOIN cupping_sessions cs ON cs.id = csamp.session_id
            WHERE cs.business_id = $1
              AND cs.session_date >= CURRENT_DATE - INTERVAL '30 days'
            "#,
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        // Pending alerts
        let pending_alerts: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM inventory_alerts 
            WHERE business_id = $1 AND is_active = true AND acknowledged_at IS NULL
            "#,
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await
        .unwrap_or(0);

        // Recent harvests (last 7 days)
        let recent_harvests: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM harvests h
            JOIN lots l ON l.id = h.lot_id
            WHERE l.business_id = $1
              AND h.harvest_date >= CURRENT_DATE - INTERVAL '7 days'
            "#,
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        // Expiring certifications (next 90 days)
        let expiring_certs: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM certifications
            WHERE business_id = $1
              AND status = 'active'
              AND expiration_date <= CURRENT_DATE + INTERVAL '90 days'
            "#,
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        Ok(DashboardMetrics {
            total_lots: lot_counts.0,
            active_lots: lot_counts.1,
            total_inventory_kg: inventory_kg,
            avg_cupping_score: avg_score,
            pending_alerts,
            recent_harvests,
            expiring_certifications: expiring_certs,
        })
    }

    /// Export report data as CSV
    pub fn export_to_csv<T: Serialize>(data: &[T]) -> AppResult<String> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        for record in data {
            wtr.serialize(record).map_err(|e| {
                crate::error::AppError::Internal(format!("CSV serialization error: {}", e))
            })?;
        }
        let csv_data = String::from_utf8(wtr.into_inner().map_err(|e| {
            crate::error::AppError::Internal(format!("CSV writer error: {}", e))
        })?)
        .map_err(|e| crate::error::AppError::Internal(format!("UTF-8 conversion error: {}", e)))?;
        Ok(csv_data)
    }
}
