//! Green bean grading models

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Green bean grade record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreenBeanGrade {
    pub id: Uuid,
    pub lot_id: Uuid,
    pub grading_date: NaiveDate,
    pub grader_name: String,
    pub sample_weight_grams: Decimal,
    pub defects: DefectCount,
    pub ai_detection: Option<AiDefectDetection>,
    pub moisture_percent: Decimal,
    pub density: Option<Decimal>,
    pub screen_size: Option<ScreenSizeDistribution>,
    pub grade: GradeClassification,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Defect counts for grading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectCount {
    /// Category 1 (primary) defects
    pub category1_count: i32,
    /// Category 2 (secondary) defects
    pub category2_count: i32,
    /// Detailed breakdown by defect type
    pub defect_breakdown: Option<DefectBreakdown>,
}

impl DefectCount {
    pub fn total(&self) -> i32 {
        self.category1_count + self.category2_count
    }
}

/// Detailed defect breakdown by type
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefectBreakdown {
    // Category 1 (Primary) Defects
    pub full_black: i32,
    pub full_sour: i32,
    pub pod_cherry: i32,
    pub large_stones: i32,
    pub medium_stones: i32,
    pub large_sticks: i32,
    pub medium_sticks: i32,
    // Category 2 (Secondary) Defects
    pub partial_black: i32,
    pub partial_sour: i32,
    pub parchment: i32,
    pub floater: i32,
    pub immature: i32,
    pub withered: i32,
    pub shell: i32,
    pub broken: i32,
    pub chipped: i32,
    pub cut: i32,
    pub insect_damage: i32,
    pub husk: i32,
}

/// AI defect detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiDefectDetection {
    pub request_id: String,
    pub image_url: String,
    pub detected_beans: i32,
    pub defect_breakdown: DefectBreakdown,
    pub category1_count: i32,
    pub category2_count: i32,
    pub confidence_score: f32,
    pub processing_time_ms: i32,
    pub annotated_image_url: Option<String>,
}

/// Screen size distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenSizeDistribution {
    pub screen_18_plus: Decimal,
    pub screen_17: Decimal,
    pub screen_16: Decimal,
    pub screen_15: Decimal,
    pub screen_14_below: Decimal,
}

/// SCA grade classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GradeClassification {
    /// 0-5 defects, 0 category 1
    SpecialtyGrade,
    /// 0-8 defects
    PremiumGrade,
    /// 9-23 defects
    ExchangeGrade,
    /// 24-86 defects
    BelowStandard,
    /// 86+ defects
    OffGrade,
}

impl std::fmt::Display for GradeClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GradeClassification::SpecialtyGrade => write!(f, "Specialty Grade"),
            GradeClassification::PremiumGrade => write!(f, "Premium Grade"),
            GradeClassification::ExchangeGrade => write!(f, "Exchange Grade"),
            GradeClassification::BelowStandard => write!(f, "Below Standard"),
            GradeClassification::OffGrade => write!(f, "Off Grade"),
        }
    }
}

/// Classify grade based on defect counts (SCA rules)
pub fn classify_grade(defects: &DefectCount) -> GradeClassification {
    let total = defects.total();
    match (defects.category1_count, total) {
        (0, 0..=5) => GradeClassification::SpecialtyGrade,
        (_, 0..=8) => GradeClassification::PremiumGrade,
        (_, 9..=23) => GradeClassification::ExchangeGrade,
        (_, 24..=86) => GradeClassification::BelowStandard,
        _ => GradeClassification::OffGrade,
    }
}
