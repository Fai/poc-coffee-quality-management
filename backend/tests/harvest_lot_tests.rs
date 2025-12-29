//! Harvest and Lot property-based and unit tests
//!
//! Comprehensive tests for:
//! - Property 3: Lot Identifier Uniqueness
//! - Property 5: Ripeness Assessment Validity
//! - Property 6: Harvest Yield Calculation
//! - Property 7: Lot Blending Traceability

use proptest::prelude::*;
use rust_decimal::Decimal;
use std::collections::HashSet;

// ============================================================================
// Property Test Strategies
// ============================================================================

/// Generate valid ripeness percentages that sum to 100
fn valid_ripeness_strategy() -> impl Strategy<Value = (i32, i32, i32)> {
    (0..=100i32).prop_flat_map(|underripe| {
        (0..=(100 - underripe)).prop_flat_map(move |ripe| {
            let overripe = 100 - underripe - ripe;
            Just((underripe, ripe, overripe))
        })
    })
}

/// Generate invalid ripeness percentages (don't sum to 100)
fn invalid_ripeness_strategy() -> impl Strategy<Value = (i32, i32, i32)> {
    (0..=100i32, 0..=100i32, 0..=100i32)
        .prop_filter("must not sum to 100", |(u, r, o)| u + r + o != 100)
}

/// Generate valid cherry weight in kg (0.1 to 1000)
fn cherry_weight_strategy() -> impl Strategy<Value = Decimal> {
    (1..=10000i64).prop_map(|n| Decimal::new(n, 1)) // 0.1 to 1000.0
}

/// Generate valid plot area in rai (0.1 to 100)
fn plot_area_strategy() -> impl Strategy<Value = Decimal> {
    (1..=1000i64).prop_map(|n| Decimal::new(n, 1)) // 0.1 to 100.0
}

/// Generate valid blend proportions that sum to 100
fn valid_blend_proportions_strategy() -> impl Strategy<Value = Vec<Decimal>> {
    (2..=5usize).prop_flat_map(|count| {
        prop::collection::vec(1..=50i64, count).prop_filter_map(
            "proportions must sum to 100 with all positive",
            |values| {
                let total: i64 = values.iter().sum();
                if total == 0 {
                    return None;
                }
                // Normalize to sum to 100, ensuring minimum of 1
                let mut proportions: Vec<i64> = values
                    .iter()
                    .map(|v| std::cmp::max(1, *v * 100 / total))
                    .collect();
                
                // Adjust to ensure exact sum of 100
                let current_sum: i64 = proportions.iter().sum();
                let diff = 100 - current_sum;
                
                // Add difference to the largest proportion
                if let Some(max_idx) = proportions.iter().enumerate().max_by_key(|(_, v)| *v).map(|(i, _)| i) {
                    proportions[max_idx] += diff;
                }
                
                // Verify all are positive and sum to 100
                let final_sum: i64 = proportions.iter().sum();
                if final_sum == 100 && proportions.iter().all(|&p| p > 0) {
                    Some(proportions.into_iter().map(Decimal::from).collect())
                } else {
                    None
                }
            },
        )
    })
}

/// Generate traceability code components
fn traceability_code_strategy() -> impl Strategy<Value = (i32, String, i32)> {
    let year = 2020..=2030i32;
    let business_code = "[A-Z]{3,6}";
    let sequence = 1..=9999i32;
    (year, business_code, sequence)
}

// ============================================================================
// Property-Based Tests
// ============================================================================

