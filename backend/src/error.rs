//! Error handling for the Coffee Quality Management Platform
//!
//! Provides consistent error responses in Thai and English

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Application error types
#[derive(Error, Debug)]
pub enum AppError {
    // Authentication errors
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Unauthorized: {message}")]
    Unauthorized {
        message: String,
        message_th: String,
    },

    // Validation errors
    #[error("Validation error: {message}")]
    Validation {
        field: String,
        message: String,
        message_th: String,
    },

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    #[error("Conflict: {message}")]
    Conflict {
        resource: String,
        message: String,
        message_th: String,
    },

    #[error("Resource not found: {0}")]
    NotFound(String),

    // Business logic errors
    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),

    #[error("Insufficient inventory: {0}")]
    InsufficientInventory(String),

    #[error("Certification expired: {0}")]
    CertificationExpired(String),

    // External service errors
    #[error("Weather service unavailable")]
    WeatherServiceUnavailable,

    #[error("LINE API error: {0}")]
    LineApiError(String),

    #[error("AI detection service error: {0}")]
    AiDetectionError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    // Sync errors
    #[error("Sync conflict detected")]
    SyncConflict {
        conflict: crate::services::sync::SyncConflict,
    },

    // Database errors
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    // Internal errors
    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Internal server error")]
    InternalError(#[from] anyhow::Error),
}

