//! Notification service for managing LINE and in-app notifications
//!
//! Supports:
//! - Notification preferences per user
//! - LINE messaging integration
//! - In-app notification management
//! - Notification triggers for various events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Notification service for managing notifications
#[derive(Clone)]
pub struct NotificationService {
    db: PgPool,
    line_client: Option<LineMessagingClient>,
}

/// LINE Messaging API client
#[derive(Clone)]
pub struct LineMessagingClient {
    channel_access_token: String,
    http_client: reqwest::Client,
}

/// Notification type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "notification_type", rename_all = "snake_case")]
pub enum NotificationType {
    LowInventory,
    CertificationExpiring,
    ProcessingMilestone,
    WeatherAlert,
    HarvestReminder,
    QualityAlert,
    System,
}

/// Notification channel enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "notification_channel", rename_all = "snake_case")]
pub enum NotificationChannel {
    Line,
    InApp,
    Email,
}

/// Notification status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "notification_status", rename_all = "snake_case")]
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed,
    Read,
}

/// Notification preferences
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct NotificationPreferences {
    pub user_id: Uuid,
    pub line_enabled: bool,
    pub email_enabled: bool,
    pub low_inventory_enabled: bool,
    pub certification_expiring_enabled: bool,
    pub processing_milestone_enabled: bool,
    pub weather_alert_enabled: bool,
    pub harvest_reminder_enabled: bool,
    pub quality_alert_enabled: bool,
}

/// Input for updating notification preferences
#[derive(Debug, Deserialize)]
pub struct UpdatePreferencesInput {
    pub line_enabled: Option<bool>,
    pub email_enabled: Option<bool>,
    pub low_inventory_enabled: Option<bool>,
    pub certification_expiring_enabled: Option<bool>,
    pub processing_milestone_enabled: Option<bool>,
    pub weather_alert_enabled: Option<bool>,
    pub harvest_reminder_enabled: Option<bool>,
    pub quality_alert_enabled: Option<bool>,
}

/// Queued notification
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct QueuedNotification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub business_id: Uuid,
    pub notification_type: NotificationType,
    pub title: String,
    pub title_th: Option<String>,
    pub message: String,
    pub message_th: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub scheduled_at: DateTime<Utc>,
    pub priority: i32,
    pub status: NotificationStatus,
    pub created_at: DateTime<Utc>,
}

/// Notification log entry
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct NotificationLogEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub business_id: Uuid,
    pub notification_type: NotificationType,
    pub channel: NotificationChannel,
    pub title: String,
    pub title_th: Option<String>,
    pub message: String,
    pub message_th: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub status: NotificationStatus,
    pub error_message: Option<String>,
    pub line_message_id: Option<String>,
    pub sent_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// In-app notification
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct InAppNotification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub business_id: Uuid,
    pub notification_type: NotificationType,
    pub title: String,
    pub title_th: Option<String>,
    pub message: String,
    pub message_th: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub action_url: Option<String>,
    pub is_read: bool,
    pub is_dismissed: bool,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

/// Input for creating a notification
#[derive(Debug, Deserialize)]
pub struct CreateNotificationInput {
    pub notification_type: NotificationType,
    pub title: String,
    pub title_th: Option<String>,
    pub message: String,
    pub message_th: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub priority: Option<i32>,
}

/// LINE message types
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum LineMessage {
    #[serde(rename = "text")]
    Text { text: String },
}

/// LINE push message request
#[derive(Debug, Serialize)]
struct LinePushRequest {
    to: String,
    messages: Vec<LineMessage>,
}

/// LINE API response
#[derive(Debug, Deserialize)]
struct LineApiResponse {
    #[serde(default)]
    message: Option<String>,
}