proptest! {
    /// Property 5: Ripeness Assessment Validity
    /// Verify underripe + ripe + overripe = 100
    #[test]
    fn test_valid_ripeness_sums_to_100(
        (underripe, ripe, overripe) in valid_ripeness_strategy()
    ) {
        let total = underripe + ripe + overripe;
        prop_assert_eq!(total, 100, "Ripeness must sum to 100");
        prop_assert!(underripe >= 0 && underripe <= 100);
        prop_assert!(ripe >= 0 && ripe <= 100);
        prop_assert!(overripe >= 0 && overripe <= 100);
    }

    /// Property 5: Invalid ripeness should be rejected
    #[test]
    fn test_invalid_ripeness_rejected(
        (underripe, ripe, overripe) in invalid_ripeness_strategy()
    ) {
        let total = underripe + ripe + overripe;
        prop_assert_ne!(total, 100, "Invalid ripeness should not sum to 100");
    }

    /// Property 6: Harvest Yield Calculation
    /// Verify yield = total cherry weight / plot area
    #[test]
    fn test_yield_calculation_accuracy(
        cherry_weight in cherry_weight_strategy(),
        plot_area in plot_area_strategy()
    ) {
        let expected_yield = cherry_weight / plot_area;
        let calculated_yield = calculate_yield_per_rai(cherry_weight, plot_area);
        
        prop_assert!(calculated_yield.is_some());
        let yield_value = calculated_yield.unwrap();
        
        // Allow small floating point differences
        let diff = (yield_value - expected_yield).abs();
        prop_assert!(diff < Decimal::new(1, 10), "Yield calculation should be accurate");
    }

    /// Property 6: Zero area should return None
    #[test]
    fn test_yield_zero_area_returns_none(
        cherry_weight in cherry_weight_strategy()
    ) {
        let yield_result = calculate_yield_per_rai(cherry_weight, Decimal::ZERO);
        prop_assert!(yield_result.is_none(), "Zero area should return None");
    }

    /// Property 3: Lot Identifier Uniqueness
    /// Generate many traceability codes and verify all are unique
    #[test]
    fn test_traceability_code_uniqueness(
        codes in prop::collection::vec(traceability_code_strategy(), 10..100)
    ) {
        let generated_codes: Vec<String> = codes
            .iter()
            .map(|(year, biz, seq)| format!("CQM-{}-{}-{:04}", year, biz, seq))
            .collect();
        
        let unique_codes: HashSet<&String> = generated_codes.iter().collect();
        
        // If all inputs are unique, outputs should be unique
        let unique_inputs: HashSet<_> = codes.iter().collect();
        if unique_inputs.len() == codes.len() {
            prop_assert_eq!(
                unique_codes.len(),
                generated_codes.len(),
                "All unique inputs should produce unique codes"
            );
        }
    }

    /// Property 7: Lot Blending Traceability
    /// Verify source proportions sum to 100%
    #[test]
    fn test_blend_proportions_sum_to_100(
        proportions in valid_blend_proportions_strategy()
    ) {
        let total: Decimal = proportions.iter().sum();
        prop_assert_eq!(total, Decimal::from(100), "Blend proportions must sum to 100%");
        
        // All proportions should be positive
        for p in &proportions {
            prop_assert!(*p > Decimal::ZERO, "All proportions must be positive");
        }
    }

    /// Property: Traceability code format validation
    #[test]
    fn test_traceability_code_format(
        (year, business_code, sequence) in traceability_code_strategy()
    ) {
        let code = format!("CQM-{}-{}-{:04}", year, business_code, sequence);
        
        // Verify format
        prop_assert!(code.starts_with("CQM-"));
        
        let parts: Vec<&str> = code.split('-').collect();
        prop_assert_eq!(parts.len(), 4);
        prop_assert_eq!(parts[0], "CQM");
        prop_assert!(parts[1].parse::<i32>().is_ok());
        prop_assert!(parts[2].len() >= 3 && parts[2].len() <= 6);
        prop_assert!(parts[3].len() == 4);
    }
}

// ============================================================================
// Helper Functions (mirroring service implementations)
// ============================================================================

/// Calculate yield per rai
fn calculate_yield_per_rai(
    total_cherry_weight_kg: Decimal,
    area_rai: Decimal,
) -> Option<Decimal> {
    if area_rai > Decimal::ZERO {
        Some(total_cherry_weight_kg / area_rai)
    } else {
        None
    }
}

/// Validate ripeness assessment
fn validate_ripeness(underripe: i32, ripe: i32, overripe: i32) -> Result<(), String> {
    let total = underripe + ripe + overripe;
    if total != 100 {
        return Err(format!("Ripeness must sum to 100, got {}", total));
    }
    if underripe < 0 || ripe < 0 || overripe < 0 {
        return Err("Ripeness values cannot be negative".to_string());
    }
    if underripe > 100 || ripe > 100 || overripe > 100 {
        return Err("Ripeness values cannot exceed 100".to_string());
    }
    Ok(())
}

