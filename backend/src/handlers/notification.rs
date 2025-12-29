//! HTTP handlers for notification management endpoints

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppResult;
use crate::middleware::CurrentUser;
use crate::services::notification::{
    CreateNotificationInput, InAppNotification, NotificationLogEntry,
    NotificationPreferences, NotificationService, UpdatePreferencesInput,
};
use crate::AppState;

// ============================================================================
// Notification Preferences
// ============================================================================

/// Get notification preferences
pub async fn get_preferences(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<NotificationPreferences>> {
    let service = NotificationService::new(state.db);
    let prefs = service.get_preferences(current_user.0.user_id).await?;
    Ok(Json(prefs))
}

/// Update notification preferences
pub async fn update_preferences(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(input): Json<UpdatePreferencesInput>,
) -> AppResult<Json<NotificationPreferences>> {
    let service = NotificationService::new(state.db);
    let prefs = service
        .update_preferences(current_user.0.user_id, input)
        .await?;
    Ok(Json(prefs))
}

// ============================================================================
// In-App Notifications
// ============================================================================

/// Query parameters for listing notifications
#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    pub unread_only: Option<bool>,
    pub limit: Option<i32>,
}

/// Get in-app notifications
pub async fn get_notifications(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<ListNotificationsQuery>,
) -> AppResult<Json<Vec<InAppNotification>>> {
    let service = NotificationService::new(state.db);
    let unread_only = query.unread_only.unwrap_or(false);
    let limit = query.limit.unwrap_or(50);
    
    let notifications = service
        .get_in_app_notifications(current_user.0.user_id, unread_only, limit)
        .await?;
    Ok(Json(notifications))
}

/// Get unread notification count
pub async fn get_unread_count(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<UnreadCountResponse>> {
    let service = NotificationService::new(state.db);
    let count = service.get_unread_count(current_user.0.user_id).await?;
    Ok(Json(UnreadCountResponse { count }))
}

/// Unread count response
#[derive(Debug, serde::Serialize)]
pub struct UnreadCountResponse {
    pub count: i64,
}

/// Mark notification as read
pub async fn mark_as_read(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(notification_id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let service = NotificationService::new(state.db);
    service
        .mark_as_read(current_user.0.user_id, notification_id)
        .await?;
    Ok(Json(()))
}

/// Mark all notifications as read
pub async fn mark_all_as_read(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<MarkAllReadResponse>> {
    let service = NotificationService::new(state.db);
    let count = service.mark_all_as_read(current_user.0.user_id).await?;
    Ok(Json(MarkAllReadResponse { marked_count: count }))
}

/// Mark all read response
#[derive(Debug, serde::Serialize)]
pub struct MarkAllReadResponse {
    pub marked_count: i64,
}

/// Dismiss a notification
pub async fn dismiss_notification(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(notification_id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let service = NotificationService::new(state.db);
    service
        .dismiss_notification(current_user.0.user_id, notification_id)
        .await?;
    Ok(Json(()))
}

// ============================================================================
// Notification History
// ============================================================================

/// Query parameters for notification history
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i32>,
}

/// Get notification history
pub async fn get_notification_history(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<HistoryQuery>,
) -> AppResult<Json<Vec<NotificationLogEntry>>> {
    let service = NotificationService::new(state.db);
    let limit = query.limit.unwrap_or(100);
    
    let history = service
        .get_notification_history(current_user.0.user_id, limit)
        .await?;
    Ok(Json(history))
}

// ============================================================================
// Send Notification (Admin/System)
// ============================================================================

/// Send a notification to a user (for testing/admin purposes)
pub async fn send_notification(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(input): Json<CreateNotificationInput>,
) -> AppResult<Json<SendNotificationResponse>> {
    let service = NotificationService::new(state.db);
    
    // Queue the notification
    let queued = service
        .queue_notification(current_user.0.user_id, current_user.0.business_id, input)
        .await?;
    
    match queued {
        Some(notification) => {
            // Send immediately
            let log_entry = service.send_notification(&notification).await?;
            Ok(Json(SendNotificationResponse {
                success: true,
                notification_id: Some(log_entry.id),
                message: "Notification sent".to_string(),
            }))
        }
        None => {
            Ok(Json(SendNotificationResponse {
                success: false,
                notification_id: None,
                message: "Notification type is disabled for this user".to_string(),
            }))
        }
    }
}

/// Send notification response
#[derive(Debug, serde::Serialize)]
pub struct SendNotificationResponse {
    pub success: bool,
    pub notification_id: Option<Uuid>,
    pub message: String,
}

// ============================================================================
// Notification Triggers
// ============================================================================

/// Trigger response
#[derive(Debug, serde::Serialize)]
pub struct TriggerResponse {
    pub notifications_queued: i32,
}

/// Trigger low inventory alerts
pub async fn trigger_inventory_alerts(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<TriggerResponse>> {
    let service = NotificationService::new(state.db);
    let count = service
        .trigger_low_inventory_alerts(current_user.0.business_id)
        .await?;
    Ok(Json(TriggerResponse { notifications_queued: count }))
}

/// Trigger certification expiry alerts
pub async fn trigger_certification_alerts(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<TriggerResponse>> {
    let service = NotificationService::new(state.db);
    let count = service
        .trigger_certification_expiry_alerts(current_user.0.business_id)
        .await?;
    Ok(Json(TriggerResponse { notifications_queued: count }))
}

/// Trigger weather alerts
pub async fn trigger_weather_alerts(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<TriggerResponse>> {
    let service = NotificationService::new(state.db);
    let count = service
        .trigger_weather_alerts(current_user.0.business_id)
        .await?;
    Ok(Json(TriggerResponse { notifications_queued: count }))
}

/// Run all notification triggers
pub async fn run_all_triggers(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<TriggerResponse>> {
    let service = NotificationService::new(state.db);
    let count = service
        .run_all_triggers(current_user.0.business_id)
        .await?;
    Ok(Json(TriggerResponse { notifications_queued: count }))
}

/// Process notification queue
pub async fn process_queue(
    State(state): State<AppState>,
    _current_user: CurrentUser,
) -> AppResult<Json<ProcessQueueResponse>> {
    let service = NotificationService::new(state.db);
    let sent = service.process_notification_queue(100).await?;
    Ok(Json(ProcessQueueResponse { notifications_sent: sent }))
}

/// Process queue response
#[derive(Debug, serde::Serialize)]
pub struct ProcessQueueResponse {
    pub notifications_sent: i32,
}
