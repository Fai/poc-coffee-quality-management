//! HTTP handlers for processing management

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    error::AppResult,
    middleware::CurrentUser,
    services::processing::{
        CompleteProcessingInput, LogDryingInput, LogFermentationInput, ProcessingService,
        StartProcessingInput,
    },
    AppState,
};

/// Start processing for a lot
pub async fn start_processing(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(input): Json<StartProcessingInput>,
) -> AppResult<impl IntoResponse> {
    let service = ProcessingService::new(state.db);
    let record = service.start_processing(user.0.business_id, input).await?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// Log fermentation data
pub async fn log_fermentation(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(processing_id): Path<Uuid>,
    Json(input): Json<LogFermentationInput>,
) -> AppResult<impl IntoResponse> {
    let service = ProcessingService::new(state.db);
    let record = service
        .log_fermentation(user.0.business_id, processing_id, input)
        .await?;
    Ok(Json(record))
}

/// Log drying data
pub async fn log_drying(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(processing_id): Path<Uuid>,
    Json(input): Json<LogDryingInput>,
) -> AppResult<impl IntoResponse> {
    let service = ProcessingService::new(state.db);
    let record = service
        .log_drying(user.0.business_id, processing_id, input)
        .await?;
    Ok(Json(record))
}

/// Complete processing
pub async fn complete_processing(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(processing_id): Path<Uuid>,
    Json(input): Json<CompleteProcessingInput>,
) -> AppResult<impl IntoResponse> {
    let service = ProcessingService::new(state.db);
    let record = service
        .complete_processing(user.0.business_id, processing_id, input)
        .await?;
    Ok(Json(record))
}

/// Get processing record by ID
pub async fn get_processing(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(processing_id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let service = ProcessingService::new(state.db);
    let record = service
        .get_processing(user.0.business_id, processing_id)
        .await?;
    Ok(Json(record))
}

/// Get processing record by lot ID
pub async fn get_processing_by_lot(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(lot_id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let service = ProcessingService::new(state.db);
    let record = service
        .get_processing_by_lot(user.0.business_id, lot_id)
        .await?;
    Ok(Json(record))
}

/// List all processing records
pub async fn list_processing(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> AppResult<impl IntoResponse> {
    let service = ProcessingService::new(state.db);
    let records = service.list_processing(user.0.business_id).await?;
    Ok(Json(records))
}