impl LineMessagingClient {
    /// Create a new LINE messaging client
    pub fn new(channel_access_token: String) -> Self {
        Self {
            channel_access_token,
            http_client: reqwest::Client::new(),
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Option<Self> {
        let token = std::env::var("LINE_CHANNEL_ACCESS_TOKEN").ok()?;
        Some(Self::new(token))
    }

    /// Send a push message to a user
    pub async fn send_push_message(
        &self,
        line_user_id: &str,
        message: LineMessage,
    ) -> Result<(), String> {
        let request = LinePushRequest {
            to: line_user_id.to_string(),
            messages: vec![message],
        };

        let response = self
            .http_client
            .post("https://api.line.me/v2/bot/message/push")
            .header("Authorization", format!("Bearer {}", self.channel_access_token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send LINE message: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error: LineApiResponse = response
                .json()
                .await
                .unwrap_or(LineApiResponse { message: Some("Unknown error".to_string()) });
            Err(error.message.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }
}

impl NotificationService {
    /// Create a new NotificationService instance
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            line_client: LineMessagingClient::from_env(),
        }
    }

    /// Create with explicit LINE client
    pub fn with_line_client(db: PgPool, line_client: LineMessagingClient) -> Self {
        Self {
            db,
            line_client: Some(line_client),
        }
    }

    // ========================================================================
    // Notification Preferences
    // ========================================================================

    /// Get notification preferences for a user
    pub async fn get_preferences(&self, user_id: Uuid) -> AppResult<NotificationPreferences> {
        let prefs = sqlx::query_as::<_, NotificationPreferences>(
            r#"
            SELECT user_id, line_enabled, email_enabled,
                   low_inventory_enabled, certification_expiring_enabled,
                   processing_milestone_enabled, weather_alert_enabled,
                   harvest_reminder_enabled, quality_alert_enabled
            FROM notification_preferences
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Notification preferences".to_string()))?;

        Ok(prefs)
    }

    /// Update notification preferences
    pub async fn update_preferences(
        &self,
        user_id: Uuid,
        input: UpdatePreferencesInput,
    ) -> AppResult<NotificationPreferences> {
        let prefs = sqlx::query_as::<_, NotificationPreferences>(
            r#"
            UPDATE notification_preferences SET
                line_enabled = COALESCE($2, line_enabled),
                email_enabled = COALESCE($3, email_enabled),
                low_inventory_enabled = COALESCE($4, low_inventory_enabled),
                certification_expiring_enabled = COALESCE($5, certification_expiring_enabled),
                processing_milestone_enabled = COALESCE($6, processing_milestone_enabled),
                weather_alert_enabled = COALESCE($7, weather_alert_enabled),
                harvest_reminder_enabled = COALESCE($8, harvest_reminder_enabled),
                quality_alert_enabled = COALESCE($9, quality_alert_enabled)
            WHERE user_id = $1
            RETURNING user_id, line_enabled, email_enabled,
                      low_inventory_enabled, certification_expiring_enabled,
                      processing_milestone_enabled, weather_alert_enabled,
                      harvest_reminder_enabled, quality_alert_enabled
            "#,
        )
        .bind(user_id)
        .bind(input.line_enabled)
        .bind(input.email_enabled)
        .bind(input.low_inventory_enabled)
        .bind(input.certification_expiring_enabled)
        .bind(input.processing_milestone_enabled)
        .bind(input.weather_alert_enabled)
        .bind(input.harvest_reminder_enabled)
        .bind(input.quality_alert_enabled)
        .fetch_one(&self.db)
        .await?;

        Ok(prefs)
    }

    /// Check if a notification type is enabled for a user
    pub async fn is_notification_enabled(
        &self,
        user_id: Uuid,
        notification_type: &NotificationType,
    ) -> AppResult<bool> {
        let prefs = self.get_preferences(user_id).await?;

        let enabled = match notification_type {
            NotificationType::LowInventory => prefs.low_inventory_enabled,
            NotificationType::CertificationExpiring => prefs.certification_expiring_enabled,
            NotificationType::ProcessingMilestone => prefs.processing_milestone_enabled,
            NotificationType::WeatherAlert => prefs.weather_alert_enabled,
            NotificationType::HarvestReminder => prefs.harvest_reminder_enabled,
            NotificationType::QualityAlert => prefs.quality_alert_enabled,
            NotificationType::System => true, // System notifications always enabled
        };

        Ok(enabled)
    }

    // ========================================================================
    // Notification Queue
    // ========================================================================

    /// Queue a notification for sending
    pub async fn queue_notification(
        &self,
        user_id: Uuid,
        business_id: Uuid,
        input: CreateNotificationInput,
    ) -> AppResult<Option<QueuedNotification>> {
        // Check if notification type is enabled
        if !self.is_notification_enabled(user_id, &input.notification_type).await? {
            return Ok(None);
        }

        let notification = sqlx::query_as::<_, QueuedNotification>(
            r#"
            INSERT INTO notification_queue (
                user_id, business_id, notification_type,
                title, title_th, message, message_th,
                entity_type, entity_id, priority
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, user_id, business_id, notification_type,
                      title, title_th, message, message_th,
                      entity_type, entity_id, scheduled_at, priority,
                      status, created_at
            "#,
        )
        .bind(user_id)
        .bind(business_id)
        .bind(&input.notification_type)
        .bind(&input.title)
        .bind(&input.title_th)
        .bind(&input.message)
        .bind(&input.message_th)
        .bind(&input.entity_type)
        .bind(input.entity_id)
        .bind(input.priority.unwrap_or(0))
        .fetch_one(&self.db)
        .await?;

        Ok(Some(notification))
    }

    /// Get pending notifications from queue
    pub async fn get_pending_notifications(&self, limit: i32) -> AppResult<Vec<QueuedNotification>> {
        let notifications = sqlx::query_as::<_, QueuedNotification>(
            r#"
            SELECT id, user_id, business_id, notification_type,
                   title, title_th, message, message_th,
                   entity_type, entity_id, scheduled_at, priority,
                   status, created_at
            FROM notification_queue
            WHERE status = 'pending'
              AND scheduled_at <= NOW()
            ORDER BY priority DESC, scheduled_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.db)
        .await?;

        Ok(notifications)
    }

    // ========================================================================
    // Send Notifications
    // ========================================================================

    /// Send a notification (determines channel and sends)
    pub async fn send_notification(
        &self,
        notification: &QueuedNotification,
    ) -> AppResult<NotificationLogEntry> {
        // Determine the channel to use
        let channel = self.get_notification_channel(notification.user_id).await?;

        match channel {
            NotificationChannel::Line => {
                self.send_line_notification(notification).await
            }
            NotificationChannel::InApp => {
                self.send_in_app_notification(notification).await
            }
            NotificationChannel::Email => {
                // Email not implemented yet, fall back to in-app
                self.send_in_app_notification(notification).await
            }
        }
    }

    /// Get the preferred notification channel for a user
    pub async fn get_notification_channel(&self, user_id: Uuid) -> AppResult<NotificationChannel> {
        // Check if LINE is connected and enabled
        let line_info = sqlx::query_as::<_, (bool, Option<String>)>(
            r#"
            SELECT np.line_enabled, lc.line_user_id
            FROM notification_preferences np
            LEFT JOIN line_connections lc ON lc.user_id = np.user_id
            WHERE np.user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        match line_info {
            Some((line_enabled, Some(_line_user_id))) if line_enabled && self.line_client.is_some() => {
                Ok(NotificationChannel::Line)
            }
            _ => Ok(NotificationChannel::InApp),
        }
    }

    /// Send notification via LINE
    async fn send_line_notification(
        &self,
        notification: &QueuedNotification,
    ) -> AppResult<NotificationLogEntry> {
        // Get LINE user ID
        let line_user_id = sqlx::query_scalar::<_, String>(
            "SELECT line_user_id FROM line_connections WHERE user_id = $1",
        )
        .bind(notification.user_id)
        .fetch_optional(&self.db)
        .await?;

        let line_user_id = match line_user_id {
            Some(id) => id,
            None => {
                // Fall back to in-app if LINE not connected
                return self.send_in_app_notification(notification).await;
            }
        };

        // Send via LINE
        let message_text = format!("{}\n\n{}", notification.title, notification.message);
        let message = LineMessage::Text { text: message_text };

        let (status, error_message, line_message_id) = match &self.line_client {
            Some(client) => {
                match client.send_push_message(&line_user_id, message).await {
                    Ok(()) => (NotificationStatus::Sent, None, None),
                    Err(e) => (NotificationStatus::Failed, Some(e), None),
                }
            }
            None => {
                // No LINE client, fall back to in-app
                return self.send_in_app_notification(notification).await;
            }
        };

        // Log the notification
        let log_entry = self.log_notification(
            notification,
            NotificationChannel::Line,
            status,
            error_message,
            line_message_id,
        ).await?;

        // Update queue status
        self.update_queue_status(notification.id, NotificationStatus::Sent).await?;

        // Also create in-app notification
        self.create_in_app_notification(notification).await?;

        Ok(log_entry)
    }

    /// Send notification via in-app
    async fn send_in_app_notification(
        &self,
        notification: &QueuedNotification,
    ) -> AppResult<NotificationLogEntry> {
        // Create in-app notification
        self.create_in_app_notification(notification).await?;

        // Log the notification
        let log_entry = self.log_notification(
            notification,
            NotificationChannel::InApp,
            NotificationStatus::Sent,
            None,
            None,
        ).await?;

        // Update queue status
        self.update_queue_status(notification.id, NotificationStatus::Sent).await?;

        Ok(log_entry)
    }

    /// Create an in-app notification
    async fn create_in_app_notification(
        &self,
        notification: &QueuedNotification,
    ) -> AppResult<InAppNotification> {
        let in_app = sqlx::query_as::<_, InAppNotification>(
            r#"
            INSERT INTO in_app_notifications (
                user_id, business_id, notification_type,
                title, title_th, message, message_th,
                entity_type, entity_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, user_id, business_id, notification_type,
                      title, title_th, message, message_th,
                      entity_type, entity_id, action_url,
                      is_read, is_dismissed, created_at, read_at
            "#,
        )
        .bind(notification.user_id)
        .bind(notification.business_id)
        .bind(&notification.notification_type)
        .bind(&notification.title)
        .bind(&notification.title_th)
        .bind(&notification.message)
        .bind(&notification.message_th)
        .bind(&notification.entity_type)
        .bind(notification.entity_id)
        .fetch_one(&self.db)
        .await?;

        Ok(in_app)
    }

    /// Log a sent notification
    async fn log_notification(
        &self,
        notification: &QueuedNotification,
        channel: NotificationChannel,
        status: NotificationStatus,
        error_message: Option<String>,
        line_message_id: Option<String>,
    ) -> AppResult<NotificationLogEntry> {
        let log_entry = sqlx::query_as::<_, NotificationLogEntry>(
            r#"
            INSERT INTO notification_log (
                user_id, business_id, notification_type, channel,
                title, title_th, message, message_th,
                entity_type, entity_id, status, error_message, line_message_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, user_id, business_id, notification_type, channel,
                      title, title_th, message, message_th,
                      entity_type, entity_id, status, error_message,
                      line_message_id, sent_at, read_at, created_at
            "#,
        )
        .bind(notification.user_id)
        .bind(notification.business_id)
        .bind(&notification.notification_type)
        .bind(&channel)
        .bind(&notification.title)
        .bind(&notification.title_th)
        .bind(&notification.message)
        .bind(&notification.message_th)
        .bind(&notification.entity_type)
        .bind(notification.entity_id)
        .bind(&status)
        .bind(&error_message)
        .bind(&line_message_id)
        .fetch_one(&self.db)
        .await?;

        Ok(log_entry)
    }

    /// Update queue status
    async fn update_queue_status(
        &self,
        queue_id: Uuid,
        status: NotificationStatus,
    ) -> AppResult<()> {
        sqlx::query("UPDATE notification_queue SET status = $2 WHERE id = $1")
            .bind(queue_id)
            .bind(&status)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    // ========================================================================
    // In-App Notifications
    // ========================================================================

    /// Get in-app notifications for a user
    pub async fn get_in_app_notifications(
        &self,
        user_id: Uuid,
        unread_only: bool,
        limit: i32,
    ) -> AppResult<Vec<InAppNotification>> {
        let notifications = if unread_only {
            sqlx::query_as::<_, InAppNotification>(
                r#"
                SELECT id, user_id, business_id, notification_type,
                       title, title_th, message, message_th,
                       entity_type, entity_id, action_url,
                       is_read, is_dismissed, created_at, read_at
                FROM in_app_notifications
                WHERE user_id = $1 AND is_read = false AND is_dismissed = false
                ORDER BY created_at DESC
                LIMIT $2
                "#,
            )
            .bind(user_id)
            .bind(limit)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, InAppNotification>(
                r#"
                SELECT id, user_id, business_id, notification_type,
                       title, title_th, message, message_th,
                       entity_type, entity_id, action_url,
                       is_read, is_dismissed, created_at, read_at
                FROM in_app_notifications
                WHERE user_id = $1 AND is_dismissed = false
                ORDER BY created_at DESC
                LIMIT $2
                "#,
            )
            .bind(user_id)
            .bind(limit)
            .fetch_all(&self.db)
            .await?
        };

        Ok(notifications)
    }

    /// Get unread notification count
    pub async fn get_unread_count(&self, user_id: Uuid) -> AppResult<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM in_app_notifications
            WHERE user_id = $1 AND is_read = false AND is_dismissed = false
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(count)
    }

    /// Mark notification as read
    pub async fn mark_as_read(&self, user_id: Uuid, notification_id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE in_app_notifications
            SET is_read = true, read_at = NOW()
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(notification_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Notification".to_string()));
        }

        Ok(())
    }

    /// Mark all notifications as read
    pub async fn mark_all_as_read(&self, user_id: Uuid) -> AppResult<i64> {
        let result = sqlx::query(
            r#"
            UPDATE in_app_notifications
            SET is_read = true, read_at = NOW()
            WHERE user_id = $1 AND is_read = false
            "#,
        )
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Dismiss a notification
    pub async fn dismiss_notification(&self, user_id: Uuid, notification_id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE in_app_notifications
            SET is_dismissed = true
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(notification_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Notification".to_string()));
        }

        Ok(())
    }

    // ========================================================================
    // Notification History
    // ========================================================================

    /// Get notification history for a user
    pub async fn get_notification_history(
        &self,
        user_id: Uuid,
        limit: i32,
    ) -> AppResult<Vec<NotificationLogEntry>> {
        let history = sqlx::query_as::<_, NotificationLogEntry>(
            r#"
            SELECT id, user_id, business_id, notification_type, channel,
                   title, title_th, message, message_th,
                   entity_type, entity_id, status, error_message,
                   line_message_id, sent_at, read_at, created_at
            FROM notification_log
            WHERE user_id = $1
            ORDER BY sent_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.db)
        .await?;

        Ok(history)
    }
}

// ============================================================================
// Notification Trigger Helpers
// ============================================================================

/// Create a low inventory notification
pub fn create_low_inventory_notification(
    lot_name: &str,
    current_quantity: f64,
    threshold: f64,
    stage: &str,
) -> CreateNotificationInput {
    CreateNotificationInput {
        notification_type: NotificationType::LowInventory,
        title: format!("Low Inventory Alert: {}", lot_name),
        title_th: Some(format!("แจ้งเตือนสินค้าคงคลังต่ำ: {}", lot_name)),
        message: format!(
            "Lot '{}' has fallen below the threshold. Current: {:.2} kg, Threshold: {:.2} kg, Stage: {}",
            lot_name, current_quantity, threshold, stage
        ),
        message_th: Some(format!(
            "ล็อต '{}' มีปริมาณต่ำกว่าเกณฑ์ ปัจจุบัน: {:.2} กก., เกณฑ์: {:.2} กก., ขั้นตอน: {}",
            lot_name, current_quantity, threshold, stage
        )),
        entity_type: Some("lot".to_string()),
        entity_id: None,
        priority: Some(1),
    }
}

/// Create a certification expiring notification
pub fn create_certification_expiring_notification(
    cert_name: &str,
    days_until: i32,
    cert_id: Uuid,
) -> CreateNotificationInput {
    CreateNotificationInput {
        notification_type: NotificationType::CertificationExpiring,
        title: format!("Certification Expiring: {}", cert_name),
        title_th: Some(format!("ใบรับรองใกล้หมดอายุ: {}", cert_name)),
        message: format!(
            "Your certification '{}' will expire in {} days. Please renew to maintain compliance.",
            cert_name, days_until
        ),
        message_th: Some(format!(
            "ใบรับรอง '{}' จะหมดอายุใน {} วัน กรุณาต่ออายุเพื่อรักษาการปฏิบัติตามมาตรฐาน",
            cert_name, days_until
        )),
        entity_type: Some("certification".to_string()),
        entity_id: Some(cert_id),
        priority: Some(if days_until <= 30 { 2 } else { 1 }),
    }
}

/// Create a weather alert notification
pub fn create_weather_alert_notification(
    plot_name: &str,
    alert_message: &str,
    plot_id: Uuid,
) -> CreateNotificationInput {
    CreateNotificationInput {
        notification_type: NotificationType::WeatherAlert,
        title: format!("Weather Alert: {}", plot_name),
        title_th: Some(format!("แจ้งเตือนสภาพอากาศ: {}", plot_name)),
        message: alert_message.to_string(),
        message_th: None,
        entity_type: Some("plot".to_string()),
        entity_id: Some(plot_id),
        priority: Some(2),
    }
}

/// Create a processing milestone notification
pub fn create_processing_milestone_notification(
    lot_name: &str,
    milestone: &str,
    lot_id: Uuid,
) -> CreateNotificationInput {
    CreateNotificationInput {
        notification_type: NotificationType::ProcessingMilestone,
        title: format!("Processing Update: {}", lot_name),
        title_th: Some(format!("อัปเดตการแปรรูป: {}", lot_name)),
        message: format!("Lot '{}' has reached milestone: {}", lot_name, milestone),
        message_th: Some(format!("ล็อต '{}' ถึงขั้นตอน: {}", lot_name, milestone)),
        entity_type: Some("lot".to_string()),
        entity_id: Some(lot_id),
        priority: Some(0),
    }
}

// ============================================================================
// Notification Triggers
// ============================================================================

/// Triggered inventory alert info
#[derive(Debug, Clone)]
pub struct TriggeredInventoryAlert {
    pub alert_id: Uuid,
    pub lot_id: Uuid,
    pub lot_name: String,
    pub stage: String,
    pub current_quantity: f64,
    pub threshold: f64,
    pub user_id: Uuid,
    pub business_id: Uuid,
}

/// Expiring certification info
#[derive(Debug, Clone)]
pub struct ExpiringCertification {
    pub cert_id: Uuid,
    pub cert_name: String,
    pub days_until: i32,
    pub user_id: Uuid,
    pub business_id: Uuid,
}

/// Weather alert trigger info
#[derive(Debug, Clone)]
pub struct WeatherAlertTrigger {
    pub alert_id: Uuid,
    pub plot_id: Uuid,
    pub plot_name: String,
    pub alert_message: String,
    pub user_id: Uuid,
    pub business_id: Uuid,
}

impl NotificationService {
    // ========================================================================
    // Notification Triggers
    // ========================================================================

    /// Trigger notifications for low inventory alerts
    /// Returns the number of notifications queued
    pub async fn trigger_low_inventory_alerts(&self, business_id: Uuid) -> AppResult<i32> {
        // Get triggered inventory alerts
        let alerts = sqlx::query_as::<_, (Uuid, Uuid, String, String, f64, f64, Uuid)>(
            r#"
            SELECT ia.id, ia.lot_id, l.name, ia.stage::text, 
                   COALESCE(get_lot_inventory_balance(ia.lot_id, ia.stage::text), 0)::float8 as current_qty,
                   ia.threshold_kg::float8,
                   b.owner_id
            FROM inventory_alerts ia
            JOIN lots l ON l.id = ia.lot_id
            JOIN businesses b ON b.id = ia.business_id
            WHERE ia.business_id = $1
              AND ia.is_active = true
              AND COALESCE(get_lot_inventory_balance(ia.lot_id, ia.stage::text), 0) <= ia.threshold_kg
              AND (ia.last_triggered_at IS NULL OR ia.last_triggered_at < NOW() - INTERVAL '24 hours')
            "#,
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        let mut count = 0;
        for (alert_id, lot_id, lot_name, stage, current_qty, threshold, user_id) in alerts {
            let notification = create_low_inventory_notification(
                &lot_name,
                current_qty,
                threshold,
                &stage,
            );

            // Queue the notification
            if let Some(_) = self.queue_notification(user_id, business_id, notification).await? {
                // Update last triggered time
                sqlx::query("UPDATE inventory_alerts SET last_triggered_at = NOW() WHERE id = $1")
                    .bind(alert_id)
                    .execute(&self.db)
                    .await?;
                count += 1;
            }
        }

        Ok(count)
    }

    /// Trigger notifications for expiring certifications
    /// Returns the number of notifications queued
    pub async fn trigger_certification_expiry_alerts(&self, business_id: Uuid) -> AppResult<i32> {
        // Get certifications expiring within 90 days that haven't been notified recently
        let certs = sqlx::query_as::<_, (Uuid, String, i32, Uuid)>(
            r#"
            SELECT c.id, c.certification_name, 
                   (c.expiration_date - CURRENT_DATE)::int as days_until,
                   b.owner_id
            FROM certifications c
            JOIN businesses b ON b.id = c.business_id
            LEFT JOIN certification_alerts ca ON ca.certification_id = c.id
            WHERE c.business_id = $1
              AND c.is_active = true
              AND c.expiration_date > CURRENT_DATE
              AND c.expiration_date <= CURRENT_DATE + INTERVAL '90 days'
              AND (
                  ca.id IS NULL 
                  OR (
                      (c.expiration_date - CURRENT_DATE <= 30 AND ca.alert_30_days_sent = false)
                      OR (c.expiration_date - CURRENT_DATE <= 60 AND c.expiration_date - CURRENT_DATE > 30 AND ca.alert_60_days_sent = false)
                      OR (c.expiration_date - CURRENT_DATE <= 90 AND c.expiration_date - CURRENT_DATE > 60 AND ca.alert_90_days_sent = false)
                  )
              )
            "#,
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        let mut count = 0;
        for (cert_id, cert_name, days_until, user_id) in certs {
            let notification = create_certification_expiring_notification(
                &cert_name,
                days_until,
                cert_id,
            );

            // Queue the notification
            if let Some(_) = self.queue_notification(user_id, business_id, notification).await? {
                // Update alert tracking
                let alert_column = if days_until <= 30 {
                    "alert_30_days_sent"
                } else if days_until <= 60 {
                    "alert_60_days_sent"
                } else {
                    "alert_90_days_sent"
                };

                sqlx::query(&format!(
                    r#"
                    INSERT INTO certification_alerts (certification_id, {})
                    VALUES ($1, true)
                    ON CONFLICT (certification_id) DO UPDATE SET {} = true
                    "#,
                    alert_column, alert_column
                ))
                .bind(cert_id)
                .execute(&self.db)
                .await?;

                count += 1;
            }
        }

        Ok(count)
    }

    /// Trigger notifications for weather alerts
    /// Returns the number of notifications queued
    pub async fn trigger_weather_alerts(&self, business_id: Uuid) -> AppResult<i32> {
        // Get active weather alerts that have been triggered
        let alerts = sqlx::query_as::<_, (Uuid, Uuid, String, String, Uuid)>(
            r#"
            SELECT wa.id, wa.plot_id, p.name, 
                   COALESCE(wa.alert_type, 'rain') || ' alert for ' || p.name as alert_message,
                   b.owner_id
            FROM weather_alerts wa
            JOIN plots p ON p.id = wa.plot_id
            JOIN businesses b ON b.id = wa.business_id
            WHERE wa.business_id = $1
              AND wa.is_active = true
              AND wa.notify_line = true
              AND (wa.last_triggered_at IS NULL OR wa.last_triggered_at < NOW() - INTERVAL '6 hours')
            "#,
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        let mut count = 0;
        for (alert_id, plot_id, plot_name, alert_message, user_id) in alerts {
            let notification = create_weather_alert_notification(
                &plot_name,
                &alert_message,
                plot_id,
            );

            // Queue the notification
            if let Some(_) = self.queue_notification(user_id, business_id, notification).await? {
                // Update last triggered time
                sqlx::query("UPDATE weather_alerts SET last_triggered_at = NOW() WHERE id = $1")
                    .bind(alert_id)
                    .execute(&self.db)
                    .await?;
                count += 1;
            }
        }

        Ok(count)
    }

    /// Trigger notification for processing milestone
    pub async fn trigger_processing_milestone(
        &self,
        user_id: Uuid,
        business_id: Uuid,
        lot_id: Uuid,
        lot_name: &str,
        milestone: &str,
    ) -> AppResult<Option<QueuedNotification>> {
        let notification = create_processing_milestone_notification(lot_name, milestone, lot_id);
        self.queue_notification(user_id, business_id, notification).await
    }

    /// Process all pending notifications in the queue
    /// Returns the number of notifications sent
    pub async fn process_notification_queue(&self, batch_size: i32) -> AppResult<i32> {
        let pending = self.get_pending_notifications(batch_size).await?;
        let mut sent_count = 0;

        for notification in pending {
            match self.send_notification(&notification).await {
                Ok(_) => sent_count += 1,
                Err(e) => {
                    // Log error but continue processing
                    tracing::error!("Failed to send notification {}: {}", notification.id, e);
                    // Mark as failed
                    self.update_queue_status(notification.id, NotificationStatus::Failed).await?;
                }
            }
        }

        Ok(sent_count)
    }

    /// Run all notification triggers for a business
    /// Returns total notifications queued
    pub async fn run_all_triggers(&self, business_id: Uuid) -> AppResult<i32> {
        let mut total = 0;

        // Trigger inventory alerts
        total += self.trigger_low_inventory_alerts(business_id).await?;

        // Trigger certification expiry alerts
        total += self.trigger_certification_expiry_alerts(business_id).await?;

        // Trigger weather alerts
        total += self.trigger_weather_alerts(business_id).await?;

        Ok(total)
    }
}
