//! Reporting handlers for analytics and data export

use axum::{
    extract::{Query, State},
    http::header,
    response::IntoResponse,
    Extension, Json,
};
use serde::Deserialize;

use crate::error::AppResult;
use crate::middleware::auth::AuthUser;
use crate::services::reporting::{
    DashboardMetrics, HarvestYieldReport, ProcessingEfficiencyReport, QualityTrendPoint,
    ReportFilter, ReportingService,
};
use crate::AppState;

#[derive(Deserialize)]
pub struct ReportQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub format: Option<String>, // "json" or "csv"
}

#[derive(Deserialize)]
pub struct QualityTrendQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub group_by: Option<String>, // "month", "quarter", "year"
    pub format: Option<String>,
}

/// Get dashboard metrics
pub async fn get_dashboard(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<DashboardMetrics>> {
    let service = ReportingService::new(state.db.clone());
    let metrics = service.get_dashboard_metrics(user.business_id).await?;
    Ok(Json(metrics))
}

/// Get harvest yield report
pub async fn get_harvest_yield_report(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<ReportQuery>,
) -> AppResult<impl IntoResponse> {
    let service = ReportingService::new(state.db.clone());

    let filter = ReportFilter {
        start_date: query.start_date.and_then(|s| s.parse().ok()),
        end_date: query.end_date.and_then(|s| s.parse().ok()),
        plot_ids: None,
        varieties: None,
        processing_methods: None,
    };

    let data = service.get_harvest_yield_report(user.business_id, &filter).await?;

    if query.format.as_deref() == Some("csv") {
        let csv = ReportingService::export_to_csv(&data)?;
        Ok((
            [(header::CONTENT_TYPE, "text/csv"), (header::CONTENT_DISPOSITION, "attachment; filename=\"harvest_yield.csv\"")],
            csv,
        ).into_response())
    } else {
        Ok(Json(data).into_response())
    }
}

/// Get quality trend report
pub async fn get_quality_trend_report(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<QualityTrendQuery>,
) -> AppResult<impl IntoResponse> {
    let service = ReportingService::new(state.db.clone());

    let filter = ReportFilter {
        start_date: query.start_date.and_then(|s| s.parse().ok()),
        end_date: query.end_date.and_then(|s| s.parse().ok()),
        plot_ids: None,
        varieties: None,
        processing_methods: None,
    };

    let group_by = query.group_by.as_deref().unwrap_or("month");
    let data = service.get_quality_trend_report(user.business_id, &filter, group_by).await?;

    if query.format.as_deref() == Some("csv") {
        let csv = ReportingService::export_to_csv(&data)?;
        Ok((
            [(header::CONTENT_TYPE, "text/csv"), (header::CONTENT_DISPOSITION, "attachment; filename=\"quality_trend.csv\"")],
            csv,
        ).into_response())
    } else {
        Ok(Json(data).into_response())
    }
}

/// Get processing efficiency report
pub async fn get_processing_efficiency_report(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<ReportQuery>,
) -> AppResult<impl IntoResponse> {
    let service = ReportingService::new(state.db.clone());

    let filter = ReportFilter {
        start_date: query.start_date.and_then(|s| s.parse().ok()),
        end_date: query.end_date.and_then(|s| s.parse().ok()),
        plot_ids: None,
        varieties: None,
        processing_methods: None,
    };

    let data = service.get_processing_efficiency_report(user.business_id, &filter).await?;

    if query.format.as_deref() == Some("csv") {
        let csv = ReportingService::export_to_csv(&data)?;
        Ok((
            [(header::CONTENT_TYPE, "text/csv"), (header::CONTENT_DISPOSITION, "attachment; filename=\"processing_efficiency.csv\"")],
            csv,
        ).into_response())
    } else {
        Ok(Json(data).into_response())
    }
}