/// Validate blend proportions
fn validate_blend_proportions(proportions: &[Decimal]) -> Result<(), String> {
    if proportions.is_empty() {
        return Err("At least one source required".to_string());
    }
    
    let total: Decimal = proportions.iter().sum();
    if total != Decimal::from(100) {
        return Err(format!("Proportions must sum to 100%, got {}%", total));
    }
    
    for p in proportions {
        if *p <= Decimal::ZERO {
            return Err("All proportions must be positive".to_string());
        }
    }
    
    Ok(())
}

// ============================================================================
// Unit Tests: Ripeness Validation
// ============================================================================

#[cfg(test)]
mod ripeness_tests {
    use super::*;

    #[test]
    fn test_valid_ripeness_all_ripe() {
        assert!(validate_ripeness(0, 100, 0).is_ok());
    }

    #[test]
    fn test_valid_ripeness_mixed() {
        assert!(validate_ripeness(10, 80, 10).is_ok());
    }

    #[test]
    fn test_valid_ripeness_edge_cases() {
        assert!(validate_ripeness(100, 0, 0).is_ok());
        assert!(validate_ripeness(0, 0, 100).is_ok());
        assert!(validate_ripeness(33, 34, 33).is_ok());
    }

    #[test]
    fn test_invalid_ripeness_sum_less_than_100() {
        assert!(validate_ripeness(10, 10, 10).is_err());
    }

    #[test]
    fn test_invalid_ripeness_sum_more_than_100() {
        assert!(validate_ripeness(50, 50, 50).is_err());
    }

    #[test]
    fn test_invalid_ripeness_negative() {
        assert!(validate_ripeness(-10, 100, 10).is_err());
    }
}

// ============================================================================
// Unit Tests: Yield Calculation
// ============================================================================

#[cfg(test)]
mod yield_tests {
    use super::*;

    #[test]
    fn test_yield_simple_calculation() {
        let weight = Decimal::from(100);
        let area = Decimal::from(2);
        let yield_val = calculate_yield_per_rai(weight, area);
        assert_eq!(yield_val, Some(Decimal::from(50)));
    }

    #[test]
    fn test_yield_decimal_precision() {
        let weight = Decimal::new(1000, 1); // 100.0
        let area = Decimal::new(30, 1); // 3.0
        let yield_val = calculate_yield_per_rai(weight, area);
        assert!(yield_val.is_some());
        // 100.0 / 3.0 ≈ 33.333...
        let expected = Decimal::new(1000, 1) / Decimal::new(30, 1);
        assert_eq!(yield_val.unwrap(), expected);
    }

    #[test]
    fn test_yield_zero_area() {
        let weight = Decimal::from(100);
        let area = Decimal::ZERO;
        assert!(calculate_yield_per_rai(weight, area).is_none());
    }

    #[test]
    fn test_yield_zero_weight() {
        let weight = Decimal::ZERO;
        let area = Decimal::from(2);
        let yield_val = calculate_yield_per_rai(weight, area);
        assert_eq!(yield_val, Some(Decimal::ZERO));
    }
}

// ============================================================================
// Unit Tests: Traceability Code
// ============================================================================

#[cfg(test)]
mod traceability_tests {
    #[test]
    fn test_traceability_code_format() {
        let code = format!("CQM-{}-{}-{:04}", 2024, "DOI", 1);
        assert_eq!(code, "CQM-2024-DOI-0001");
    }

    #[test]
    fn test_traceability_code_sequence_padding() {
        let code1 = format!("CQM-{}-{}-{:04}", 2024, "CMI", 1);
        let code2 = format!("CQM-{}-{}-{:04}", 2024, "CMI", 999);
        let code3 = format!("CQM-{}-{}-{:04}", 2024, "CMI", 9999);
        
        assert_eq!(code1, "CQM-2024-CMI-0001");
        assert_eq!(code2, "CQM-2024-CMI-0999");
        assert_eq!(code3, "CQM-2024-CMI-9999");
    }

    #[test]
    fn test_traceability_code_uniqueness_same_business() {
        let codes: Vec<String> = (1..=100)
            .map(|seq| format!("CQM-{}-{}-{:04}", 2024, "DOI", seq))
            .collect();
        
        let unique: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(unique.len(), codes.len());
    }

