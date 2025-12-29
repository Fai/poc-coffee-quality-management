//! Harvest management HTTP handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use uuid::Uuid;

use crate::middleware::CurrentUser;
use crate::services::harvest::{HarvestService, RecordHarvestInput, UpdateHarvestInput};
use crate::AppState;

/// List all harvests for the current business
pub async fn list_harvests(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> impl IntoResponse {
    let service = HarvestService::new(state.db.clone());
    
    match service.get_harvests(current_user.0.business_id).await {
        Ok(harvests) => (StatusCode::OK, Json(serde_json::json!({ "harvests": harvests }))).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get harvests for a specific lot
pub async fn get_harvests_by_lot(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(lot_id): Path<Uuid>,
) -> impl IntoResponse {
    let service = HarvestService::new(state.db.clone());
    
    match service.get_harvests_by_lot(current_user.0.business_id, lot_id).await {
        Ok(harvests) => (StatusCode::OK, Json(serde_json::json!({ "harvests": harvests }))).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get a specific harvest
pub async fn get_harvest(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(harvest_id): Path<Uuid>,
) -> impl IntoResponse {
    let service = HarvestService::new(state.db.clone());
    
    match service.get_harvest(current_user.0.business_id, harvest_id).await {
        Ok(harvest) => (StatusCode::OK, Json(harvest)).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Record a new harvest
pub async fn record_harvest(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(input): Json<RecordHarvestInput>,
) -> impl IntoResponse {
    let service = HarvestService::new(state.db.clone());
    
    // Get business code for lot traceability code generation
    let business_code = match sqlx::query_scalar::<_, String>(
        "SELECT code FROM businesses WHERE id = $1"
    )
    .bind(current_user.0.business_id)
    .fetch_one(&state.db)
    .await {
        Ok(code) => code,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    
    match service.record_harvest(current_user.0.business_id, &business_code, input).await {
        Ok(harvest) => (StatusCode::CREATED, Json(harvest)).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Update a harvest
pub async fn update_harvest(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(harvest_id): Path<Uuid>,
    Json(input): Json<UpdateHarvestInput>,
) -> impl IntoResponse {
    let service = HarvestService::new(state.db.clone());
    
    match service.update_harvest(current_user.0.business_id, harvest_id, input).await {
        Ok(harvest) => (StatusCode::OK, Json(harvest)).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Delete a harvest
pub async fn delete_harvest(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(harvest_id): Path<Uuid>,
) -> impl IntoResponse {
    let service = HarvestService::new(state.db.clone());
    
    match service.delete_harvest(current_user.0.business_id, harvest_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => e.into_response(),
    }
}
