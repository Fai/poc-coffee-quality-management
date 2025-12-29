//! HTTP handlers for roast profile management endpoints

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppResult;
use crate::middleware::CurrentUser;
use crate::services::roasting::{
    CompleteRoastInput, CreateTemplateInput, CuppingSampleSummary, LogMilestonesInput,
    LogTemperatureInput, RoastProfileTemplate, RoastSession, RoastingService,
    StartRoastSessionInput, UpdateTemplateInput,
};
use crate::AppState;

// ============================================================================
// Profile Template Handlers
// ============================================================================

/// Create a new roast profile template
pub async fn create_template(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(input): Json<CreateTemplateInput>,
) -> AppResult<Json<RoastProfileTemplate>> {
    let service = RoastingService::new(state.db);
    let template = service
        .create_template(current_user.0.business_id, current_user.0.user_id, input)
        .await?;
    Ok(Json(template))
}

/// Get a roast profile template by ID
pub async fn get_template(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(template_id): Path<Uuid>,
) -> AppResult<Json<RoastProfileTemplate>> {
    let service = RoastingService::new(state.db);
    let template = service
        .get_template(current_user.0.business_id, template_id)
        .await?;
    Ok(Json(template))
}

/// Query parameters for listing templates
#[derive(Debug, Deserialize)]
pub struct ListTemplatesQuery {
    pub active_only: Option<bool>,
}

/// List all roast profile templates
pub async fn list_templates(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<ListTemplatesQuery>,
) -> AppResult<Json<Vec<RoastProfileTemplate>>> {
    let service = RoastingService::new(state.db);
    let active_only = query.active_only.unwrap_or(true);
    let templates = service
        .list_templates(current_user.0.business_id, active_only)
        .await?;
    Ok(Json(templates))
}

/// Update a roast profile template
pub async fn update_template(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(template_id): Path<Uuid>,
    Json(input): Json<UpdateTemplateInput>,
) -> AppResult<Json<RoastProfileTemplate>> {
    let service = RoastingService::new(state.db);
    let template = service
        .update_template(current_user.0.business_id, template_id, input)
        .await?;
    Ok(Json(template))
}

/// Delete a roast profile template (soft delete)
pub async fn delete_template(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(template_id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let service = RoastingService::new(state.db);
    service
        .delete_template(current_user.0.business_id, template_id)
        .await?;
    Ok(Json(()))
}

// ============================================================================
// Roast Session Handlers
// ============================================================================

/// Start a new roast session
pub async fn start_session(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(input): Json<StartRoastSessionInput>,
) -> AppResult<Json<RoastSession>> {
    let service = RoastingService::new(state.db);
    let session = service
        .start_session(current_user.0.business_id, current_user.0.user_id, input)
        .await?;
    Ok(Json(session))
}

/// Get a roast session by ID
pub async fn get_session(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(session_id): Path<Uuid>,
) -> AppResult<Json<RoastSession>> {
    let service = RoastingService::new(state.db);
    let session = service
        .get_session(current_user.0.business_id, session_id)
        .await?;
    Ok(Json(session))
}

/// List all roast sessions
pub async fn list_sessions(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<Vec<RoastSession>>> {
    let service = RoastingService::new(state.db);
    let sessions = service.list_sessions(current_user.0.business_id).await?;
    Ok(Json(sessions))
}

/// Get roast sessions for a lot
pub async fn get_sessions_by_lot(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(lot_id): Path<Uuid>,
) -> AppResult<Json<Vec<RoastSession>>> {
    let service = RoastingService::new(state.db);
    let sessions = service
        .get_sessions_by_lot(current_user.0.business_id, lot_id)
        .await?;
    Ok(Json(sessions))
}

/// Log temperature checkpoints
pub async fn log_temperature(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(session_id): Path<Uuid>,
    Json(input): Json<LogTemperatureInput>,
) -> AppResult<Json<RoastSession>> {
    let service = RoastingService::new(state.db);
    let session = service
        .log_temperature(current_user.0.business_id, session_id, input)
        .await?;
    Ok(Json(session))
}

/// Log roast milestones
pub async fn log_milestones(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(session_id): Path<Uuid>,
    Json(input): Json<LogMilestonesInput>,
) -> AppResult<Json<RoastSession>> {
    let service = RoastingService::new(state.db);
    let session = service
        .log_milestones(current_user.0.business_id, session_id, input)
        .await?;
    Ok(Json(session))
}

/// Complete a roast session
pub async fn complete_session(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(session_id): Path<Uuid>,
    Json(input): Json<CompleteRoastInput>,
) -> AppResult<Json<RoastSession>> {
    let service = RoastingService::new(state.db);
    let session = service
        .complete_session(current_user.0.business_id, session_id, input)
        .await?;
    Ok(Json(session))
}

/// Input for failing a session
#[derive(Debug, Deserialize)]
pub struct FailSessionInput {
    pub notes: Option<String>,
    pub notes_th: Option<String>,
}

/// Mark a roast session as failed
pub async fn fail_session(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(session_id): Path<Uuid>,
    Json(input): Json<FailSessionInput>,
) -> AppResult<Json<RoastSession>> {
    let service = RoastingService::new(state.db);
    let session = service
        .fail_session(
            current_user.0.business_id,
            session_id,
            input.notes,
            input.notes_th,
        )
        .await?;
    Ok(Json(session))
}

/// Get cupping samples linked to a roast session
pub async fn get_session_cuppings(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(session_id): Path<Uuid>,
) -> AppResult<Json<Vec<CuppingSampleSummary>>> {
    let service = RoastingService::new(state.db);
    let samples = service
        .get_session_cuppings(current_user.0.business_id, session_id)
        .await?;
    Ok(Json(samples))
}