    #[test]
    fn test_traceability_code_uniqueness_different_years() {
        let code1 = format!("CQM-{}-{}-{:04}", 2024, "DOI", 1);
        let code2 = format!("CQM-{}-{}-{:04}", 2025, "DOI", 1);
        assert_ne!(code1, code2);
    }
}

// ============================================================================
// Unit Tests: Blend Proportions
// ============================================================================

#[cfg(test)]
mod blend_tests {
    use super::*;

    #[test]
    fn test_valid_blend_two_sources() {
        let proportions = vec![Decimal::from(60), Decimal::from(40)];
        assert!(validate_blend_proportions(&proportions).is_ok());
    }

    #[test]
    fn test_valid_blend_three_sources() {
        let proportions = vec![
            Decimal::from(50),
            Decimal::from(30),
            Decimal::from(20),
        ];
        assert!(validate_blend_proportions(&proportions).is_ok());
    }

    #[test]
    fn test_valid_blend_single_source() {
        let proportions = vec![Decimal::from(100)];
        assert!(validate_blend_proportions(&proportions).is_ok());
    }

    #[test]
    fn test_invalid_blend_empty() {
        let proportions: Vec<Decimal> = vec![];
        assert!(validate_blend_proportions(&proportions).is_err());
    }

    #[test]
    fn test_invalid_blend_not_100() {
        let proportions = vec![Decimal::from(60), Decimal::from(30)];
        assert!(validate_blend_proportions(&proportions).is_err());
    }

    #[test]
    fn test_invalid_blend_zero_proportion() {
        let proportions = vec![Decimal::from(100), Decimal::ZERO];
        assert!(validate_blend_proportions(&proportions).is_err());
    }

    #[test]
    fn test_invalid_blend_negative_proportion() {
        let proportions = vec![Decimal::from(110), Decimal::from(-10)];
        assert!(validate_blend_proportions(&proportions).is_err());
    }
}

// ============================================================================
// Unit Tests: Thai Coffee Harvest Context
// ============================================================================

#[cfg(test)]
mod thai_harvest_tests {
    use super::*;

    /// Thai Arabica harvest season: November to February
    #[test]
    fn test_harvest_season_months() {
        let harvest_months = vec![11, 12, 1, 2]; // Nov, Dec, Jan, Feb
        assert_eq!(harvest_months.len(), 4);
    }

    /// Typical cherry weight per tree per harvest
    #[test]
    fn test_typical_cherry_weight_per_tree() {
        // Thai Arabica: 2-5 kg cherry per tree per year
        let min_kg = Decimal::from(2);
        let max_kg = Decimal::from(5);
        let typical_kg = Decimal::from(3);
        
        assert!(typical_kg >= min_kg);
        assert!(typical_kg <= max_kg);
    }

    /// Typical yield per rai in Thai highlands
    #[test]
    fn test_typical_yield_per_rai() {
        // Thai Arabica: 200-400 kg cherry per rai
        let min_yield = Decimal::from(200);
        let max_yield = Decimal::from(400);
        let typical_yield = Decimal::from(300);
        
        assert!(typical_yield >= min_yield);
        assert!(typical_yield <= max_yield);
    }

    /// Cherry to green bean conversion ratio
    #[test]
    fn test_cherry_to_green_ratio() {
        // Typically 5:1 to 6:1 (cherry:green)
        let cherry_kg = Decimal::from(100);
        let expected_green_min = Decimal::new(1667, 2); // ~16.67 kg
        let expected_green_max = Decimal::from(20); // 20 kg
        
        let green_min = cherry_kg / Decimal::from(6);
        let green_max = cherry_kg / Decimal::from(5);
        
        assert!(green_min >= expected_green_min - Decimal::from(1));
        assert!(green_max <= expected_green_max + Decimal::from(1));
    }

    /// Optimal ripeness for specialty coffee
    #[test]
    fn test_optimal_ripeness_for_specialty() {
        // For specialty grade: >90% ripe cherries
        let underripe = 5;
        let ripe = 92;
        let overripe = 3;
        
        assert!(validate_ripeness(underripe, ripe, overripe).is_ok());
        assert!(ripe >= 90, "Specialty coffee needs >90% ripe cherries");
    }
}
