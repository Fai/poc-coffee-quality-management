//! HTTP handlers for certification management endpoints

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppResult;
use crate::middleware::CurrentUser;
use crate::services::certification::{
    Certification, CertificationCompliance, CertificationDocument, CertificationRequirement,
    CertificationService, CertificationType, CertificationWithCompliance,
    CreateCertificationInput, ExpiringCertification, UpdateCertificationInput,
    UpdateComplianceInput, UploadDocumentInput,
};
use crate::AppState;

// ============================================================================
// Certification CRUD
// ============================================================================

/// Create a new certification
pub async fn create_certification(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(input): Json<CreateCertificationInput>,
) -> AppResult<Json<Certification>> {
    let service = CertificationService::new(state.db);
    let certification = service
        .create_certification(current_user.0.business_id, input)
        .await?;
    Ok(Json(certification))
}

/// Get a certification by ID
pub async fn get_certification(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(certification_id): Path<Uuid>,
) -> AppResult<Json<Certification>> {
    let service = CertificationService::new(state.db);
    let certification = service
        .get_certification(current_user.0.business_id, certification_id)
        .await?;
    Ok(Json(certification))
}

/// Query parameters for listing certifications
#[derive(Debug, Deserialize)]
pub struct ListCertificationsQuery {
    pub active_only: Option<bool>,
}

/// List certifications for a business
pub async fn list_certifications(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<ListCertificationsQuery>,
) -> AppResult<Json<Vec<Certification>>> {
    let service = CertificationService::new(state.db);
    let active_only = query.active_only.unwrap_or(false);
    let certifications = service
        .list_certifications(current_user.0.business_id, active_only)
        .await?;
    Ok(Json(certifications))
}

/// Update a certification
pub async fn update_certification(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(certification_id): Path<Uuid>,
    Json(input): Json<UpdateCertificationInput>,
) -> AppResult<Json<Certification>> {
    let service = CertificationService::new(state.db);
    let certification = service
        .update_certification(current_user.0.business_id, certification_id, input)
        .await?;
    Ok(Json(certification))
}

