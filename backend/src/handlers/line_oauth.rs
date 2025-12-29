//! HTTP handlers for LINE OAuth endpoints

use axum::{
    extract::{Query, State},
    response::Redirect,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::middleware::CurrentUser;
use crate::services::line_oauth::{LineConnection, LineOAuthConfig, LineOAuthResult, LineOAuthService};
use crate::AppState;

// ============================================================================
// Request/Response Types
// ============================================================================

/// Query parameters for OAuth callback
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: Option<String>,
}

/// Response for authorization URL
#[derive(Debug, Serialize)]
pub struct AuthorizationUrlResponse {
    pub url: String,
    pub state: String,
}

/// Response for LINE connection status
#[derive(Debug, Serialize)]
pub struct ConnectionStatusResponse {
    pub connected: bool,
    pub line_user_id: Option<String>,
    pub display_name: Option<String>,
    pub picture_url: Option<String>,
    pub connected_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Response for disconnect
#[derive(Debug, Serialize)]
pub struct DisconnectResponse {
    pub success: bool,
    pub message: String,
}

// ============================================================================
// Handlers
// ============================================================================

/// Get LINE OAuth authorization URL
/// GET /auth/line
pub async fn get_authorization_url(
    State(state): State<AppState>,
) -> AppResult<Json<AuthorizationUrlResponse>> {
    let service = get_line_service(&state)?;
    
    // Generate a random state for CSRF protection
    let oauth_state = Uuid::new_v4().to_string();
    
    let url = service.get_authorization_url(&oauth_state);
    
    Ok(Json(AuthorizationUrlResponse {
        url,
        state: oauth_state,
    }))
}

/// Handle LINE OAuth callback (for linking to existing user)
/// GET /auth/line/callback
pub async fn handle_callback(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<OAuthCallbackQuery>,
) -> AppResult<Json<LineOAuthResult>> {
    let service = get_line_service(&state)?;
    
    // Handle callback with current user ID for linking
    let result = service
        .handle_callback(&query.code, Some(current_user.0.user_id))
        .await?;
    
    Ok(Json(result))
}

/// Handle LINE OAuth callback (public - for login/registration)
/// GET /auth/line/callback/public
pub async fn handle_public_callback(
    State(state): State<AppState>,
    Query(query): Query<OAuthCallbackQuery>,
) -> AppResult<Json<LineOAuthResult>> {
    let service = get_line_service(&state)?;
    
    // Handle callback without user ID - will check if LINE is already linked
    let result = service.handle_callback(&query.code, None).await?;
    
    Ok(Json(result))
}

/// Get LINE connection status for current user
/// GET /auth/line/status
pub async fn get_connection_status(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<ConnectionStatusResponse>> {
    let service = get_line_service(&state)?;
    
    let connection = service.get_connection(current_user.0.user_id).await?;
    
    match connection {
        Some(conn) => Ok(Json(ConnectionStatusResponse {
            connected: true,
            line_user_id: Some(conn.line_user_id),
            display_name: conn.display_name,
            picture_url: conn.picture_url,
            connected_at: Some(conn.connected_at),
        })),
        None => Ok(Json(ConnectionStatusResponse {
            connected: false,
            line_user_id: None,
            display_name: None,
            picture_url: None,
            connected_at: None,
        })),
    }
}

/// Disconnect LINE from current user
/// DELETE /auth/line
pub async fn disconnect_line(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<DisconnectResponse>> {
    let service = get_line_service(&state)?;
    
    let disconnected = service.disconnect(current_user.0.user_id).await?;
    
    if disconnected {
        Ok(Json(DisconnectResponse {
            success: true,
            message: "LINE account disconnected successfully".to_string(),
        }))
    } else {
        Ok(Json(DisconnectResponse {
            success: false,
            message: "No LINE account was connected".to_string(),
        }))
    }
}

/// Get LINE connection details
/// GET /auth/line/connection
pub async fn get_connection(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<Option<LineConnection>>> {
    let service = get_line_service(&state)?;
    
    let connection = service.get_connection(current_user.0.user_id).await?;
    
    // Remove sensitive tokens from response
    let safe_connection = connection.map(|mut c| {
        c.access_token = None;
        c.refresh_token = None;
        c
    });
    
    Ok(Json(safe_connection))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get LINE OAuth service from app state
fn get_line_service(state: &AppState) -> AppResult<LineOAuthService> {
    let client_id = std::env::var("LINE_CHANNEL_ID")
        .map_err(|_| AppError::Configuration("LINE_CHANNEL_ID not configured".to_string()))?;
    let client_secret = std::env::var("LINE_CHANNEL_SECRET")
        .map_err(|_| AppError::Configuration("LINE_CHANNEL_SECRET not configured".to_string()))?;
    let redirect_uri = std::env::var("LINE_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:3000/auth/line/callback".to_string());

    Ok(LineOAuthService::new(
        state.db.clone(),
        LineOAuthConfig {
            client_id,
            client_secret,
            redirect_uri,
        },
    ))
}
