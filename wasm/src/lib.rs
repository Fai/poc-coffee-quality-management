//! WebAssembly module for Coffee Quality Management Platform
//!
//! Provides client-side computation for:
//! - Cupping score calculations
//! - Grade classification
//! - Yield calculations
//! - Offline data validation

use rust_decimal::Decimal;
use wasm_bindgen::prelude::*;

// Re-export shared types for use in JavaScript
pub use shared::models::*;
pub use shared::types::*;
pub use shared::validation::*;

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn init() {
    // Set up panic hook for better error messages in browser console
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Calculate total cupping score from individual scores
#[wasm_bindgen]
pub fn calculate_cupping_total(scores_json: &str) -> Result<f64, JsValue> {
    let scores: CuppingScores = serde_json::from_str(scores_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid scores JSON: {}", e)))?;

    let total = scores.total();
    Ok(total.to_string().parse().unwrap_or(0.0))
}

/// Classify coffee grade based on defect counts
#[wasm_bindgen]
pub fn classify_coffee_grade(category1: i32, category2: i32) -> String {
    let defects = DefectCount {
        category1_count: category1,
        category2_count: category2,
        defect_breakdown: None,
    };

    let grade = classify_grade(&defects);
    format!("{}", grade)
}

/// Calculate processing yield percentage
#[wasm_bindgen]
pub fn calculate_processing_yield(cherry_weight: f64, green_bean_weight: f64) -> f64 {
    if cherry_weight <= 0.0 {
        return 0.0;
    }
    (green_bean_weight / cherry_weight) * 100.0
}

/// Calculate roast weight loss percentage
#[wasm_bindgen]
pub fn calculate_roast_weight_loss(green_weight: f64, roasted_weight: f64) -> f64 {
    if green_weight <= 0.0 {
        return 0.0;
    }
    ((green_weight - roasted_weight) / green_weight) * 100.0
}

/// Validate ripeness assessment (must sum to 100)
#[wasm_bindgen]
pub fn validate_ripeness_assessment(underripe: i32, ripe: i32, overripe: i32) -> bool {
    let total = underripe + ripe + overripe;
    total == 100 && underripe >= 0 && ripe >= 0 && overripe >= 0
}

/// Classify coffee by cupping score
#[wasm_bindgen]
pub fn classify_by_cupping_score(score: f64) -> String {
    let decimal_score = Decimal::try_from(score).unwrap_or(Decimal::ZERO);
    let classification = classify_by_score(decimal_score);
    format!("{}", classification)
}

/// Calculate harvest yield (kg per rai)
#[wasm_bindgen]
pub fn calculate_harvest_yield(total_weight_kg: f64, area_rai: f64) -> f64 {
    if area_rai <= 0.0 {
        return 0.0;
    }
    total_weight_kg / area_rai
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_coffee_grade() {
        assert_eq!(classify_coffee_grade(0, 3), "Specialty Grade");
        assert_eq!(classify_coffee_grade(0, 7), "Premium Grade");
        assert_eq!(classify_coffee_grade(1, 15), "Exchange Grade");
        assert_eq!(classify_coffee_grade(5, 50), "Below Standard");
        assert_eq!(classify_coffee_grade(10, 100), "Off Grade");
    }

    #[test]
    fn test_validate_ripeness() {
        assert!(validate_ripeness_assessment(10, 80, 10));
        assert!(!validate_ripeness_assessment(10, 80, 20));
        assert!(!validate_ripeness_assessment(-10, 100, 10));
    }

    #[test]
    fn test_processing_yield() {
        let yield_pct = calculate_processing_yield(100.0, 20.0);
        assert!((yield_pct - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_roast_weight_loss() {
        let loss = calculate_roast_weight_loss(100.0, 85.0);
        assert!((loss - 15.0).abs() < 0.001);
    }
}
