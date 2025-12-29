//! Authentication handlers

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::services::AuthService;
use crate::AppState;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub business_name: String,
    pub business_type: String,
    pub business_code: String,
    pub owner_name: String,
    pub email: String,
    pub password: String,
    pub phone: Option<String>,
    pub province: Option<String>,
    pub preferred_language: Option<String>,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub business_id: String,
    pub user_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Login endpoint handler
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let auth_service = AuthService::new(state.db.clone(), &state.config);
    let tokens = auth_service.login(&body.email, &body.password).await?;

    Ok(Json(LoginResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        token_type: tokens.token_type,
        expires_in: tokens.expires_in,
    }))
}

/// Register business endpoint handler
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), AppError> {
    use crate::services::auth::RegisterBusinessInput;
    use shared::types::Language;

    let language = body.preferred_language.as_deref().map(|l| match l {
        "en" => Language::English,
        _ => Language::Thai,
    });

    let input = RegisterBusinessInput {
        business_name: body.business_name,
        business_type: body.business_type,
        business_code: body.business_code,
        owner_name: body.owner_name,
        email: body.email,
        password: body.password,
        phone: body.phone,
        province: body.province,
        preferred_language: language,
    };

    let auth_service = AuthService::new(state.db.clone(), &state.config);
    let result = auth_service.register_business(input).await?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            business_id: result.business_id.to_string(),
            user_id: result.user_id.to_string(),
            access_token: result.access_token,
            refresh_token: result.refresh_token,
            token_type: result.token_type,
            expires_in: result.expires_in,
        }),
    ))
}

/// Refresh token endpoint handler
pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let auth_service = AuthService::new(state.db.clone(), &state.config);
    let tokens = auth_service.refresh_token(&body.refresh_token).await?;

    Ok(Json(LoginResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        token_type: tokens.token_type,
        expires_in: tokens.expires_in,
    }))
}
