//! Processing tests for the Coffee Quality Management Platform
//!
//! Feature: coffee-quality-management
//! Tests for processing management including Property 8: Processing Yield Calculation

use proptest::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Helper to create Decimal from string
fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

/// Calculate processing yield percentage
/// Yield = (green_bean_weight / cherry_weight) × 100
fn calculate_processing_yield(cherry_weight: Decimal, green_bean_weight: Decimal) -> Decimal {
    if cherry_weight.is_zero() {
        Decimal::ZERO
    } else {
        (green_bean_weight / cherry_weight) * Decimal::from(100)
    }
}

// ============================================================================
// Property 8: Processing Yield Calculation
// ============================================================================
// For any completed processing record with cherry weight C and green bean weight G,
// the processing yield SHALL equal (G / C) × 100.
// Validates: Requirements 4.5

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 8: Processing Yield Calculation
    /// Verify yield = (green_bean_weight / cherry_weight) × 100
    #[test]
    fn property_8_processing_yield_calculation(
        cherry_kg in 1u32..10000,
        yield_percent in 15u32..25, // Typical coffee yield is 15-25%
    ) {
        let cherry_weight = Decimal::from(cherry_kg);
        // Calculate green bean weight from yield percentage
        let expected_yield = Decimal::from(yield_percent);
        let green_bean_weight = cherry_weight * expected_yield / Decimal::from(100);

        let calculated_yield = calculate_processing_yield(cherry_weight, green_bean_weight);

        // Verify the formula: yield = (green_bean / cherry) * 100
        let tolerance = dec("0.0001");
        let diff = (calculated_yield - expected_yield).abs();
        prop_assert!(
            diff < tolerance,
            "Yield calculation mismatch: expected {}, got {}, diff {}",
            expected_yield,
            calculated_yield,
            diff
        );
    }

    /// Property 8 variant: Verify yield is always positive when inputs are positive
    #[test]
    fn property_8_yield_always_positive(
        cherry_kg in 1u32..10000,
        green_kg in 1u32..5000,
    ) {
        let cherry_weight = Decimal::from(cherry_kg);
        let green_bean_weight = Decimal::from(green_kg);

        let yield_percent = calculate_processing_yield(cherry_weight, green_bean_weight);

        prop_assert!(
            yield_percent > Decimal::ZERO,
            "Yield should be positive for positive inputs"
        );
    }

    /// Property 8 variant: Verify yield is zero when cherry weight is zero
    #[test]
    fn property_8_yield_zero_for_zero_cherry(
        green_kg in 0u32..5000,
    ) {
        let cherry_weight = Decimal::ZERO;
        let green_bean_weight = Decimal::from(green_kg);

        let yield_percent = calculate_processing_yield(cherry_weight, green_bean_weight);

        prop_assert_eq!(
            yield_percent,
            Decimal::ZERO,
            "Yield should be zero when cherry weight is zero"
        );
    }
}

// ============================================================================
// Unit Tests for Processing Yield
// ============================================================================

#[test]
fn test_typical_washed_processing_yield() {
    // Typical washed process: 18-20% yield
    let cherry_weight = dec("100.0");
    let green_bean_weight = dec("18.5");

    let yield_percent = calculate_processing_yield(cherry_weight, green_bean_weight);

    assert_eq!(yield_percent, dec("18.5"));
}

#[test]
fn test_typical_natural_processing_yield() {
    // Natural process typically has slightly higher yield: 20-22%
    let cherry_weight = dec("100.0");
    let green_bean_weight = dec("21.0");

    let yield_percent = calculate_processing_yield(cherry_weight, green_bean_weight);

    assert_eq!(yield_percent, dec("21.0"));
}

#[test]
fn test_honey_processing_yield() {
    // Honey process: 19-21% yield
    let cherry_weight = dec("500.0");
    let green_bean_weight = dec("100.0");

    let yield_percent = calculate_processing_yield(cherry_weight, green_bean_weight);

    assert_eq!(yield_percent, dec("20.0"));
}

#[test]
fn test_large_batch_processing_yield() {
    // Large batch: 1000kg cherry -> 185kg green bean (18.5% yield)
    let cherry_weight = dec("1000.0");
    let green_bean_weight = dec("185.0");

    let yield_percent = calculate_processing_yield(cherry_weight, green_bean_weight);

    assert_eq!(yield_percent, dec("18.5"));
}

#[test]
fn test_small_batch_processing_yield() {
    // Small batch: 10kg cherry -> 2kg green bean (20% yield)
    let cherry_weight = dec("10.0");
    let green_bean_weight = dec("2.0");

    let yield_percent = calculate_processing_yield(cherry_weight, green_bean_weight);

    assert_eq!(yield_percent, dec("20.0"));
}

