//! Roast profile management service for coffee roasting operations

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::services::lot::LotStage;

/// Roasting service for managing roast sessions and profile templates
#[derive(Clone)]
pub struct RoastingService {
    db: PgPool,
}

/// Roast session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoastStatus {
    InProgress,
    Completed,
    Failed,
}

impl RoastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoastStatus::InProgress => "in_progress",
            RoastStatus::Completed => "completed",
            RoastStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "in_progress" => Some(RoastStatus::InProgress),
            "completed" => Some(RoastStatus::Completed),
            "failed" => Some(RoastStatus::Failed),
            _ => None,
        }
    }
}

/// Roast level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoastLevel {
    Light,
    MediumLight,
    Medium,
    MediumDark,
    Dark,
}

impl RoastLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoastLevel::Light => "light",
            RoastLevel::MediumLight => "medium_light",
            RoastLevel::Medium => "medium",
            RoastLevel::MediumDark => "medium_dark",
            RoastLevel::Dark => "dark",
        }
    }
}

/// Temperature checkpoint in roast profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureCheckpoint {
    pub time_seconds: i32,
    pub temp_celsius: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Roast profile template
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct RoastProfileTemplate {
    pub id: Uuid,
    pub business_id: Uuid,
    pub name: String,
    pub name_th: Option<String>,
    pub description: Option<String>,
    pub description_th: Option<String>,
    pub target_first_crack_time_seconds: Option<i32>,
    pub target_first_crack_temp_celsius: Option<Decimal>,
    pub target_development_time_seconds: Option<i32>,
    pub target_end_temp_celsius: Option<Decimal>,
    pub target_total_time_seconds: Option<i32>,
    pub target_weight_loss_percent: Option<Decimal>,
    pub temperature_profile: Option<serde_json::Value>,
    pub roast_level: Option<String>,
    pub recommended_equipment: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

/// Input for creating a roast profile template
#[derive(Debug, Deserialize)]
pub struct CreateTemplateInput {
    pub name: String,
    pub name_th: Option<String>,
    pub description: Option<String>,
    pub description_th: Option<String>,
    pub target_first_crack_time_seconds: Option<i32>,
    pub target_first_crack_temp_celsius: Option<Decimal>,
    pub target_development_time_seconds: Option<i32>,
    pub target_end_temp_celsius: Option<Decimal>,
    pub target_total_time_seconds: Option<i32>,
    pub target_weight_loss_percent: Option<Decimal>,
    pub temperature_profile: Option<Vec<TemperatureCheckpoint>>,
    pub roast_level: Option<RoastLevel>,
    pub recommended_equipment: Option<String>,
}

/// Input for updating a roast profile template
#[derive(Debug, Deserialize)]
pub struct UpdateTemplateInput {
    pub name: Option<String>,
    pub name_th: Option<String>,
    pub description: Option<String>,
    pub description_th: Option<String>,
    pub target_first_crack_time_seconds: Option<i32>,
    pub target_first_crack_temp_celsius: Option<Decimal>,
    pub target_development_time_seconds: Option<i32>,
    pub target_end_temp_celsius: Option<Decimal>,
    pub target_total_time_seconds: Option<i32>,
    pub target_weight_loss_percent: Option<Decimal>,
    pub temperature_profile: Option<Vec<TemperatureCheckpoint>>,
    pub roast_level: Option<RoastLevel>,
    pub recommended_equipment: Option<String>,
    pub is_active: Option<bool>,
}


/// Roast session record
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct RoastSession {
    pub id: Uuid,
    pub business_id: Uuid,
    pub lot_id: Uuid,
    pub template_id: Option<Uuid>,
    pub session_date: NaiveDate,
    pub roaster_name: String,
    pub equipment: Option<String>,
    pub green_bean_weight_kg: Decimal,
    pub initial_moisture_percent: Option<Decimal>,
    pub temperature_log: Option<serde_json::Value>,
    pub charge_temp_celsius: Option<Decimal>,
    pub turning_point_time_seconds: Option<i32>,
    pub turning_point_temp_celsius: Option<Decimal>,
    pub first_crack_time_seconds: Option<i32>,
    pub first_crack_temp_celsius: Option<Decimal>,
    pub second_crack_time_seconds: Option<i32>,
    pub second_crack_temp_celsius: Option<Decimal>,
    pub drop_time_seconds: Option<i32>,
    pub drop_temp_celsius: Option<Decimal>,
    pub roasted_weight_kg: Option<Decimal>,
    pub weight_loss_percent: Option<Decimal>,
    pub final_moisture_percent: Option<Decimal>,
    pub development_time_seconds: Option<i32>,
    pub development_time_ratio: Option<Decimal>,
    pub roast_level: Option<String>,
    pub color_value: Option<Decimal>,
    pub status: String,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
}

/// Input for starting a roast session
#[derive(Debug, Deserialize)]
pub struct StartRoastSessionInput {
    pub lot_id: Uuid,
    pub template_id: Option<Uuid>,
    pub session_date: NaiveDate,
    pub roaster_name: String,
    pub equipment: Option<String>,
    pub green_bean_weight_kg: Decimal,
    pub initial_moisture_percent: Option<Decimal>,
    pub charge_temp_celsius: Option<Decimal>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
}

/// Input for logging temperature checkpoint
#[derive(Debug, Deserialize)]
pub struct LogTemperatureInput {
    pub checkpoints: Vec<TemperatureCheckpoint>,
}

/// Input for logging roast milestones
#[derive(Debug, Deserialize)]
pub struct LogMilestonesInput {
    pub turning_point_time_seconds: Option<i32>,
    pub turning_point_temp_celsius: Option<Decimal>,
    pub first_crack_time_seconds: Option<i32>,
    pub first_crack_temp_celsius: Option<Decimal>,
    pub second_crack_time_seconds: Option<i32>,
    pub second_crack_temp_celsius: Option<Decimal>,
}

/// Input for completing a roast session
#[derive(Debug, Deserialize)]
pub struct CompleteRoastInput {
    pub drop_time_seconds: i32,
    pub drop_temp_celsius: Decimal,
    pub roasted_weight_kg: Decimal,
    pub final_moisture_percent: Option<Decimal>,
    pub roast_level: Option<RoastLevel>,
    pub color_value: Option<Decimal>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
}

impl RoastingService {
    /// Create a new RoastingService instance
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    // ========================================================================
    // Profile Template Methods
    // ========================================================================

    /// Create a new roast profile template
    pub async fn create_template(
        &self,
        business_id: Uuid,
        user_id: Uuid,
        input: CreateTemplateInput,
    ) -> AppResult<RoastProfileTemplate> {
        // Validate name
        if input.name.trim().is_empty() {
            return Err(AppError::Validation {
                field: "name".to_string(),
                message: "Template name is required".to_string(),
                message_th: "ต้องระบุชื่อโปรไฟล์".to_string(),
            });
        }

        let temp_profile = input.temperature_profile.map(|tp| {
            serde_json::to_value(&tp).unwrap_or(serde_json::Value::Array(vec![]))
        });

        let roast_level = input.roast_level.map(|rl| rl.as_str().to_string());

        let template = sqlx::query_as::<_, RoastProfileTemplate>(
            r#"
            INSERT INTO roast_profile_templates (
                business_id, name, name_th, description, description_th,
                target_first_crack_time_seconds, target_first_crack_temp_celsius,
                target_development_time_seconds, target_end_temp_celsius,
                target_total_time_seconds, target_weight_loss_percent,
                temperature_profile, roast_level, recommended_equipment, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id, business_id, name, name_th, description, description_th,
                      target_first_crack_time_seconds, target_first_crack_temp_celsius,
                      target_development_time_seconds, target_end_temp_celsius,
                      target_total_time_seconds, target_weight_loss_percent,
                      temperature_profile, roast_level, recommended_equipment,
                      is_active, created_at, updated_at, created_by
            "#,
        )
        .bind(business_id)
        .bind(&input.name)
        .bind(&input.name_th)
        .bind(&input.description)
        .bind(&input.description_th)
        .bind(input.target_first_crack_time_seconds)
        .bind(input.target_first_crack_temp_celsius)
        .bind(input.target_development_time_seconds)
        .bind(input.target_end_temp_celsius)
        .bind(input.target_total_time_seconds)
        .bind(input.target_weight_loss_percent)
        .bind(&temp_profile)
        .bind(&roast_level)
        .bind(&input.recommended_equipment)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(template)
    }

    /// Get a roast profile template by ID
    pub async fn get_template(
        &self,
        business_id: Uuid,
        template_id: Uuid,
    ) -> AppResult<RoastProfileTemplate> {
        let template = sqlx::query_as::<_, RoastProfileTemplate>(
            r#"
            SELECT id, business_id, name, name_th, description, description_th,
                   target_first_crack_time_seconds, target_first_crack_temp_celsius,
                   target_development_time_seconds, target_end_temp_celsius,
                   target_total_time_seconds, target_weight_loss_percent,
                   temperature_profile, roast_level, recommended_equipment,
                   is_active, created_at, updated_at, created_by
            FROM roast_profile_templates
            WHERE id = $1 AND business_id = $2
            "#,
        )
        .bind(template_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Roast profile template".to_string()))?;

        Ok(template)
    }

    /// List all roast profile templates for a business
    pub async fn list_templates(
        &self,
        business_id: Uuid,
        active_only: bool,
    ) -> AppResult<Vec<RoastProfileTemplate>> {
        let templates = if active_only {
            sqlx::query_as::<_, RoastProfileTemplate>(
                r#"
                SELECT id, business_id, name, name_th, description, description_th,
                       target_first_crack_time_seconds, target_first_crack_temp_celsius,
                       target_development_time_seconds, target_end_temp_celsius,
                       target_total_time_seconds, target_weight_loss_percent,
                       temperature_profile, roast_level, recommended_equipment,
                       is_active, created_at, updated_at, created_by
                FROM roast_profile_templates
                WHERE business_id = $1 AND is_active = true
                ORDER BY name
                "#,
            )
            .bind(business_id)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, RoastProfileTemplate>(
                r#"
                SELECT id, business_id, name, name_th, description, description_th,
                       target_first_crack_time_seconds, target_first_crack_temp_celsius,
                       target_development_time_seconds, target_end_temp_celsius,
                       target_total_time_seconds, target_weight_loss_percent,
                       temperature_profile, roast_level, recommended_equipment,
                       is_active, created_at, updated_at, created_by
                FROM roast_profile_templates
                WHERE business_id = $1
                ORDER BY name
                "#,
            )
            .bind(business_id)
            .fetch_all(&self.db)
            .await?
        };

        Ok(templates)
    }

    /// Update a roast profile template
    pub async fn update_template(
        &self,
        business_id: Uuid,
        template_id: Uuid,
        input: UpdateTemplateInput,
    ) -> AppResult<RoastProfileTemplate> {
        // Check if template exists
        let existing = self.get_template(business_id, template_id).await?;

        let name = input.name.unwrap_or(existing.name);
        let name_th = input.name_th.or(existing.name_th);
        let description = input.description.or(existing.description);
        let description_th = input.description_th.or(existing.description_th);
        let target_first_crack_time = input
            .target_first_crack_time_seconds
            .or(existing.target_first_crack_time_seconds);
        let target_first_crack_temp = input
            .target_first_crack_temp_celsius
            .or(existing.target_first_crack_temp_celsius);
        let target_dev_time = input
            .target_development_time_seconds
            .or(existing.target_development_time_seconds);
        let target_end_temp = input
            .target_end_temp_celsius
            .or(existing.target_end_temp_celsius);
        let target_total_time = input
            .target_total_time_seconds
            .or(existing.target_total_time_seconds);
        let target_weight_loss = input
            .target_weight_loss_percent
            .or(existing.target_weight_loss_percent);
        let temp_profile = input
            .temperature_profile
            .map(|tp| serde_json::to_value(&tp).unwrap_or(serde_json::Value::Array(vec![])))
            .or(existing.temperature_profile);
        let roast_level = input
            .roast_level
            .map(|rl| rl.as_str().to_string())
            .or(existing.roast_level);
        let recommended_equipment = input.recommended_equipment.or(existing.recommended_equipment);
        let is_active = input.is_active.unwrap_or(existing.is_active);

        let template = sqlx::query_as::<_, RoastProfileTemplate>(
            r#"
            UPDATE roast_profile_templates
            SET name = $1, name_th = $2, description = $3, description_th = $4,
                target_first_crack_time_seconds = $5, target_first_crack_temp_celsius = $6,
                target_development_time_seconds = $7, target_end_temp_celsius = $8,
                target_total_time_seconds = $9, target_weight_loss_percent = $10,
                temperature_profile = $11, roast_level = $12, recommended_equipment = $13,
                is_active = $14
            WHERE id = $15
            RETURNING id, business_id, name, name_th, description, description_th,
                      target_first_crack_time_seconds, target_first_crack_temp_celsius,
                      target_development_time_seconds, target_end_temp_celsius,
                      target_total_time_seconds, target_weight_loss_percent,
                      temperature_profile, roast_level, recommended_equipment,
                      is_active, created_at, updated_at, created_by
            "#,
        )
        .bind(&name)
        .bind(&name_th)
        .bind(&description)
        .bind(&description_th)
        .bind(target_first_crack_time)
        .bind(target_first_crack_temp)
        .bind(target_dev_time)
        .bind(target_end_temp)
        .bind(target_total_time)
        .bind(target_weight_loss)
        .bind(&temp_profile)
        .bind(&roast_level)
        .bind(&recommended_equipment)
        .bind(is_active)
        .bind(template_id)
        .fetch_one(&self.db)
        .await?;

        Ok(template)
    }

    /// Delete a roast profile template (soft delete by setting is_active = false)
    pub async fn delete_template(&self, business_id: Uuid, template_id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            "UPDATE roast_profile_templates SET is_active = false WHERE id = $1 AND business_id = $2",
        )
        .bind(template_id)
        .bind(business_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Roast profile template".to_string()));
        }

        Ok(())
    }
}


impl RoastingService {
    // ========================================================================
    // Roast Session Methods
    // ========================================================================

    /// Start a new roast session
    pub async fn start_session(
        &self,
        business_id: Uuid,
        user_id: Uuid,
        input: StartRoastSessionInput,
    ) -> AppResult<RoastSession> {
        // Validate lot exists and belongs to business
        let lot = sqlx::query_as::<_, (Uuid, String, Decimal)>(
            "SELECT id, stage, current_weight_kg FROM lots WHERE id = $1 AND business_id = $2",
        )
        .bind(input.lot_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Lot".to_string()))?;

        // Validate lot is in GreenBean stage
        if lot.1 != LotStage::GreenBean.as_str() {
            return Err(AppError::Validation {
                field: "lot_id".to_string(),
                message: format!(
                    "Lot must be in GreenBean stage to start roasting, current stage: {}",
                    lot.1
                ),
                message_th: format!(
                    "ล็อตต้องอยู่ในสถานะกาแฟกะลาเพื่อเริ่มการคั่ว สถานะปัจจุบัน: {}",
                    lot.1
                ),
            });
        }

        // Validate green bean weight
        if input.green_bean_weight_kg <= Decimal::ZERO {
            return Err(AppError::Validation {
                field: "green_bean_weight_kg".to_string(),
                message: "Green bean weight must be positive".to_string(),
                message_th: "น้ำหนักกาแฟกะลาต้องเป็นค่าบวก".to_string(),
            });
        }

        // Validate roaster name
        if input.roaster_name.trim().is_empty() {
            return Err(AppError::Validation {
                field: "roaster_name".to_string(),
                message: "Roaster name is required".to_string(),
                message_th: "ต้องระบุชื่อผู้คั่ว".to_string(),
            });
        }

        // Validate template if provided
        if let Some(template_id) = input.template_id {
            let template_exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM roast_profile_templates WHERE id = $1 AND business_id = $2 AND is_active = true)"
            )
            .bind(template_id)
            .bind(business_id)
            .fetch_one(&self.db)
            .await?;

            if !template_exists {
                return Err(AppError::NotFound("Roast profile template".to_string()));
            }
        }

        let session = sqlx::query_as::<_, RoastSession>(
            r#"
            INSERT INTO roast_sessions (
                business_id, lot_id, template_id, session_date, roaster_name,
                equipment, green_bean_weight_kg, initial_moisture_percent,
                charge_temp_celsius, notes, notes_th, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, business_id, lot_id, template_id, session_date, roaster_name,
                      equipment, green_bean_weight_kg, initial_moisture_percent,
                      temperature_log, charge_temp_celsius,
                      turning_point_time_seconds, turning_point_temp_celsius,
                      first_crack_time_seconds, first_crack_temp_celsius,
                      second_crack_time_seconds, second_crack_temp_celsius,
                      drop_time_seconds, drop_temp_celsius,
                      roasted_weight_kg, weight_loss_percent, final_moisture_percent,
                      development_time_seconds, development_time_ratio,
                      roast_level, color_value, status, notes, notes_th,
                      created_at, updated_at, completed_at, created_by
            "#,
        )
        .bind(business_id)
        .bind(input.lot_id)
        .bind(input.template_id)
        .bind(input.session_date)
        .bind(&input.roaster_name)
        .bind(&input.equipment)
        .bind(input.green_bean_weight_kg)
        .bind(input.initial_moisture_percent)
        .bind(input.charge_temp_celsius)
        .bind(&input.notes)
        .bind(&input.notes_th)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(session)
    }

    /// Get a roast session by ID
    pub async fn get_session(
        &self,
        business_id: Uuid,
        session_id: Uuid,
    ) -> AppResult<RoastSession> {
        let session = sqlx::query_as::<_, RoastSession>(
            r#"
            SELECT id, business_id, lot_id, template_id, session_date, roaster_name,
                   equipment, green_bean_weight_kg, initial_moisture_percent,
                   temperature_log, charge_temp_celsius,
                   turning_point_time_seconds, turning_point_temp_celsius,
                   first_crack_time_seconds, first_crack_temp_celsius,
                   second_crack_time_seconds, second_crack_temp_celsius,
                   drop_time_seconds, drop_temp_celsius,
                   roasted_weight_kg, weight_loss_percent, final_moisture_percent,
                   development_time_seconds, development_time_ratio,
                   roast_level, color_value, status, notes, notes_th,
                   created_at, updated_at, completed_at, created_by
            FROM roast_sessions
            WHERE id = $1 AND business_id = $2
            "#,
        )
        .bind(session_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Roast session".to_string()))?;

        Ok(session)
    }

    /// List roast sessions for a business
    pub async fn list_sessions(&self, business_id: Uuid) -> AppResult<Vec<RoastSession>> {
        let sessions = sqlx::query_as::<_, RoastSession>(
            r#"
            SELECT id, business_id, lot_id, template_id, session_date, roaster_name,
                   equipment, green_bean_weight_kg, initial_moisture_percent,
                   temperature_log, charge_temp_celsius,
                   turning_point_time_seconds, turning_point_temp_celsius,
                   first_crack_time_seconds, first_crack_temp_celsius,
                   second_crack_time_seconds, second_crack_temp_celsius,
                   drop_time_seconds, drop_temp_celsius,
                   roasted_weight_kg, weight_loss_percent, final_moisture_percent,
                   development_time_seconds, development_time_ratio,
                   roast_level, color_value, status, notes, notes_th,
                   created_at, updated_at, completed_at, created_by
            FROM roast_sessions
            WHERE business_id = $1
            ORDER BY session_date DESC, created_at DESC
            "#,
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(sessions)
    }

    /// Get roast sessions for a lot
    pub async fn get_sessions_by_lot(
        &self,
        business_id: Uuid,
        lot_id: Uuid,
    ) -> AppResult<Vec<RoastSession>> {
        // Validate lot belongs to business
        let lot_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM lots WHERE id = $1 AND business_id = $2)",
        )
        .bind(lot_id)
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        if !lot_exists {
            return Err(AppError::NotFound("Lot".to_string()));
        }

        let sessions = sqlx::query_as::<_, RoastSession>(
            r#"
            SELECT id, business_id, lot_id, template_id, session_date, roaster_name,
                   equipment, green_bean_weight_kg, initial_moisture_percent,
                   temperature_log, charge_temp_celsius,
                   turning_point_time_seconds, turning_point_temp_celsius,
                   first_crack_time_seconds, first_crack_temp_celsius,
                   second_crack_time_seconds, second_crack_temp_celsius,
                   drop_time_seconds, drop_temp_celsius,
                   roasted_weight_kg, weight_loss_percent, final_moisture_percent,
                   development_time_seconds, development_time_ratio,
                   roast_level, color_value, status, notes, notes_th,
                   created_at, updated_at, completed_at, created_by
            FROM roast_sessions
            WHERE lot_id = $1 AND business_id = $2
            ORDER BY session_date DESC, created_at DESC
            "#,
        )
        .bind(lot_id)
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(sessions)
    }

    /// Log temperature checkpoints
    pub async fn log_temperature(
        &self,
        business_id: Uuid,
        session_id: Uuid,
        input: LogTemperatureInput,
    ) -> AppResult<RoastSession> {
        // Validate session exists and is in progress
        let session = self.get_session(business_id, session_id).await?;
        
        if session.status != RoastStatus::InProgress.as_str() {
            return Err(AppError::Validation {
                field: "session_id".to_string(),
                message: "Cannot log temperature for completed or failed session".to_string(),
                message_th: "ไม่สามารถบันทึกอุณหภูมิสำหรับเซสชันที่เสร็จสิ้นหรือล้มเหลว".to_string(),
            });
        }

        // Merge with existing temperature log
        let mut existing_log: Vec<TemperatureCheckpoint> = session
            .temperature_log
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        existing_log.extend(input.checkpoints);

        // Sort by time
        existing_log.sort_by_key(|c| c.time_seconds);

        let temp_log_json = serde_json::to_value(&existing_log)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let updated = sqlx::query_as::<_, RoastSession>(
            r#"
            UPDATE roast_sessions
            SET temperature_log = $1
            WHERE id = $2
            RETURNING id, business_id, lot_id, template_id, session_date, roaster_name,
                      equipment, green_bean_weight_kg, initial_moisture_percent,
                      temperature_log, charge_temp_celsius,
                      turning_point_time_seconds, turning_point_temp_celsius,
                      first_crack_time_seconds, first_crack_temp_celsius,
                      second_crack_time_seconds, second_crack_temp_celsius,
                      drop_time_seconds, drop_temp_celsius,
                      roasted_weight_kg, weight_loss_percent, final_moisture_percent,
                      development_time_seconds, development_time_ratio,
                      roast_level, color_value, status, notes, notes_th,
                      created_at, updated_at, completed_at, created_by
            "#,
        )
        .bind(&temp_log_json)
        .bind(session_id)
        .fetch_one(&self.db)
        .await?;

        Ok(updated)
    }

    /// Log roast milestones (turning point, first crack, second crack)
    pub async fn log_milestones(
        &self,
        business_id: Uuid,
        session_id: Uuid,
        input: LogMilestonesInput,
    ) -> AppResult<RoastSession> {
        // Validate session exists and is in progress
        let session = self.get_session(business_id, session_id).await?;
        
        if session.status != RoastStatus::InProgress.as_str() {
            return Err(AppError::Validation {
                field: "session_id".to_string(),
                message: "Cannot log milestones for completed or failed session".to_string(),
                message_th: "ไม่สามารถบันทึกจุดสำคัญสำหรับเซสชันที่เสร็จสิ้นหรือล้มเหลว".to_string(),
            });
        }

        let updated = sqlx::query_as::<_, RoastSession>(
            r#"
            UPDATE roast_sessions
            SET turning_point_time_seconds = COALESCE($1, turning_point_time_seconds),
                turning_point_temp_celsius = COALESCE($2, turning_point_temp_celsius),
                first_crack_time_seconds = COALESCE($3, first_crack_time_seconds),
                first_crack_temp_celsius = COALESCE($4, first_crack_temp_celsius),
                second_crack_time_seconds = COALESCE($5, second_crack_time_seconds),
                second_crack_temp_celsius = COALESCE($6, second_crack_temp_celsius)
            WHERE id = $7
            RETURNING id, business_id, lot_id, template_id, session_date, roaster_name,
                      equipment, green_bean_weight_kg, initial_moisture_percent,
                      temperature_log, charge_temp_celsius,
                      turning_point_time_seconds, turning_point_temp_celsius,
                      first_crack_time_seconds, first_crack_temp_celsius,
                      second_crack_time_seconds, second_crack_temp_celsius,
                      drop_time_seconds, drop_temp_celsius,
                      roasted_weight_kg, weight_loss_percent, final_moisture_percent,
                      development_time_seconds, development_time_ratio,
                      roast_level, color_value, status, notes, notes_th,
                      created_at, updated_at, completed_at, created_by
            "#,
        )
        .bind(input.turning_point_time_seconds)
        .bind(input.turning_point_temp_celsius)
        .bind(input.first_crack_time_seconds)
        .bind(input.first_crack_temp_celsius)
        .bind(input.second_crack_time_seconds)
        .bind(input.second_crack_temp_celsius)
        .bind(session_id)
        .fetch_one(&self.db)
        .await?;

        Ok(updated)
    }
}


impl RoastingService {
    /// Complete a roast session
    pub async fn complete_session(
        &self,
        business_id: Uuid,
        session_id: Uuid,
        input: CompleteRoastInput,
    ) -> AppResult<RoastSession> {
        // Validate session exists and is in progress
        let session = self.get_session(business_id, session_id).await?;
        
        if session.status != RoastStatus::InProgress.as_str() {
            return Err(AppError::Validation {
                field: "session_id".to_string(),
                message: "Session is not in progress".to_string(),
                message_th: "เซสชันไม่ได้อยู่ในสถานะกำลังดำเนินการ".to_string(),
            });
        }

        // Validate roasted weight
        if input.roasted_weight_kg <= Decimal::ZERO {
            return Err(AppError::Validation {
                field: "roasted_weight_kg".to_string(),
                message: "Roasted weight must be positive".to_string(),
                message_th: "น้ำหนักกาแฟคั่วต้องเป็นค่าบวก".to_string(),
            });
        }

        // Validate roasted weight is less than green bean weight
        if input.roasted_weight_kg >= session.green_bean_weight_kg {
            return Err(AppError::Validation {
                field: "roasted_weight_kg".to_string(),
                message: "Roasted weight must be less than green bean weight".to_string(),
                message_th: "น้ำหนักกาแฟคั่วต้องน้อยกว่าน้ำหนักกาแฟกะลา".to_string(),
            });
        }

        // Calculate weight loss percentage
        let weight_loss_percent =
            calculate_weight_loss(session.green_bean_weight_kg, input.roasted_weight_kg);

        // Calculate development time and DTR if first crack was recorded
        let (development_time, dtr) = if let Some(fc_time) = session.first_crack_time_seconds {
            let dev_time = input.drop_time_seconds - fc_time;
            let dtr = calculate_dtr(dev_time, input.drop_time_seconds);
            (Some(dev_time), Some(dtr))
        } else {
            (None, None)
        };

        let roast_level = input.roast_level.map(|rl| rl.as_str().to_string());

        // Start transaction
        let mut tx = self.db.begin().await?;

        // Update session
        let updated = sqlx::query_as::<_, RoastSession>(
            r#"
            UPDATE roast_sessions
            SET drop_time_seconds = $1, drop_temp_celsius = $2,
                roasted_weight_kg = $3, weight_loss_percent = $4,
                final_moisture_percent = $5, development_time_seconds = $6,
                development_time_ratio = $7, roast_level = $8, color_value = $9,
                status = $10, notes = COALESCE($11, notes), notes_th = COALESCE($12, notes_th),
                completed_at = NOW()
            WHERE id = $13
            RETURNING id, business_id, lot_id, template_id, session_date, roaster_name,
                      equipment, green_bean_weight_kg, initial_moisture_percent,
                      temperature_log, charge_temp_celsius,
                      turning_point_time_seconds, turning_point_temp_celsius,
                      first_crack_time_seconds, first_crack_temp_celsius,
                      second_crack_time_seconds, second_crack_temp_celsius,
                      drop_time_seconds, drop_temp_celsius,
                      roasted_weight_kg, weight_loss_percent, final_moisture_percent,
                      development_time_seconds, development_time_ratio,
                      roast_level, color_value, status, notes, notes_th,
                      created_at, updated_at, completed_at, created_by
            "#,
        )
        .bind(input.drop_time_seconds)
        .bind(input.drop_temp_celsius)
        .bind(input.roasted_weight_kg)
        .bind(weight_loss_percent)
        .bind(input.final_moisture_percent)
        .bind(development_time)
        .bind(dtr)
        .bind(&roast_level)
        .bind(input.color_value)
        .bind(RoastStatus::Completed.as_str())
        .bind(&input.notes)
        .bind(&input.notes_th)
        .bind(session_id)
        .fetch_one(&mut *tx)
        .await?;

        // Update lot stage to RoastedBean and weight
        sqlx::query(
            r#"
            UPDATE lots
            SET stage = $1, current_weight_kg = $2
            WHERE id = $3
            "#,
        )
        .bind(LotStage::RoastedBean.as_str())
        .bind(input.roasted_weight_kg)
        .bind(session.lot_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(updated)
    }

    /// Mark a roast session as failed
    pub async fn fail_session(
        &self,
        business_id: Uuid,
        session_id: Uuid,
        notes: Option<String>,
        notes_th: Option<String>,
    ) -> AppResult<RoastSession> {
        // Validate session exists and is in progress
        let session = self.get_session(business_id, session_id).await?;
        
        if session.status != RoastStatus::InProgress.as_str() {
            return Err(AppError::Validation {
                field: "session_id".to_string(),
                message: "Session is not in progress".to_string(),
                message_th: "เซสชันไม่ได้อยู่ในสถานะกำลังดำเนินการ".to_string(),
            });
        }

        let updated = sqlx::query_as::<_, RoastSession>(
            r#"
            UPDATE roast_sessions
            SET status = $1, notes = COALESCE($2, notes), notes_th = COALESCE($3, notes_th),
                completed_at = NOW()
            WHERE id = $4
            RETURNING id, business_id, lot_id, template_id, session_date, roaster_name,
                      equipment, green_bean_weight_kg, initial_moisture_percent,
                      temperature_log, charge_temp_celsius,
                      turning_point_time_seconds, turning_point_temp_celsius,
                      first_crack_time_seconds, first_crack_temp_celsius,
                      second_crack_time_seconds, second_crack_temp_celsius,
                      drop_time_seconds, drop_temp_celsius,
                      roasted_weight_kg, weight_loss_percent, final_moisture_percent,
                      development_time_seconds, development_time_ratio,
                      roast_level, color_value, status, notes, notes_th,
                      created_at, updated_at, completed_at, created_by
            "#,
        )
        .bind(RoastStatus::Failed.as_str())
        .bind(&notes)
        .bind(&notes_th)
        .bind(session_id)
        .fetch_one(&self.db)
        .await?;

        Ok(updated)
    }

    /// Get cupping samples linked to a roast session
    pub async fn get_session_cuppings(
        &self,
        business_id: Uuid,
        session_id: Uuid,
    ) -> AppResult<Vec<CuppingSampleSummary>> {
        // Validate session exists
        let _ = self.get_session(business_id, session_id).await?;

        let samples = sqlx::query_as::<_, CuppingSampleSummary>(
            r#"
            SELECT cs.id, cs.session_id, cs.lot_id, cs.total_score, cs.notes, cs.notes_th,
                   cs.created_at, l.name as lot_name, l.traceability_code
            FROM cupping_samples cs
            JOIN lots l ON l.id = cs.lot_id
            WHERE cs.roast_session_id = $1
            ORDER BY cs.created_at DESC
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.db)
        .await?;

        Ok(samples)
    }
}

/// Summary of cupping sample linked to roast session
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CuppingSampleSummary {
    pub id: Uuid,
    pub session_id: Uuid,
    pub lot_id: Uuid,
    pub total_score: Option<Decimal>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
    pub created_at: DateTime<Utc>,
    pub lot_name: String,
    pub traceability_code: String,
}

/// Calculate weight loss percentage
/// Formula: ((green_weight - roasted_weight) / green_weight) × 100
pub fn calculate_weight_loss(green_weight: Decimal, roasted_weight: Decimal) -> Decimal {
    if green_weight.is_zero() {
        return Decimal::ZERO;
    }
    ((green_weight - roasted_weight) / green_weight) * Decimal::from(100)
}

/// Calculate development time ratio (DTR)
/// Formula: (development_time / total_time) × 100
pub fn calculate_dtr(development_time: i32, total_time: i32) -> Decimal {
    if total_time <= 0 {
        return Decimal::ZERO;
    }
    (Decimal::from(development_time) / Decimal::from(total_time)) * Decimal::from(100)
}
