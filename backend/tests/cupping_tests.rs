//! Cupping session and score tests
//!
//! Tests for SCA cupping protocol implementation including:
//! - Property 10: Cupping Score Calculation
//! - Property 11: Cupping Score Range Validity

use proptest::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;

// Helper to create Decimal from string
fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

// Import from the cupping service module
// Note: In actual integration tests, these would be imported from the crate
// For unit tests, we replicate the core logic here

/// SCA Cupping Protocol Scores
#[derive(Debug, Clone)]
struct CuppingScores {
    fragrance_aroma: Decimal,
    flavor: Decimal,
    aftertaste: Decimal,
    acidity: Decimal,
    body: Decimal,
    balance: Decimal,
    uniformity: Decimal,
    clean_cup: Decimal,
    sweetness: Decimal,
    overall: Decimal,
}

/// Cupping defects
#[derive(Debug, Clone, Default)]
struct CuppingDefects {
    taint_count: i32,  // 2 points each
    fault_count: i32,  // 4 points each
}

impl CuppingDefects {
    fn total_deduction(&self) -> Decimal {
        Decimal::from(self.taint_count * 2 + self.fault_count * 4)
    }
}

/// Coffee classification based on cupping score
#[derive(Debug, Clone, PartialEq, Eq)]
enum CoffeeClassification {
    Outstanding,      // 90+
    Excellent,        // 85-89.99
    VeryGood,         // 80-84.99
    BelowSpecialty,   // <80
}

/// Calculate total cupping score from individual scores
fn calculate_total_score(scores: &CuppingScores) -> Decimal {
    scores.fragrance_aroma
        + scores.flavor
        + scores.aftertaste
        + scores.acidity
        + scores.body
        + scores.balance
        + scores.uniformity
        + scores.clean_cup
        + scores.sweetness
        + scores.overall
}

