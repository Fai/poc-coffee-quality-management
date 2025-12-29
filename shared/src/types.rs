//! Common types used across the platform

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// GPS coordinates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GpsCoordinates {
    pub latitude: Decimal,
    pub longitude: Decimal,
}

impl GpsCoordinates {
    pub fn new(latitude: Decimal, longitude: Decimal) -> Self {
        Self {
            latitude,
            longitude,
        }
    }
}

/// Supported languages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Thai,
    English,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::Thai => "th",
            Language::English => "en",
        }
    }
}

/// Media reference for photos and documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaReference {
    pub id: uuid::Uuid,
    pub file_type: MediaType,
    pub url: String,
    pub original_filename: Option<String>,
}

/// Types of media files
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Image,
    Document,
    Video,
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
        }
    }
}

/// Paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total_items: u64,
    pub total_pages: u32,
}

/// Date range for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: chrono::NaiveDate,
    pub end: chrono::NaiveDate,
}
