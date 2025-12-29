//! Certification models

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A certification record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certification {
    pub id: Uuid,
    pub business_id: Uuid,
    pub certification_type: CertificationType,
    pub certification_body: String,
    pub certificate_number: String,
    pub issue_date: NaiveDate,
    pub expiration_date: NaiveDate,
    pub scope: CertificationScope,
    pub status: CertificationStatus,
    pub created_at: DateTime<Utc>,
}

/// Types of certifications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CertificationType {
    ThaiGAP,
    OrganicThailand,
    USDAOrganic,
    FairTrade,
    RainforestAlliance,
    UTZ,
    Custom(String),
}

impl std::fmt::Display for CertificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CertificationType::ThaiGAP => write!(f, "Thai GAP"),
            CertificationType::OrganicThailand => write!(f, "Organic Thailand"),
            CertificationType::USDAOrganic => write!(f, "USDA Organic"),
            CertificationType::FairTrade => write!(f, "Fair Trade"),
            CertificationType::RainforestAlliance => write!(f, "Rainforest Alliance"),
            CertificationType::UTZ => write!(f, "UTZ"),
            CertificationType::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Scope of a certification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationScope {
    pub plots: Vec<Uuid>,
    pub facilities: Vec<String>,
}

/// Status of a certification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CertificationStatus {
    Active,
    /// Within 90 days of expiration
    ExpiringSoon,
    Expired,
    Suspended,
}

/// Days before expiration for alerts
pub const EXPIRATION_ALERT_DAYS: [i64; 3] = [90, 60, 30];
