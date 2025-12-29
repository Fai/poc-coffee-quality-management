//! HTTP handlers for cupping session and score management

use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{
    error::AppResult,
    middleware::CurrentUser,
    services::cupping::{
        AddCuppingSampleInput, CreateCuppingSessionInput, CuppingSample, CuppingSession,
        CuppingTrend,
    },
    services::CuppingService,
    AppState,
};

/// Create a new cupping session
pub async fn create_cupping_session(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(input): Json<CreateCuppingSessionInput>,
) -> AppResult<Json<CuppingSession>> {
    let service = CuppingService::new(state.db);
    let session = service.create_session(current_user.0.business_id, input).await?;
    Ok(Json(session))
}

/// Add a sample to a cupping session
pub async fn add_cupping_sample(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(session_id): Path<Uuid>,
    Json(input): Json<AddCuppingSampleInput>,
) -> AppResult<Json<CuppingSample>> {
    let service = CuppingService::new(state.db);
    let sample = service.add_sample(current_user.0.business_id, session_id, input).await?;
    Ok(Json(sample))
}

/// Get a cupping session with all samples
pub async fn get_cupping_session(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(session_id): Path<Uuid>,
) -> AppResult<Json<CuppingSession>> {
    let service = CuppingService::new(state.db);
    let session = service.get_session(current_user.0.business_id, session_id).await?;
    Ok(Json(session))
}

/// List all cupping sessions for the business
pub async fn list_cupping_sessions(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<Vec<CuppingSession>>> {
    let service = CuppingService::new(state.db);
    let sessions = service.list_sessions(current_user.0.business_id).await?;
    Ok(Json(sessions))
}

/// Get cupping history for a lot
pub async fn get_lot_cupping_history(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(lot_id): Path<Uuid>,
) -> AppResult<Json<Vec<CuppingSample>>> {
    let service = CuppingService::new(state.db);
    let samples = service.get_lot_cupping_history(current_user.0.business_id, lot_id).await?;
    Ok(Json(samples))
}

/// Get cupping trend for a lot
pub async fn get_lot_cupping_trend(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(lot_id): Path<Uuid>,
) -> AppResult<Json<CuppingTrend>> {
    let service = CuppingService::new(state.db);
    let trend = service.get_lot_cupping_trend(current_user.0.business_id, lot_id).await?;
    Ok(Json(trend))
}
