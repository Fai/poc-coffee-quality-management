//! HTTP handlers for LINE chatbot webhook

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::Serialize;

use crate::services::line_chatbot::{LineChatbotService, LineWebhookRequest};
use crate::AppState;

// ============================================================================
// Response Types
// ============================================================================

/// Response for webhook processing
#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    pub success: bool,
    pub message: String,
}

// ============================================================================
// Handlers
// ============================================================================

/// Handle LINE webhook events
/// POST /webhook/line
/// 
/// This endpoint receives webhook events from LINE Messaging API.
/// It verifies the signature and processes messages for quick data entry.
pub async fn handle_line_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<WebhookResponse>, (StatusCode, Json<WebhookResponse>)> {
    // Verify LINE signature
    if let Err(e) = verify_line_signature(&headers, &body) {
        tracing::warn!("LINE webhook signature verification failed: {}", e);
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(WebhookResponse {
                success: false,
                message: "Invalid signature".to_string(),
            }),
        ));
    }

    // Parse webhook request
    let request: LineWebhookRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to parse LINE webhook: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(WebhookResponse {
                    success: false,
                    message: format!("Invalid request body: {}", e),
                }),
            ));
        }
    };

    // Process webhook events
    let service = LineChatbotService::new(state.db.clone());
    
    if let Err(e) = service.process_webhook(request).await {
        tracing::error!("Failed to process LINE webhook: {}", e);
        // Still return 200 to LINE to prevent retries
        return Ok(Json(WebhookResponse {
            success: false,
            message: format!("Processing error: {}", e),
        }));
    }

    Ok(Json(WebhookResponse {
        success: true,
        message: "Webhook processed successfully".to_string(),
    }))
}

/// Verify LINE webhook signature
fn verify_line_signature(headers: &HeaderMap, body: &[u8]) -> Result<(), String> {
    // Get channel secret from environment
    let channel_secret = std::env::var("LINE_CHANNEL_SECRET")
        .map_err(|_| "LINE_CHANNEL_SECRET not configured")?;

    // Get signature from header
    let signature = headers
        .get("x-line-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or("Missing x-line-signature header")?;

    // Calculate expected signature
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(channel_secret.as_bytes())
        .map_err(|_| "Failed to create HMAC")?;
    mac.update(body);
    let expected = BASE64.encode(mac.finalize().into_bytes());

    // Compare signatures
    if signature != expected {
        return Err("Signature mismatch".to_string());
    }

    Ok(())
}
