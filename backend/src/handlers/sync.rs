//! Sync handlers for offline support

use axum::{extract::State, Extension, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::middleware::auth::AuthUser;
use crate::services::sync::{ConflictResolution, PendingChange, SyncChange, SyncConflict, SyncService};
use crate::AppState;

#[derive(Deserialize)]
pub struct GetChangesRequest {
    pub since_version: i64,
    #[serde(default = "default_limit")]
    pub limit: i32,
    pub device_id: String,
}

fn default_limit() -> i32 {
    1000
}

#[derive(Serialize)]
pub struct GetChangesResponse {
    pub changes: Vec<SyncChange>,
    pub server_version: i64,
}

#[derive(Deserialize)]
pub struct ApplyChangesRequest {
    pub changes: Vec<PendingChange>,
    pub device_id: String,
}

#[derive(Serialize)]
pub struct ApplyChangesResponse {
    pub applied: Vec<Uuid>,
    pub conflicts: Vec<SyncConflict>,
    pub server_version: i64,
}

#[derive(Deserialize)]
pub struct ResolveConflictRequest {
    pub conflict_id: Uuid,
    pub resolution: String, // "keep_local", "keep_server", "merge"
    pub merged_data: Option<serde_json::Value>,
}

/// Get changes since last sync
pub async fn get_changes(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<GetChangesRequest>,
) -> AppResult<Json<GetChangesResponse>> {
    let sync_service = SyncService::new(state.db.clone());

    let changes = sync_service
        .get_changes_since(user.business_id, body.since_version, body.limit)
        .await?;

    let server_version = changes.last().map(|c| c.entity_version).unwrap_or(body.since_version);

    // Update sync state
    sync_service
        .update_sync_state(user.user_id, &body.device_id, server_version)
        .await?;

    Ok(Json(GetChangesResponse {
        changes,
        server_version,
    }))
}

/// Apply pending changes from client
pub async fn apply_changes(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<ApplyChangesRequest>,
) -> AppResult<Json<ApplyChangesResponse>> {
    let sync_service = SyncService::new(state.db.clone());

    let result = sync_service
        .apply_changes(user.user_id, user.business_id, body.changes)
        .await?;

    // Update sync state
    sync_service
        .update_sync_state(user.user_id, &body.device_id, result.server_version)
        .await?;

    Ok(Json(ApplyChangesResponse {
        applied: result.applied,
        conflicts: result.conflicts,
        server_version: result.server_version,
    }))
}

/// Get pending conflicts for user
pub async fn get_conflicts(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Vec<SyncConflict>>> {
    let sync_service = SyncService::new(state.db.clone());
    let conflicts = sync_service.get_pending_conflicts(user.user_id).await?;
    Ok(Json(conflicts))
}

/// Resolve a sync conflict
pub async fn resolve_conflict(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<ResolveConflictRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let sync_service = SyncService::new(state.db.clone());

    let resolution = match body.resolution.as_str() {
        "keep_local" => ConflictResolution::KeepLocal,
        "keep_server" => ConflictResolution::KeepServer,
        "merge" => {
            let data = body.merged_data.ok_or(AppError::Validation {
                field: "merged_data".to_string(),
                message: "merged_data required for merge resolution".to_string(),
            })?;
            ConflictResolution::Merge(data)
        }
        _ => {
            return Err(AppError::Validation {
                field: "resolution".to_string(),
                message: "Invalid resolution type".to_string(),
            })
        }
    };

    sync_service
        .resolve_conflict(body.conflict_id, user.user_id, resolution)
        .await?;

    Ok(Json(serde_json::json!({"status": "resolved"})))
}
