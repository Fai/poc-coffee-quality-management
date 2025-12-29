//! Business and organization models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{GpsCoordinates, Language};

/// Business types supported by the platform
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BusinessType {
    Farmer,
    Processor,
    Roaster,
    /// Combined operations (e.g., farm-to-cup)
    Integrated,
}

/// A registered business on the platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Business {
    pub id: Uuid,
    pub name: String,
    pub business_type: BusinessType,
    pub location: Option<GpsCoordinates>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub preferred_language: Language,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for registering a new business
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterBusinessInput {
    pub business_name: String,
    pub business_type: BusinessType,
    pub owner_name: String,
    pub email: String,
    pub password: String,
    pub phone: String,
    pub location: Option<GpsCoordinates>,
    pub address: Option<String>,
    pub preferred_language: Language,
}