/// Error response structure
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message_en: String,
    pub message_th: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_detail) = match &self {
            AppError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                ErrorDetail {
                    code: "INVALID_CREDENTIALS".to_string(),
                    message_en: "Invalid email or password".to_string(),
                    message_th: "อีเมลหรือรหัสผ่านไม่ถูกต้อง".to_string(),
                    field: None,
                },
            ),
            AppError::TokenExpired => (
                StatusCode::UNAUTHORIZED,
                ErrorDetail {
                    code: "TOKEN_EXPIRED".to_string(),
                    message_en: "Token has expired".to_string(),
                    message_th: "โทเค็นหมดอายุแล้ว".to_string(),
                    field: None,
                },
            ),
            AppError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                ErrorDetail {
                    code: "INVALID_TOKEN".to_string(),
                    message_en: "Invalid token".to_string(),
                    message_th: "โทเค็นไม่ถูกต้อง".to_string(),
                    field: None,
                },
            ),
            AppError::InsufficientPermissions => (
                StatusCode::FORBIDDEN,
                ErrorDetail {
                    code: "INSUFFICIENT_PERMISSIONS".to_string(),
                    message_en: "You do not have permission to perform this action".to_string(),
                    message_th: "คุณไม่มีสิทธิ์ในการดำเนินการนี้".to_string(),
                    field: None,
                },
            ),
            AppError::Unauthorized { message, message_th } => (
                StatusCode::UNAUTHORIZED,
                ErrorDetail {
                    code: "UNAUTHORIZED".to_string(),
                    message_en: message.clone(),
                    message_th: message_th.clone(),
                    field: None,
                },
            ),
            AppError::Validation { field, message, message_th } => (
                StatusCode::BAD_REQUEST,
                ErrorDetail {
                    code: "VALIDATION_ERROR".to_string(),
                    message_en: message.clone(),
                    message_th: message_th.clone(),
                    field: Some(field.clone()),
                },
            ),
            AppError::ValidationError(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorDetail {
                    code: "VALIDATION_ERROR".to_string(),
                    message_en: msg.clone(),
                    message_th: format!("ข้อมูลไม่ถูกต้อง: {}", msg),
                    field: None,
                },
            ),
            AppError::DuplicateEntry(field) => (
                StatusCode::CONFLICT,
                ErrorDetail {
                    code: "DUPLICATE_ENTRY".to_string(),
                    message_en: format!("A record with this {} already exists", field),
                    message_th: format!("มีข้อมูล {} นี้อยู่แล้ว", field),
                    field: Some(field.clone()),
                },
            ),
            AppError::Conflict { resource, message, message_th } => (
                StatusCode::CONFLICT,
                ErrorDetail {
                    code: "CONFLICT".to_string(),
                    message_en: message.clone(),
                    message_th: message_th.clone(),
                    field: Some(resource.clone()),
                },
            ),
            AppError::NotFound(resource) => (
                StatusCode::NOT_FOUND,
                ErrorDetail {
                    code: "NOT_FOUND".to_string(),
                    message_en: format!("{} not found", resource),
                    message_th: format!("ไม่พบ {}", resource),
                    field: None,
                },
            ),
            AppError::InvalidStateTransition(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorDetail {
                    code: "INVALID_STATE_TRANSITION".to_string(),
                    message_en: msg.clone(),
                    message_th: format!("ไม่สามารถเปลี่ยนสถานะได้: {}", msg),
                    field: None,
                },
            ),
            AppError::InsufficientInventory(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorDetail {
                    code: "INSUFFICIENT_INVENTORY".to_string(),
                    message_en: msg.clone(),
                    message_th: format!("สินค้าคงคลังไม่เพียงพอ: {}", msg),
                    field: None,
                },
            ),
            AppError::CertificationExpired(cert) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorDetail {
                    code: "CERTIFICATION_EXPIRED".to_string(),
                    message_en: format!("Certification {} has expired", cert),
                    message_th: format!("ใบรับรอง {} หมดอายุแล้ว", cert),
                    field: None,
                },
            ),
            AppError::WeatherServiceUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                ErrorDetail {
                    code: "WEATHER_SERVICE_UNAVAILABLE".to_string(),
                    message_en: "Weather service is temporarily unavailable".to_string(),
                    message_th: "บริการข้อมูลสภาพอากาศไม่พร้อมใช้งานชั่วคราว".to_string(),
                    field: None,
                },
            ),
            AppError::LineApiError(msg) => (
                StatusCode::BAD_GATEWAY,
                ErrorDetail {
                    code: "LINE_API_ERROR".to_string(),
                    message_en: format!("LINE API error: {}", msg),
                    message_th: format!("เกิดข้อผิดพลาดกับ LINE API: {}", msg),
                    field: None,
                },
            ),
            AppError::AiDetectionError(msg) => (
                StatusCode::BAD_GATEWAY,
                ErrorDetail {
                    code: "AI_DETECTION_ERROR".to_string(),
                    message_en: format!("AI detection service error: {}", msg),
                    message_th: format!("เกิดข้อผิดพลาดกับบริการตรวจจับ AI: {}", msg),
                    field: None,
                },
            ),
            AppError::StorageError(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                ErrorDetail {
                    code: "STORAGE_ERROR".to_string(),
                    message_en: format!("Storage error: {}", msg),
                    message_th: format!("เกิดข้อผิดพลาดในการจัดเก็บ: {}", msg),
                    field: None,
                },
            ),
            AppError::ExternalService(msg) => (
                StatusCode::BAD_GATEWAY,
                ErrorDetail {
                    code: "EXTERNAL_SERVICE_ERROR".to_string(),
                    message_en: format!("External service error: {}", msg),
                    message_th: format!("เกิดข้อผิดพลาดกับบริการภายนอก: {}", msg),
                    field: None,
                },
            ),
            AppError::Configuration(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorDetail {
                    code: "CONFIGURATION_ERROR".to_string(),
                    message_en: format!("Configuration error: {}", msg),
                    message_th: format!("เกิดข้อผิดพลาดในการตั้งค่า: {}", msg),
                    field: None,
                },
            ),
            AppError::SyncConflict { .. } => (
                StatusCode::CONFLICT,
                ErrorDetail {
                    code: "SYNC_CONFLICT".to_string(),
                    message_en: "A sync conflict was detected. Please resolve the conflict."
                        .to_string(),
                    message_th: "พบความขัดแย้งในการซิงค์ กรุณาแก้ไขความขัดแย้ง".to_string(),
                    field: None,
                },
            ),
            AppError::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorDetail {
                    code: "DATABASE_ERROR".to_string(),
                    message_en: "A database error occurred".to_string(),
                    message_th: "เกิดข้อผิดพลาดกับฐานข้อมูล".to_string(),
                    field: None,
                },
            ),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorDetail {
                    code: "INTERNAL_ERROR".to_string(),
                    message_en: msg.clone(),
                    message_th: "เกิดข้อผิดพลาดภายในเซิร์ฟเวอร์".to_string(),
                    field: None,
                },
            ),
            AppError::InternalError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorDetail {
                    code: "INTERNAL_ERROR".to_string(),
                    message_en: "An internal server error occurred".to_string(),
                    message_th: "เกิดข้อผิดพลาดภายในเซิร์ฟเวอร์".to_string(),
                    field: None,
                },
            ),
        };

        // Log the error for debugging
        tracing::error!("Error: {:?}", self);

        (status, Json(ErrorResponse { error: error_detail })).into_response()
    }
}

/// Result type alias for handlers
pub type AppResult<T> = Result<T, AppError>;