#[test]
fn test_zero_green_bean_weight() {
    let cherry_weight = dec("100.0");
    let green_bean_weight = Decimal::ZERO;

    let yield_percent = calculate_processing_yield(cherry_weight, green_bean_weight);

    assert_eq!(yield_percent, Decimal::ZERO);
}

#[test]
fn test_zero_cherry_weight() {
    let cherry_weight = Decimal::ZERO;
    let green_bean_weight = dec("20.0");

    let yield_percent = calculate_processing_yield(cherry_weight, green_bean_weight);

    assert_eq!(yield_percent, Decimal::ZERO);
}

#[test]
fn test_decimal_precision() {
    // Test with precise decimal values
    let cherry_weight = dec("123.456");
    let green_bean_weight = dec("24.6912"); // 20% of 123.456

    let yield_percent = calculate_processing_yield(cherry_weight, green_bean_weight);

    assert_eq!(yield_percent, dec("20.0"));
}

// ============================================================================
// Processing Method Tests
// ============================================================================

#[test]
fn test_processing_method_display() {
    use shared::ProcessingMethod;

    assert_eq!(ProcessingMethod::Natural.to_string(), "Natural");
    assert_eq!(ProcessingMethod::Washed.to_string(), "Washed");
    assert_eq!(
        ProcessingMethod::Honey { mucilage_percent: 50 }.to_string(),
        "Honey (50% mucilage)"
    );
    assert_eq!(ProcessingMethod::WetHulled.to_string(), "Wet Hulled");
    assert_eq!(
        ProcessingMethod::Anaerobic { hours: 72 }.to_string(),
        "Anaerobic (72h)"
    );
    assert_eq!(
        ProcessingMethod::Custom("Carbonic Maceration".to_string()).to_string(),
        "Carbonic Maceration"
    );
}

// ============================================================================
// Fermentation Log Tests
// ============================================================================

#[test]
fn test_fermentation_log_serialization() {
    use chrono::Utc;
    use shared::{FermentationLog, PhReading, TemperatureReading};

    let log = FermentationLog {
        duration_hours: 48,
        temperature_readings: vec![
            TemperatureReading {
                timestamp: Utc::now(),
                temperature_celsius: dec("25.5"),
            },
            TemperatureReading {
                timestamp: Utc::now(),
                temperature_celsius: dec("26.0"),
            },
        ],
        ph_readings: vec![
            PhReading {
                timestamp: Utc::now(),
                ph_value: dec("4.5"),
            },
            PhReading {
                timestamp: Utc::now(),
                ph_value: dec("4.2"),
            },
        ],
    };

    let json = serde_json::to_string(&log).unwrap();
    let deserialized: FermentationLog = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.duration_hours, 48);
    assert_eq!(deserialized.temperature_readings.len(), 2);
    assert_eq!(deserialized.ph_readings.len(), 2);
}

// ============================================================================
// Drying Log Tests
// ============================================================================

#[test]
fn test_drying_log_serialization() {
    use chrono::{NaiveDate, Utc};
    use shared::{DryingLog, DryingMethod, MoistureReading};

    let log = DryingLog {
        method: DryingMethod::RaisedBed,
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end_date: Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
        target_moisture_percent: dec("11.0"),
        moisture_readings: vec![
            MoistureReading {
                timestamp: Utc::now(),
                moisture_percent: dec("45.0"),
            },
            MoistureReading {
                timestamp: Utc::now(),
                moisture_percent: dec("25.0"),
            },
            MoistureReading {
                timestamp: Utc::now(),
                moisture_percent: dec("11.5"),
            },
        ],
    };

    let json = serde_json::to_string(&log).unwrap();
    let deserialized: DryingLog = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.method, DryingMethod::RaisedBed);
    assert_eq!(deserialized.target_moisture_percent, dec("11.0"));
    assert_eq!(deserialized.moisture_readings.len(), 3);
}

#[test]
fn test_drying_methods() {
    use shared::DryingMethod;

    // Test all drying methods can be serialized/deserialized
    let methods = vec![
        DryingMethod::RaisedBed,
        DryingMethod::Patio,
        DryingMethod::Mechanical,
        DryingMethod::Greenhouse,
        DryingMethod::Custom("Solar Dryer".to_string()),
    ];

    for method in methods {
        let json = serde_json::to_string(&method).unwrap();
        let deserialized: DryingMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, method);
    }
}
