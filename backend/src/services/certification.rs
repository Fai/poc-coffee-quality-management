//! Certification service for managing certifications and compliance

use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Certification service for managing certifications
#[derive(Clone)]
pub struct CertificationService {
    db: PgPool,
}

/// Certification type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "certification_type", rename_all = "snake_case")]
pub enum CertificationType {
    ThaiGap,
    OrganicThailand,
    UsdaOrganic,
    FairTrade,
    RainforestAlliance,
    Utz,
    Other,
}

impl CertificationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CertificationType::ThaiGap => "thai_gap",
            CertificationType::OrganicThailand => "organic_thailand",
            CertificationType::UsdaOrganic => "usda_organic",
            CertificationType::FairTrade => "fair_trade",
            CertificationType::RainforestAlliance => "rainforest_alliance",
            CertificationType::Utz => "utz",
            CertificationType::Other => "other",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            CertificationType::ThaiGap => "Thai GAP",
            CertificationType::OrganicThailand => "Organic Thailand",
            CertificationType::UsdaOrganic => "USDA Organic",
            CertificationType::FairTrade => "Fair Trade",
            CertificationType::RainforestAlliance => "Rainforest Alliance",
            CertificationType::Utz => "UTZ",
            CertificationType::Other => "Other",
        }
    }

    pub fn display_name_th(&self) -> &'static str {
        match self {
            CertificationType::ThaiGap => "มาตรฐาน GAP ไทย",
            CertificationType::OrganicThailand => "เกษตรอินทรีย์ไทย",
            CertificationType::UsdaOrganic => "USDA Organic",
            CertificationType::FairTrade => "การค้าที่เป็นธรรม",
            CertificationType::RainforestAlliance => "Rainforest Alliance",
            CertificationType::Utz => "UTZ",
            CertificationType::Other => "อื่นๆ",
        }
    }
}

/// Certification scope enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "certification_scope", rename_all = "snake_case")]
pub enum CertificationScope {
    Farm,
    Plot,
    Facility,
    Business,
}

