//! Cupping score models (SCA protocol)

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A cupping session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuppingSession {
    pub id: Uuid,
    pub business_id: Uuid,
    pub session_date: NaiveDate,
    pub cupper_name: String,
    pub samples: Vec<CuppingSample>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// A cupping sample within a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuppingSample {
    pub id: Uuid,
    pub session_id: Uuid,
    pub lot_id: Uuid,
    pub scores: CuppingScores,
    pub total_score: Decimal,
    pub tasting_notes: Option<String>,
    pub tasting_notes_th: Option<String>,
}

/// SCA Cupping Protocol Scores
/// Each attribute is scored on a 6.0-10.0 scale with 0.25 increments
/// Uniformity, Clean Cup, and Sweetness are scored 0-10 (2 points per cup)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuppingScores {
    pub fragrance_aroma: Decimal,
    pub flavor: Decimal,
    pub aftertaste: Decimal,
    pub acidity: Decimal,
    pub body: Decimal,
    pub balance: Decimal,
    /// 10 points max (2 per cup)
    pub uniformity: Decimal,
    /// 10 points max (2 per cup)
    pub clean_cup: Decimal,
    /// 10 points max (2 per cup)
    pub sweetness: Decimal,
    pub overall: Decimal,
}

impl CuppingScores {
    /// Calculate total cupping score
    pub fn total(&self) -> Decimal {
        self.fragrance_aroma
            + self.flavor
            + self.aftertaste
            + self.acidity
            + self.body
            + self.balance
            + self.uniformity
            + self.clean_cup
            + self.sweetness
            + self.overall
    }

    /// Validate that all scores are within valid ranges
    pub fn is_valid(&self) -> bool {
        let standard_range = |score: Decimal| score >= Decimal::from(6) && score <= Decimal::from(10);
        let full_range = |score: Decimal| score >= Decimal::ZERO && score <= Decimal::from(10);

        standard_range(self.fragrance_aroma)
            && standard_range(self.flavor)
            && standard_range(self.aftertaste)
            && standard_range(self.acidity)
            && standard_range(self.body)
            && standard_range(self.balance)
            && full_range(self.uniformity)
            && full_range(self.clean_cup)
            && full_range(self.sweetness)
            && standard_range(self.overall)
    }
}

/// Coffee classification based on cupping score
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoffeeClassification {
    /// 90+ points
    Outstanding,
    /// 85-89.99 points
    Excellent,
    /// 80-84.99 points
    VeryGood,
    /// Below 80 points
    BelowSpecialty,
}

impl std::fmt::Display for CoffeeClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoffeeClassification::Outstanding => write!(f, "Outstanding"),
            CoffeeClassification::Excellent => write!(f, "Excellent"),
            CoffeeClassification::VeryGood => write!(f, "Very Good"),
            CoffeeClassification::BelowSpecialty => write!(f, "Below Specialty"),
        }
    }
}

/// Classify coffee based on cupping score
pub fn classify_by_score(score: Decimal) -> CoffeeClassification {
    if score >= Decimal::from(90) {
        CoffeeClassification::Outstanding
    } else if score >= Decimal::from(85) {
        CoffeeClassification::Excellent
    } else if score >= Decimal::from(80) {
        CoffeeClassification::VeryGood
    } else {
        CoffeeClassification::BelowSpecialty
    }
}