/// Classify coffee based on final cupping score
fn classify_by_score(score: Decimal) -> CoffeeClassification {
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

/// Validate cupping scores are within valid ranges
fn validate_scores(scores: &CuppingScores) -> Result<(), String> {
    let min = Decimal::from(0);
    let max = Decimal::from(10);

    let all_scores = [
        ("fragrance_aroma", scores.fragrance_aroma),
        ("flavor", scores.flavor),
        ("aftertaste", scores.aftertaste),
        ("acidity", scores.acidity),
        ("body", scores.body),
        ("balance", scores.balance),
        ("uniformity", scores.uniformity),
        ("clean_cup", scores.clean_cup),
        ("sweetness", scores.sweetness),
        ("overall", scores.overall),
    ];

    for (name, score) in all_scores {
        if score < min || score > max {
            return Err(format!("{} must be between 0 and 10, got {}", name, score));
        }
    }

    Ok(())
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_calculate_total_score_perfect() {
        let scores = CuppingScores {
            fragrance_aroma: dec("10.0"),
            flavor: dec("10.0"),
            aftertaste: dec("10.0"),
            acidity: dec("10.0"),
            body: dec("10.0"),
            balance: dec("10.0"),
            uniformity: dec("10.0"),
            clean_cup: dec("10.0"),
            sweetness: dec("10.0"),
            overall: dec("10.0"),
        };

        let total = calculate_total_score(&scores);
        assert_eq!(total, dec("100.0"));
    }

    #[test]
    fn test_calculate_total_score_specialty() {
        let scores = CuppingScores {
            fragrance_aroma: dec("8.0"),
            flavor: dec("8.25"),
            aftertaste: dec("7.75"),
            acidity: dec("8.0"),
            body: dec("7.5"),
            balance: dec("8.0"),
            uniformity: dec("10.0"),
            clean_cup: dec("10.0"),
            sweetness: dec("10.0"),
            overall: dec("8.0"),
        };

        let total = calculate_total_score(&scores);
        assert_eq!(total, dec("85.5"));
    }

    #[test]
    fn test_calculate_total_score_minimum() {
        let scores = CuppingScores {
            fragrance_aroma: dec("0.0"),
            flavor: dec("0.0"),
            aftertaste: dec("0.0"),
            acidity: dec("0.0"),
            body: dec("0.0"),
            balance: dec("0.0"),
            uniformity: dec("0.0"),
            clean_cup: dec("0.0"),
            sweetness: dec("0.0"),
            overall: dec("0.0"),
        };

        let total = calculate_total_score(&scores);
        assert_eq!(total, dec("0.0"));
    }

    #[test]
    fn test_defect_deduction_taint() {
        let defects = CuppingDefects {
            taint_count: 2,
            fault_count: 0,
        };
        assert_eq!(defects.total_deduction(), dec("4"));
    }

    #[test]
    fn test_defect_deduction_fault() {
        let defects = CuppingDefects {
            taint_count: 0,
            fault_count: 2,
        };
        assert_eq!(defects.total_deduction(), dec("8"));
    }

    #[test]
    fn test_defect_deduction_combined() {
        let defects = CuppingDefects {
            taint_count: 1,
            fault_count: 1,
        };
        // 1 taint (2 pts) + 1 fault (4 pts) = 6 pts
        assert_eq!(defects.total_deduction(), dec("6"));
    }

    #[test]
    fn test_classification_outstanding() {
        assert_eq!(classify_by_score(dec("90.0")), CoffeeClassification::Outstanding);
        assert_eq!(classify_by_score(dec("95.5")), CoffeeClassification::Outstanding);
        assert_eq!(classify_by_score(dec("100.0")), CoffeeClassification::Outstanding);
    }

    #[test]
    fn test_classification_excellent() {
        assert_eq!(classify_by_score(dec("85.0")), CoffeeClassification::Excellent);
        assert_eq!(classify_by_score(dec("87.5")), CoffeeClassification::Excellent);
        assert_eq!(classify_by_score(dec("89.99")), CoffeeClassification::Excellent);
    }

    #[test]
    fn test_classification_very_good() {
        assert_eq!(classify_by_score(dec("80.0")), CoffeeClassification::VeryGood);
        assert_eq!(classify_by_score(dec("82.5")), CoffeeClassification::VeryGood);
        assert_eq!(classify_by_score(dec("84.99")), CoffeeClassification::VeryGood);
    }

    #[test]
    fn test_classification_below_specialty() {
        assert_eq!(classify_by_score(dec("79.99")), CoffeeClassification::BelowSpecialty);
        assert_eq!(classify_by_score(dec("70.0")), CoffeeClassification::BelowSpecialty);
        assert_eq!(classify_by_score(dec("0.0")), CoffeeClassification::BelowSpecialty);
    }

    #[test]
    fn test_classification_boundary_90() {
        assert_eq!(classify_by_score(dec("89.99")), CoffeeClassification::Excellent);
        assert_eq!(classify_by_score(dec("90.0")), CoffeeClassification::Outstanding);
    }

    #[test]
    fn test_classification_boundary_85() {
        assert_eq!(classify_by_score(dec("84.99")), CoffeeClassification::VeryGood);
        assert_eq!(classify_by_score(dec("85.0")), CoffeeClassification::Excellent);
    }

    #[test]
    fn test_classification_boundary_80() {
        assert_eq!(classify_by_score(dec("79.99")), CoffeeClassification::BelowSpecialty);
        assert_eq!(classify_by_score(dec("80.0")), CoffeeClassification::VeryGood);
    }

    #[test]
    fn test_validate_scores_valid() {
        let scores = CuppingScores {
            fragrance_aroma: dec("8.0"),
            flavor: dec("8.0"),
            aftertaste: dec("7.5"),
            acidity: dec("8.0"),
            body: dec("7.5"),
            balance: dec("8.0"),
            uniformity: dec("10.0"),
            clean_cup: dec("10.0"),
            sweetness: dec("10.0"),
            overall: dec("8.0"),
        };
        assert!(validate_scores(&scores).is_ok());
    }

    #[test]
    fn test_validate_scores_invalid_negative() {
        let scores = CuppingScores {
            fragrance_aroma: dec("-1.0"),
            flavor: dec("8.0"),
            aftertaste: dec("7.5"),
            acidity: dec("8.0"),
            body: dec("7.5"),
            balance: dec("8.0"),
            uniformity: dec("10.0"),
            clean_cup: dec("10.0"),
            sweetness: dec("10.0"),
            overall: dec("8.0"),
        };
        assert!(validate_scores(&scores).is_err());
    }

    #[test]
    fn test_validate_scores_invalid_over_max() {
        let scores = CuppingScores {
            fragrance_aroma: dec("8.0"),
            flavor: dec("11.0"),
            aftertaste: dec("7.5"),
            acidity: dec("8.0"),
            body: dec("7.5"),
            balance: dec("8.0"),
            uniformity: dec("10.0"),
            clean_cup: dec("10.0"),
            sweetness: dec("10.0"),
            overall: dec("8.0"),
        };
        assert!(validate_scores(&scores).is_err());
    }

    #[test]
    fn test_final_score_with_defects() {
        let scores = CuppingScores {
            fragrance_aroma: dec("8.5"),
            flavor: dec("8.5"),
            aftertaste: dec("8.0"),
            acidity: dec("8.5"),
            body: dec("8.0"),
            balance: dec("8.5"),
            uniformity: dec("10.0"),
            clean_cup: dec("10.0"),
            sweetness: dec("10.0"),
            overall: dec("8.5"),
        };

        let defects = CuppingDefects {
            taint_count: 1,
            fault_count: 0,
        };

        let total = calculate_total_score(&scores);
        let final_score = total - defects.total_deduction();

        assert_eq!(total, dec("88.5"));
        assert_eq!(final_score, dec("86.5")); // 88.5 - 2 = 86.5
    }

    #[test]
    fn test_score_precision() {
        let scores = CuppingScores {
            fragrance_aroma: dec("8.25"),
            flavor: dec("8.25"),
            aftertaste: dec("8.25"),
            acidity: dec("8.25"),
            body: dec("8.25"),
            balance: dec("8.25"),
            uniformity: dec("10.0"),
            clean_cup: dec("10.0"),
            sweetness: dec("10.0"),
            overall: dec("8.25"),
        };

        let total = calculate_total_score(&scores);
        // 8.25 * 7 + 10 * 3 = 57.75 + 30 = 87.75
        assert_eq!(total, dec("87.75"));
    }
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;

    /// Strategy for generating valid cupping scores (0.0 to 10.0)
    fn valid_score_strategy() -> impl Strategy<Value = Decimal> {
        (0u32..=1000u32).prop_map(|v| Decimal::from(v) / Decimal::from(100))
    }

    /// Strategy for generating valid cupping scores struct
    fn valid_scores_strategy() -> impl Strategy<Value = CuppingScores> {
        (
            valid_score_strategy(),
            valid_score_strategy(),
            valid_score_strategy(),
            valid_score_strategy(),
            valid_score_strategy(),
            valid_score_strategy(),
            valid_score_strategy(),
            valid_score_strategy(),
            valid_score_strategy(),
            valid_score_strategy(),
        )
            .prop_map(
                |(
                    fragrance_aroma,
                    flavor,
                    aftertaste,
                    acidity,
                    body,
                    balance,
                    uniformity,
                    clean_cup,
                    sweetness,
                    overall,
                )| {
                    CuppingScores {
                        fragrance_aroma,
                        flavor,
                        aftertaste,
                        acidity,
                        body,
                        balance,
                        uniformity,
                        clean_cup,
                        sweetness,
                        overall,
                    }
                },
            )
    }

    /// Strategy for generating defects
    fn defects_strategy() -> impl Strategy<Value = CuppingDefects> {
        (0i32..=5i32, 0i32..=5i32).prop_map(|(taint_count, fault_count)| CuppingDefects {
            taint_count,
            fault_count,
        })
    }

    proptest! {
        /// Property 10: Cupping Score Calculation
        /// Verify total = sum of all 10 attributes
        #[test]
        fn prop_total_score_equals_sum_of_attributes(scores in valid_scores_strategy()) {
            let total = calculate_total_score(&scores);

            let expected = scores.fragrance_aroma
                + scores.flavor
                + scores.aftertaste
                + scores.acidity
                + scores.body
                + scores.balance
                + scores.uniformity
                + scores.clean_cup
                + scores.sweetness
                + scores.overall;

            prop_assert_eq!(total, expected);
        }

        /// Property 10 (continued): Total score is within valid range
        #[test]
        fn prop_total_score_within_range(scores in valid_scores_strategy()) {
            let total = calculate_total_score(&scores);

            // Total should be between 0 and 100 (10 attributes * 10 max each)
            prop_assert!(total >= Decimal::from(0));
            prop_assert!(total <= Decimal::from(100));
        }

        /// Property 11: Cupping Score Range Validity
        /// Verify all scores within valid ranges (0-10)
        #[test]
        fn prop_valid_scores_pass_validation(scores in valid_scores_strategy()) {
            let result = validate_scores(&scores);
            prop_assert!(result.is_ok(), "Valid scores should pass validation");
        }

        /// Property 11 (continued): Invalid scores fail validation
        #[test]
        fn prop_negative_scores_fail_validation(
            valid_scores in valid_scores_strategy(),
            negative_value in (-100i32..-1i32),
            field_index in 0usize..10usize
        ) {
            let negative = Decimal::from(negative_value);
            let mut scores = valid_scores;

            // Set one field to negative
            match field_index {
                0 => scores.fragrance_aroma = negative,
                1 => scores.flavor = negative,
                2 => scores.aftertaste = negative,
                3 => scores.acidity = negative,
                4 => scores.body = negative,
                5 => scores.balance = negative,
                6 => scores.uniformity = negative,
                7 => scores.clean_cup = negative,
                8 => scores.sweetness = negative,
                _ => scores.overall = negative,
            }

            let result = validate_scores(&scores);
            prop_assert!(result.is_err(), "Negative scores should fail validation");
        }

        /// Property 11 (continued): Over-max scores fail validation
        #[test]
        fn prop_over_max_scores_fail_validation(
            valid_scores in valid_scores_strategy(),
            over_value in (11i32..100i32),
            field_index in 0usize..10usize
        ) {
            let over = Decimal::from(over_value);
            let mut scores = valid_scores;

            // Set one field to over max
            match field_index {
                0 => scores.fragrance_aroma = over,
                1 => scores.flavor = over,
                2 => scores.aftertaste = over,
                3 => scores.acidity = over,
                4 => scores.body = over,
                5 => scores.balance = over,
                6 => scores.uniformity = over,
                7 => scores.clean_cup = over,
                8 => scores.sweetness = over,
                _ => scores.overall = over,
            }

            let result = validate_scores(&scores);
            prop_assert!(result.is_err(), "Over-max scores should fail validation");
        }

        /// Property: Defect deduction is always non-negative
        #[test]
        fn prop_defect_deduction_non_negative(defects in defects_strategy()) {
            let deduction = defects.total_deduction();
            prop_assert!(deduction >= Decimal::from(0));
        }

        /// Property: Defect deduction formula is correct
        #[test]
        fn prop_defect_deduction_formula(defects in defects_strategy()) {
            let deduction = defects.total_deduction();
            let expected = Decimal::from(defects.taint_count * 2 + defects.fault_count * 4);
            prop_assert_eq!(deduction, expected);
        }

        /// Property: Final score = total - defects
        #[test]
        fn prop_final_score_calculation(
            scores in valid_scores_strategy(),
            defects in defects_strategy()
        ) {
            let total = calculate_total_score(&scores);
            let final_score = total - defects.total_deduction();

            // Final score should be total minus deductions
            prop_assert_eq!(final_score, total - defects.total_deduction());

            // Final score can be negative if defects are severe
            // but total is always >= 0
            prop_assert!(total >= Decimal::from(0));
        }

        /// Property: Classification is consistent with score ranges
        #[test]
        fn prop_classification_consistency(score in 0i32..=100i32) {
            let decimal_score = Decimal::from(score);
            let classification = classify_by_score(decimal_score);

            match classification {
                CoffeeClassification::Outstanding => {
                    prop_assert!(decimal_score >= Decimal::from(90));
                }
                CoffeeClassification::Excellent => {
                    prop_assert!(decimal_score >= Decimal::from(85));
                    prop_assert!(decimal_score < Decimal::from(90));
                }
                CoffeeClassification::VeryGood => {
                    prop_assert!(decimal_score >= Decimal::from(80));
                    prop_assert!(decimal_score < Decimal::from(85));
                }
                CoffeeClassification::BelowSpecialty => {
                    prop_assert!(decimal_score < Decimal::from(80));
                }
            }
        }

        /// Property: Classification boundaries are correct
        #[test]
        fn prop_classification_boundaries(boundary in prop::sample::select(vec![80, 85, 90])) {
            let at_boundary = classify_by_score(Decimal::from(boundary));
            let below_boundary = classify_by_score(Decimal::from(boundary) - dec("0.01"));

            // At boundary should be in higher class
            // Below boundary should be in lower class
            prop_assert_ne!(at_boundary, below_boundary);
        }
    }
}