/// Delete a certification
pub async fn delete_certification(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(certification_id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let service = CertificationService::new(state.db);
    service
        .delete_certification(current_user.0.business_id, certification_id)
        .await?;
    Ok(Json(()))
}

/// Get certification with compliance summary
pub async fn get_certification_with_compliance(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(certification_id): Path<Uuid>,
) -> AppResult<Json<CertificationWithCompliance>> {
    let service = CertificationService::new(state.db);
    let result = service
        .get_certification_with_compliance(current_user.0.business_id, certification_id)
        .await?;
    Ok(Json(result))
}

// ============================================================================
// Document Management
// ============================================================================

/// Upload a document for a certification
pub async fn upload_document(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(certification_id): Path<Uuid>,
    Json(input): Json<UploadDocumentInput>,
) -> AppResult<Json<CertificationDocument>> {
    let service = CertificationService::new(state.db);
    let document = service
        .upload_document(
            current_user.0.business_id,
            certification_id,
            current_user.0.user_id,
            input,
        )
        .await?;
    Ok(Json(document))
}

/// List documents for a certification
pub async fn list_documents(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(certification_id): Path<Uuid>,
) -> AppResult<Json<Vec<CertificationDocument>>> {
    let service = CertificationService::new(state.db);
    let documents = service
        .list_documents(current_user.0.business_id, certification_id)
        .await?;
    Ok(Json(documents))
}

/// Delete a document
pub async fn delete_document(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((certification_id, document_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<()>> {
    let service = CertificationService::new(state.db);
    service
        .delete_document(current_user.0.business_id, certification_id, document_id)
        .await?;
    Ok(Json(()))
}

// ============================================================================
// Requirements and Compliance
// ============================================================================

/// Get requirements for a certification type
pub async fn get_requirements(
    State(state): State<AppState>,
    Path(cert_type): Path<String>,
) -> AppResult<Json<Vec<CertificationRequirement>>> {
    let certification_type = parse_certification_type(&cert_type)?;
    let service = CertificationService::new(state.db);
    let requirements = service.get_requirements(&certification_type).await?;
    Ok(Json(requirements))
}

/// Get compliance status for a certification
pub async fn get_compliance(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(certification_id): Path<Uuid>,
) -> AppResult<Json<Vec<CertificationCompliance>>> {
    let service = CertificationService::new(state.db);
    let compliance = service
        .get_compliance(current_user.0.business_id, certification_id)
        .await?;
    Ok(Json(compliance))
}

/// Update compliance for a requirement
pub async fn update_compliance(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((certification_id, requirement_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateComplianceInput>,
) -> AppResult<Json<CertificationCompliance>> {
    let service = CertificationService::new(state.db);
    let compliance = service
        .update_compliance(
            current_user.0.business_id,
            certification_id,
            requirement_id,
            current_user.0.user_id,
            input,
        )
        .await?;
    Ok(Json(compliance))
}

// ============================================================================
// Expiration Alerts
// ============================================================================

/// Query parameters for expiring certifications
#[derive(Debug, Deserialize)]
pub struct ExpiringQuery {
    pub days_ahead: Option<i32>,
}

/// Get certifications expiring soon
pub async fn get_expiring_certifications(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<ExpiringQuery>,
) -> AppResult<Json<Vec<ExpiringCertification>>> {
    let service = CertificationService::new(state.db);
    let days_ahead = query.days_ahead.unwrap_or(90);
    let expiring = service
        .get_expiring_certifications(current_user.0.business_id, days_ahead)
        .await?;
    Ok(Json(expiring))
}

/// Check for expiration alerts (90, 60, 30 days)
pub async fn check_expiration_alerts(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<Vec<ExpirationAlertResponse>>> {
    let service = CertificationService::new(state.db);
    let alerts = service
        .check_expiration_alerts(current_user.0.business_id)
        .await?;
    
    let response: Vec<ExpirationAlertResponse> = alerts
        .into_iter()
        .map(|(cert, days)| ExpirationAlertResponse {
            certification: cert,
            alert_days: days,
        })
        .collect();
    
    Ok(Json(response))
}

/// Expiration alert response
#[derive(Debug, serde::Serialize)]
pub struct ExpirationAlertResponse {
    pub certification: ExpiringCertification,
    pub alert_days: i32,
}

// ============================================================================
// Traceability Integration
// ============================================================================

/// Get certifications for a lot's traceability view
pub async fn get_certifications_for_lot(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<LotCertificationsQuery>,
) -> AppResult<Json<Vec<Certification>>> {
    let service = CertificationService::new(state.db);
    let certifications = service
        .get_certifications_for_lot(current_user.0.business_id, query.plot_id)
        .await?;
    Ok(Json(certifications))
}

/// Query parameters for lot certifications
#[derive(Debug, Deserialize)]
pub struct LotCertificationsQuery {
    pub plot_id: Option<Uuid>,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse certification type from string
fn parse_certification_type(s: &str) -> AppResult<CertificationType> {
    match s.to_lowercase().as_str() {
        "thai_gap" => Ok(CertificationType::ThaiGap),
        "organic_thailand" => Ok(CertificationType::OrganicThailand),
        "usda_organic" => Ok(CertificationType::UsdaOrganic),
        "fair_trade" => Ok(CertificationType::FairTrade),
        "rainforest_alliance" => Ok(CertificationType::RainforestAlliance),
        "utz" => Ok(CertificationType::Utz),
        "other" => Ok(CertificationType::Other),
        _ => Err(crate::error::AppError::Validation {
            field: "certification_type".to_string(),
            message: format!("Invalid certification type: {}", s),
            message_th: format!("ประเภทใบรับรองไม่ถูกต้อง: {}", s),
        }),
    }
}