/// Certification record
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Certification {
    pub id: Uuid,
    pub business_id: Uuid,
    pub certification_type: CertificationType,
    pub certification_name: String,
    pub certification_body: String,
    pub certificate_number: String,
    pub scope: CertificationScope,
    pub plot_id: Option<Uuid>,
    pub issue_date: NaiveDate,
    pub expiration_date: NaiveDate,
    pub is_active: bool,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// Input for creating a certification
#[derive(Debug, Deserialize)]
pub struct CreateCertificationInput {
    pub certification_type: CertificationType,
    pub certification_name: String,
    pub certification_body: String,
    pub certificate_number: String,
    pub scope: Option<CertificationScope>,
    pub plot_id: Option<Uuid>,
    pub issue_date: NaiveDate,
    pub expiration_date: NaiveDate,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
}

/// Input for updating a certification
#[derive(Debug, Deserialize)]
pub struct UpdateCertificationInput {
    pub certification_name: Option<String>,
    pub certification_body: Option<String>,
    pub certificate_number: Option<String>,
    pub scope: Option<CertificationScope>,
    pub plot_id: Option<Uuid>,
    pub issue_date: Option<NaiveDate>,
    pub expiration_date: Option<NaiveDate>,
    pub is_active: Option<bool>,
    pub notes: Option<String>,
    pub notes_th: Option<String>,
}

/// Certification document record
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CertificationDocument {
    pub id: Uuid,
    pub certification_id: Uuid,
    pub document_type: String,
    pub document_name: String,
    pub file_url: String,
    pub file_size_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub uploaded_at: chrono::DateTime<Utc>,
    pub uploaded_by: Option<Uuid>,
}

/// Input for uploading a document
#[derive(Debug, Deserialize)]
pub struct UploadDocumentInput {
    pub document_type: String,
    pub document_name: String,
    pub file_url: String,
    pub file_size_bytes: Option<i64>,
    pub mime_type: Option<String>,
}

/// Certification requirement record
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CertificationRequirement {
    pub id: Uuid,
    pub certification_type: CertificationType,
    pub requirement_code: String,
    pub requirement_name: String,
    pub requirement_name_th: Option<String>,
    pub description: Option<String>,
    pub description_th: Option<String>,
    pub category: Option<String>,
    pub display_order: i32,
    pub is_critical: bool,
}

/// Certification compliance record
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CertificationCompliance {
    pub id: Uuid,
    pub certification_id: Uuid,
    pub requirement_id: Uuid,
    pub is_compliant: Option<bool>,
    pub compliance_notes: Option<String>,
    pub evidence_url: Option<String>,
    pub verified_at: Option<chrono::DateTime<Utc>>,
    pub verified_by: Option<Uuid>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// Input for updating compliance
#[derive(Debug, Deserialize)]
pub struct UpdateComplianceInput {
    pub is_compliant: Option<bool>,
    pub compliance_notes: Option<String>,
    pub evidence_url: Option<String>,
}

/// Expiring certification info
#[derive(Debug, Clone, Serialize)]
pub struct ExpiringCertification {
    pub certification_id: Uuid,
    pub certification_name: String,
    pub certification_type: CertificationType,
    pub expiration_date: NaiveDate,
    pub days_until_expiration: i32,
}

/// Certification with compliance summary
#[derive(Debug, Clone, Serialize)]
pub struct CertificationWithCompliance {
    pub certification: Certification,
    pub total_requirements: i64,
    pub compliant_count: i64,
    pub non_compliant_count: i64,
    pub pending_count: i64,
}

impl CertificationService {
    /// Create a new CertificationService instance
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    // ========================================================================
    // Certification CRUD
    // ========================================================================

    /// Create a new certification
    pub async fn create_certification(
        &self,
        business_id: Uuid,
        input: CreateCertificationInput,
    ) -> AppResult<Certification> {
        // Validate dates
        if input.expiration_date <= input.issue_date {
            return Err(AppError::Validation {
                field: "expiration_date".to_string(),
                message: "Expiration date must be after issue date".to_string(),
                message_th: "วันหมดอายุต้องหลังวันออกใบรับรอง".to_string(),
            });
        }

        // Validate plot if provided
        if let Some(plot_id) = input.plot_id {
            let plot_exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM plots WHERE id = $1 AND business_id = $2)",
            )
            .bind(plot_id)
            .bind(business_id)
            .fetch_one(&self.db)
            .await?;

            if !plot_exists {
                return Err(AppError::NotFound("Plot".to_string()));
            }
        }

        let scope = input.scope.unwrap_or(CertificationScope::Business);

        let certification = sqlx::query_as::<_, Certification>(
            r#"
            INSERT INTO certifications (
                business_id, certification_type, certification_name, certification_body,
                certificate_number, scope, plot_id, issue_date, expiration_date,
                notes, notes_th
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, business_id, certification_type, certification_name, certification_body,
                      certificate_number, scope, plot_id, issue_date, expiration_date,
                      is_active, notes, notes_th, created_at, updated_at
            "#,
        )
        .bind(business_id)
        .bind(&input.certification_type)
        .bind(&input.certification_name)
        .bind(&input.certification_body)
        .bind(&input.certificate_number)
        .bind(&scope)
        .bind(input.plot_id)
        .bind(input.issue_date)
        .bind(input.expiration_date)
        .bind(&input.notes)
        .bind(&input.notes_th)
        .fetch_one(&self.db)
        .await?;

        Ok(certification)
    }

    /// Get a certification by ID
    pub async fn get_certification(
        &self,
        business_id: Uuid,
        certification_id: Uuid,
    ) -> AppResult<Certification> {
        let certification = sqlx::query_as::<_, Certification>(
            r#"
            SELECT id, business_id, certification_type, certification_name, certification_body,
                   certificate_number, scope, plot_id, issue_date, expiration_date,
                   is_active, notes, notes_th, created_at, updated_at
            FROM certifications
            WHERE id = $1 AND business_id = $2
            "#,
        )
        .bind(certification_id)
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Certification".to_string()))?;

        Ok(certification)
    }

    /// List certifications for a business
    pub async fn list_certifications(
        &self,
        business_id: Uuid,
        active_only: bool,
    ) -> AppResult<Vec<Certification>> {
        let certifications = if active_only {
            sqlx::query_as::<_, Certification>(
                r#"
                SELECT id, business_id, certification_type, certification_name, certification_body,
                       certificate_number, scope, plot_id, issue_date, expiration_date,
                       is_active, notes, notes_th, created_at, updated_at
                FROM certifications
                WHERE business_id = $1 AND is_active = true
                ORDER BY expiration_date ASC
                "#,
            )
            .bind(business_id)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, Certification>(
                r#"
                SELECT id, business_id, certification_type, certification_name, certification_body,
                       certificate_number, scope, plot_id, issue_date, expiration_date,
                       is_active, notes, notes_th, created_at, updated_at
                FROM certifications
                WHERE business_id = $1
                ORDER BY is_active DESC, expiration_date ASC
                "#,
            )
            .bind(business_id)
            .fetch_all(&self.db)
            .await?
        };

        Ok(certifications)
    }

    /// Update a certification
    pub async fn update_certification(
        &self,
        business_id: Uuid,
        certification_id: Uuid,
        input: UpdateCertificationInput,
    ) -> AppResult<Certification> {
        // Get existing certification
        let existing = self.get_certification(business_id, certification_id).await?;

        // Validate dates if both are being updated
        let issue_date = input.issue_date.unwrap_or(existing.issue_date);
        let expiration_date = input.expiration_date.unwrap_or(existing.expiration_date);
        
        if expiration_date <= issue_date {
            return Err(AppError::Validation {
                field: "expiration_date".to_string(),
                message: "Expiration date must be after issue date".to_string(),
                message_th: "วันหมดอายุต้องหลังวันออกใบรับรอง".to_string(),
            });
        }

        let certification = sqlx::query_as::<_, Certification>(
            r#"
            UPDATE certifications SET
                certification_name = COALESCE($3, certification_name),
                certification_body = COALESCE($4, certification_body),
                certificate_number = COALESCE($5, certificate_number),
                scope = COALESCE($6, scope),
                plot_id = COALESCE($7, plot_id),
                issue_date = COALESCE($8, issue_date),
                expiration_date = COALESCE($9, expiration_date),
                is_active = COALESCE($10, is_active),
                notes = COALESCE($11, notes),
                notes_th = COALESCE($12, notes_th),
                updated_at = NOW()
            WHERE id = $1 AND business_id = $2
            RETURNING id, business_id, certification_type, certification_name, certification_body,
                      certificate_number, scope, plot_id, issue_date, expiration_date,
                      is_active, notes, notes_th, created_at, updated_at
            "#,
        )
        .bind(certification_id)
        .bind(business_id)
        .bind(&input.certification_name)
        .bind(&input.certification_body)
        .bind(&input.certificate_number)
        .bind(&input.scope)
        .bind(input.plot_id)
        .bind(input.issue_date)
        .bind(input.expiration_date)
        .bind(input.is_active)
        .bind(&input.notes)
        .bind(&input.notes_th)
        .fetch_one(&self.db)
        .await?;

        Ok(certification)
    }

    /// Delete a certification
    pub async fn delete_certification(
        &self,
        business_id: Uuid,
        certification_id: Uuid,
    ) -> AppResult<()> {
        let result = sqlx::query(
            "DELETE FROM certifications WHERE id = $1 AND business_id = $2",
        )
        .bind(certification_id)
        .bind(business_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Certification".to_string()));
        }

        Ok(())
    }


    // ========================================================================
    // Document Management
    // ========================================================================

    /// Upload a document for a certification
    pub async fn upload_document(
        &self,
        business_id: Uuid,
        certification_id: Uuid,
        user_id: Uuid,
        input: UploadDocumentInput,
    ) -> AppResult<CertificationDocument> {
        // Validate certification exists
        let _ = self.get_certification(business_id, certification_id).await?;

        // Validate document type
        let valid_types = ["certificate", "audit_report", "checklist", "other"];
        if !valid_types.contains(&input.document_type.as_str()) {
            return Err(AppError::Validation {
                field: "document_type".to_string(),
                message: format!("Invalid document type. Must be one of: {:?}", valid_types),
                message_th: format!("ประเภทเอกสารไม่ถูกต้อง ต้องเป็นหนึ่งใน: {:?}", valid_types),
            });
        }

        let document = sqlx::query_as::<_, CertificationDocument>(
            r#"
            INSERT INTO certification_documents (
                certification_id, document_type, document_name, file_url,
                file_size_bytes, mime_type, uploaded_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, certification_id, document_type, document_name, file_url,
                      file_size_bytes, mime_type, uploaded_at, uploaded_by
            "#,
        )
        .bind(certification_id)
        .bind(&input.document_type)
        .bind(&input.document_name)
        .bind(&input.file_url)
        .bind(input.file_size_bytes)
        .bind(&input.mime_type)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(document)
    }

    /// List documents for a certification
    pub async fn list_documents(
        &self,
        business_id: Uuid,
        certification_id: Uuid,
    ) -> AppResult<Vec<CertificationDocument>> {
        // Validate certification exists
        let _ = self.get_certification(business_id, certification_id).await?;

        let documents = sqlx::query_as::<_, CertificationDocument>(
            r#"
            SELECT id, certification_id, document_type, document_name, file_url,
                   file_size_bytes, mime_type, uploaded_at, uploaded_by
            FROM certification_documents
            WHERE certification_id = $1
            ORDER BY uploaded_at DESC
            "#,
        )
        .bind(certification_id)
        .fetch_all(&self.db)
        .await?;

        Ok(documents)
    }

    /// Delete a document
    pub async fn delete_document(
        &self,
        business_id: Uuid,
        certification_id: Uuid,
        document_id: Uuid,
    ) -> AppResult<()> {
        // Validate certification exists
        let _ = self.get_certification(business_id, certification_id).await?;

        let result = sqlx::query(
            "DELETE FROM certification_documents WHERE id = $1 AND certification_id = $2",
        )
        .bind(document_id)
        .bind(certification_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Document".to_string()));
        }

        Ok(())
    }

    // ========================================================================
    // Requirements and Compliance
    // ========================================================================

    /// Get requirements for a certification type
    pub async fn get_requirements(
        &self,
        certification_type: &CertificationType,
    ) -> AppResult<Vec<CertificationRequirement>> {
        let requirements = sqlx::query_as::<_, CertificationRequirement>(
            r#"
            SELECT id, certification_type, requirement_code, requirement_name,
                   requirement_name_th, description, description_th, category,
                   display_order, is_critical
            FROM certification_requirements
            WHERE certification_type = $1
            ORDER BY display_order ASC
            "#,
        )
        .bind(certification_type)
        .fetch_all(&self.db)
        .await?;

        Ok(requirements)
    }

    /// Get compliance status for a certification
    pub async fn get_compliance(
        &self,
        business_id: Uuid,
        certification_id: Uuid,
    ) -> AppResult<Vec<CertificationCompliance>> {
        // Validate certification exists
        let _ = self.get_certification(business_id, certification_id).await?;

        let compliance = sqlx::query_as::<_, CertificationCompliance>(
            r#"
            SELECT id, certification_id, requirement_id, is_compliant,
                   compliance_notes, evidence_url, verified_at, verified_by,
                   created_at, updated_at
            FROM certification_compliance
            WHERE certification_id = $1
            "#,
        )
        .bind(certification_id)
        .fetch_all(&self.db)
        .await?;

        Ok(compliance)
    }

    /// Update compliance for a requirement
    pub async fn update_compliance(
        &self,
        business_id: Uuid,
        certification_id: Uuid,
        requirement_id: Uuid,
        user_id: Uuid,
        input: UpdateComplianceInput,
    ) -> AppResult<CertificationCompliance> {
        // Validate certification exists
        let _ = self.get_certification(business_id, certification_id).await?;

        let compliance = sqlx::query_as::<_, CertificationCompliance>(
            r#"
            INSERT INTO certification_compliance (
                certification_id, requirement_id, is_compliant, compliance_notes,
                evidence_url, verified_at, verified_by
            )
            VALUES ($1, $2, $3, $4, $5, NOW(), $6)
            ON CONFLICT (certification_id, requirement_id) DO UPDATE SET
                is_compliant = COALESCE($3, certification_compliance.is_compliant),
                compliance_notes = COALESCE($4, certification_compliance.compliance_notes),
                evidence_url = COALESCE($5, certification_compliance.evidence_url),
                verified_at = NOW(),
                verified_by = $6,
                updated_at = NOW()
            RETURNING id, certification_id, requirement_id, is_compliant,
                      compliance_notes, evidence_url, verified_at, verified_by,
                      created_at, updated_at
            "#,
        )
        .bind(certification_id)
        .bind(requirement_id)
        .bind(input.is_compliant)
        .bind(&input.compliance_notes)
        .bind(&input.evidence_url)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(compliance)
    }

    // ========================================================================
    // Expiration Alerts
    // ========================================================================

    /// Get certifications expiring within specified days
    pub async fn get_expiring_certifications(
        &self,
        business_id: Uuid,
        days_ahead: i32,
    ) -> AppResult<Vec<ExpiringCertification>> {
        let expiring = sqlx::query_as::<_, ExpiringCertificationRow>(
            r#"
            SELECT 
                id as certification_id,
                certification_name,
                certification_type,
                expiration_date,
                (expiration_date - CURRENT_DATE)::INT as days_until_expiration
            FROM certifications
            WHERE business_id = $1
              AND is_active = true
              AND expiration_date <= CURRENT_DATE + $2
              AND expiration_date >= CURRENT_DATE
            ORDER BY expiration_date ASC
            "#,
        )
        .bind(business_id)
        .bind(days_ahead)
        .fetch_all(&self.db)
        .await?;

        Ok(expiring
            .into_iter()
            .map(|r| ExpiringCertification {
                certification_id: r.certification_id,
                certification_name: r.certification_name,
                certification_type: r.certification_type,
                expiration_date: r.expiration_date,
                days_until_expiration: r.days_until_expiration,
            })
            .collect())
    }

    /// Check for certifications that need alerts at 90, 60, 30 days
    pub async fn check_expiration_alerts(
        &self,
        business_id: Uuid,
    ) -> AppResult<Vec<(ExpiringCertification, i32)>> {
        let alert_days = [90, 60, 30];
        let mut alerts = Vec::new();

        for days in alert_days {
            let expiring = self.get_expiring_certifications(business_id, days).await?;
            for cert in expiring {
                // Check if this exact alert hasn't been sent yet
                let alert_sent = sqlx::query_scalar::<_, bool>(
                    r#"
                    SELECT EXISTS(
                        SELECT 1 FROM certification_alerts
                        WHERE certification_id = $1
                          AND alert_days_before = $2
                          AND alert_sent_at IS NOT NULL
                    )
                    "#,
                )
                .bind(cert.certification_id)
                .bind(days)
                .fetch_one(&self.db)
                .await?;

                if !alert_sent && cert.days_until_expiration <= days {
                    alerts.push((cert, days));
                }
            }
        }

        Ok(alerts)
    }

    /// Mark an alert as sent
    pub async fn mark_alert_sent(
        &self,
        certification_id: Uuid,
        days_before: i32,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE certification_alerts
            SET alert_sent_at = NOW()
            WHERE certification_id = $1 AND alert_days_before = $2
            "#,
        )
        .bind(certification_id)
        .bind(days_before)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    // ========================================================================
    // Traceability Integration
    // ========================================================================

    /// Get active certifications for a lot's traceability view
    pub async fn get_certifications_for_lot(
        &self,
        business_id: Uuid,
        plot_id: Option<Uuid>,
    ) -> AppResult<Vec<Certification>> {
        let today = Utc::now().date_naive();

        let certifications = sqlx::query_as::<_, Certification>(
            r#"
            SELECT id, business_id, certification_type, certification_name, certification_body,
                   certificate_number, scope, plot_id, issue_date, expiration_date,
                   is_active, notes, notes_th, created_at, updated_at
            FROM certifications
            WHERE business_id = $1
              AND is_active = true
              AND expiration_date >= $2
              AND (
                  scope = 'business'
                  OR scope = 'farm'
                  OR (scope = 'plot' AND plot_id = $3)
                  OR (scope = 'facility')
              )
            ORDER BY certification_type ASC
            "#,
        )
        .bind(business_id)
        .bind(today)
        .bind(plot_id)
        .fetch_all(&self.db)
        .await?;

        Ok(certifications)
    }

    /// Get certification with compliance summary
    pub async fn get_certification_with_compliance(
        &self,
        business_id: Uuid,
        certification_id: Uuid,
    ) -> AppResult<CertificationWithCompliance> {
        let certification = self.get_certification(business_id, certification_id).await?;

        // Get compliance counts
        let counts = sqlx::query_as::<_, ComplianceCounts>(
            r#"
            SELECT 
                COUNT(*)::BIGINT as total,
                COUNT(*) FILTER (WHERE cc.is_compliant = true)::BIGINT as compliant,
                COUNT(*) FILTER (WHERE cc.is_compliant = false)::BIGINT as non_compliant,
                COUNT(*) FILTER (WHERE cc.is_compliant IS NULL)::BIGINT as pending
            FROM certification_requirements cr
            LEFT JOIN certification_compliance cc 
                ON cc.requirement_id = cr.id AND cc.certification_id = $1
            WHERE cr.certification_type = $2
            "#,
        )
        .bind(certification_id)
        .bind(&certification.certification_type)
        .fetch_one(&self.db)
        .await?;

        Ok(CertificationWithCompliance {
            certification,
            total_requirements: counts.total,
            compliant_count: counts.compliant,
            non_compliant_count: counts.non_compliant,
            pending_count: counts.pending,
        })
    }
}

/// Helper struct for expiring certification query
#[derive(Debug, FromRow)]
struct ExpiringCertificationRow {
    certification_id: Uuid,
    certification_name: String,
    certification_type: CertificationType,
    expiration_date: NaiveDate,
    days_until_expiration: i32,
}

/// Helper struct for compliance counts
#[derive(Debug, FromRow)]
struct ComplianceCounts {
    total: i64,
    compliant: i64,
    non_compliant: i64,
    pending: i64,
}
