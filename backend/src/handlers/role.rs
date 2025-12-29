//! Role management handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::middleware::CurrentUser;
use crate::services::role::{CreateRoleInput, Permission, Role, RoleWithPermissions, UpdateRoleInput};
use crate::services::RoleService;
use crate::AppState;

/// Response for list of roles
#[derive(Serialize)]
pub struct RolesResponse {
    pub roles: Vec<Role>,
}

/// Response for list of permissions
#[derive(Serialize)]
pub struct PermissionsResponse {
    pub permissions: Vec<Permission>,
}

/// Get all roles for the current business
pub async fn list_roles(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<RolesResponse>, AppError> {
    if !user.has_permission("role", "view") {
        return Err(AppError::InsufficientPermissions);
    }

    let role_service = RoleService::new(state.db.clone());
    let roles = role_service.get_roles(user.business_id).await?;

    Ok(Json(RolesResponse { roles }))
}

/// Get a specific role with its permissions
pub async fn get_role(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(role_id): Path<Uuid>,
) -> Result<Json<RoleWithPermissions>, AppError> {
    if !user.has_permission("role", "view") {
        return Err(AppError::InsufficientPermissions);
    }

    let role_service = RoleService::new(state.db.clone());
    let role = role_service
        .get_role_with_permissions(user.business_id, role_id)
        .await?;

    Ok(Json(role))
}

/// Get all available permissions
pub async fn list_permissions(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<PermissionsResponse>, AppError> {
    if !user.has_permission("role", "view") {
        return Err(AppError::InsufficientPermissions);
    }

    let role_service = RoleService::new(state.db.clone());
    let permissions = role_service.get_all_permissions().await?;

    Ok(Json(PermissionsResponse { permissions }))
}

/// Create a new custom role
pub async fn create_role(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(input): Json<CreateRoleInput>,
) -> Result<(StatusCode, Json<RoleWithPermissions>), AppError> {
    if !user.has_permission("role", "create") {
        return Err(AppError::InsufficientPermissions);
    }

    let role_service = RoleService::new(state.db.clone());
    let role = role_service.create_role(user.business_id, input).await?;

    Ok((StatusCode::CREATED, Json(role)))
}

/// Update an existing role
pub async fn update_role(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(role_id): Path<Uuid>,
    Json(input): Json<UpdateRoleInput>,
) -> Result<Json<RoleWithPermissions>, AppError> {
    if !user.has_permission("role", "edit") {
        return Err(AppError::InsufficientPermissions);
    }

    let role_service = RoleService::new(state.db.clone());
    let role = role_service
        .update_role(user.business_id, role_id, input)
        .await?;

    Ok(Json(role))
}

/// Delete a custom role
pub async fn delete_role(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(role_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if !user.has_permission("role", "delete") {
        return Err(AppError::InsufficientPermissions);
    }

    let role_service = RoleService::new(state.db.clone());
    role_service.delete_role(user.business_id, role_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
