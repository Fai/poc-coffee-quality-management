//! HTTP handlers for green bean grading endpoints

use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::error::AppResult;
use crate::middleware::CurrentUser;
use crate::services::grading::{
    GradingComparison, GradingRecord, GradingService, RecordGradingInput, RecordGradingWithAiInput,
};
use crate::AppState;

/// Record a green bean grading (manual entry)
pub async fn record_grading(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(input): Json<RecordGradingInput>,
) -> AppResult<Json<GradingRecord>> {
    let service = GradingService::new(state.db);
    let grading = service.record_grading(current_user.0.business_id, input).await?;
    Ok(Json(grading))
}

/// Record grading with AI-assisted defect detection
pub async fn record_grading_with_ai(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(input): Json<RecordGradingWithAiInput>,
) -> AppResult<Json<GradingRecord>> {
    let service = GradingService::new(state.db);
    let grading = service
        .record_grading_with_ai(current_user.0.business_id, input)
        .await?;
    Ok(Json(grading))
}

/// Get grading record by ID
pub async fn get_grading(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(grading_id): Path<Uuid>,
) -> AppResult<Json<GradingRecord>> {
    let service = GradingService::new(state.db);
    let grading = service
        .get_grading(current_user.0.business_id, grading_id)
        .await?;
    Ok(Json(grading))
}

/// Get grading history for a lot
pub async fn get_grading_history(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(lot_id): Path<Uuid>,
) -> AppResult<Json<Vec<GradingRecord>>> {
    let service = GradingService::new(state.db);
    let gradings = service
        .get_grading_history(current_user.0.business_id, lot_id)
        .await?;
    Ok(Json(gradings))
}

/// List all grading records for the business
pub async fn list_gradings(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<Vec<GradingRecord>>> {
    let service = GradingService::new(state.db);
    let gradings = service.list_gradings(current_user.0.business_id).await?;
    Ok(Json(gradings))
}

/// Get grading comparison for a lot
pub async fn get_grading_comparison(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(lot_id): Path<Uuid>,
) -> AppResult<Json<GradingComparison>> {
    let service = GradingService::new(state.db);
    let comparison = service
        .get_grading_comparison(current_user.0.business_id, lot_id)
        .await?;
    Ok(Json(comparison))
}
