//! Lot management HTTP handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use uuid::Uuid;

use crate::middleware::CurrentUser;
use crate::services::lot::{BlendLotsInput, CreateLotInput, LotService, UpdateLotInput};
use crate::AppState;

/// List all lots for the current business
pub async fn list_lots(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> impl IntoResponse {
    let service = LotService::new(state.db.clone());
    
    match service.get_lots(current_user.0.business_id).await {
        Ok(lots) => (StatusCode::OK, Json(serde_json::json!({ "lots": lots }))).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get a specific lot with its sources
pub async fn get_lot(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(lot_id): Path<Uuid>,
) -> impl IntoResponse {
    let service = LotService::new(state.db.clone());
    
    match service.get_lot_with_sources(current_user.0.business_id, lot_id).await {
        Ok(lot) => (StatusCode::OK, Json(lot)).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Create a new lot
pub async fn create_lot(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(input): Json<CreateLotInput>,
) -> impl IntoResponse {
    let service = LotService::new(state.db.clone());
    
    // Get business code for traceability code generation
    let business_code = match sqlx::query_scalar::<_, String>(
        "SELECT code FROM businesses WHERE id = $1"
    )
    .bind(current_user.0.business_id)
    .fetch_one(&state.db)
    .await {
        Ok(code) => code,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    
    match service.create_lot(current_user.0.business_id, &business_code, input).await {
        Ok(lot) => (StatusCode::CREATED, Json(lot)).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Blend multiple lots into a new lot
pub async fn blend_lots(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(input): Json<BlendLotsInput>,
) -> impl IntoResponse {
    let service = LotService::new(state.db.clone());
    
    // Get business code for traceability code generation
    let business_code = match sqlx::query_scalar::<_, String>(
        "SELECT code FROM businesses WHERE id = $1"
    )
    .bind(current_user.0.business_id)
    .fetch_one(&state.db)
    .await {
        Ok(code) => code,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    
    match service.blend_lots(current_user.0.business_id, &business_code, input).await {
        Ok(lot) => (StatusCode::CREATED, Json(lot)).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Update a lot
pub async fn update_lot(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(lot_id): Path<Uuid>,
    Json(input): Json<UpdateLotInput>,
) -> impl IntoResponse {
    let service = LotService::new(state.db.clone());
    
    match service.update_lot(current_user.0.business_id, lot_id, input).await {
        Ok(lot) => (StatusCode::OK, Json(lot)).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get lot by traceability code (public endpoint)
pub async fn get_lot_by_code(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    let service = LotService::new(state.db.clone());
    
    match service.get_lot_by_code(&code).await {
        Ok(lot) => (StatusCode::OK, Json(lot)).into_response(),
        Err(e) => e.into_response(),
    }
}
