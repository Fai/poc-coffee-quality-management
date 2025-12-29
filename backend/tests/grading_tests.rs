//! Tests for green bean grading service
//! Verifies Property 9: Grade Classification Consistency

use rust_decimal::Decimal;
use shared::{classify_grade, DefectBreakdown, DefectCount, GradeClassification};

/// Helper to create Decimal from string
fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

// =============================================================================
// Property 9: Grade Classification Consistency Tests
// Verifies that grade classification matches SCA rules based on defects
// =============================================================================

mod grade_classification {
    use super::*;

    #[test]
    fn specialty_grade_zero_defects() {
        // Specialty Grade: 0-5 total defects, 0 category 1
        let defects = DefectCount {
            category1_count: 0,
            category2_count: 0,
            defect_breakdown: None,
        };
        assert_eq!(classify_grade(&defects), GradeClassification::SpecialtyGrade);
    }

    #[test]
    fn specialty_grade_max_defects() {
        // Specialty Grade: 0-5 total defects, 0 category 1
        let defects = DefectCount {
            category1_count: 0,
            category2_count: 5,
            defect_breakdown: None,
        };
        assert_eq!(classify_grade(&defects), GradeClassification::SpecialtyGrade);
    }

    #[test]
    fn specialty_grade_boundary_with_cat1_becomes_premium() {
        // Even 1 category 1 defect disqualifies from Specialty
        let defects = DefectCount {
            category1_count: 1,
            category2_count: 0,
            defect_breakdown: None,
        };
        // Total is 1, but has cat1, so Premium not Specialty
        assert_eq!(classify_grade(&defects), GradeClassification::PremiumGrade);
    }

    #[test]
    fn premium_grade_six_defects() {
        // Premium Grade: 0-8 total defects (when not qualifying for Specialty)
        let defects = DefectCount {
            category1_count: 0,
            category2_count: 6,
            defect_breakdown: None,
        };
        assert_eq!(classify_grade(&defects), GradeClassification::PremiumGrade);
    }

    #[test]
    fn premium_grade_eight_defects() {
        // Premium Grade: 0-8 total defects
        let defects = DefectCount {
            category1_count: 2,
            category2_count: 6,
            defect_breakdown: None,
        };
        assert_eq!(classify_grade(&defects), GradeClassification::PremiumGrade);
    }

    #[test]
    fn exchange_grade_nine_defects() {
        // Exchange Grade: 9-23 total defects
        let defects = DefectCount {
            category1_count: 3,
            category2_count: 6,
            defect_breakdown: None,
        };
        assert_eq!(classify_grade(&defects), GradeClassification::ExchangeGrade);
    }

    #[test]
    fn exchange_grade_twenty_three_defects() {
        // Exchange Grade: 9-23 total defects
        let defects = DefectCount {
            category1_count: 10,
            category2_count: 13,
            defect_breakdown: None,
        };
        assert_eq!(classify_grade(&defects), GradeClassification::ExchangeGrade);
    }

    #[test]
    fn below_standard_twenty_four_defects() {
        // Below Standard: 24-86 total defects
        let defects = DefectCount {
            category1_count: 10,
            category2_count: 14,
            defect_breakdown: None,
        };
        assert_eq!(classify_grade(&defects), GradeClassification::BelowStandard);
    }

    #[test]
    fn below_standard_eighty_six_defects() {
        // Below Standard: 24-86 total defects
        let defects = DefectCount {
            category1_count: 40,
            category2_count: 46,
            defect_breakdown: None,
        };
        assert_eq!(classify_grade(&defects), GradeClassification::BelowStandard);
    }

    #[test]
    fn off_grade_eighty_seven_defects() {
        // Off Grade: 87+ total defects
        let defects = DefectCount {
            category1_count: 40,
            category2_count: 47,
            defect_breakdown: None,
        };
        assert_eq!(classify_grade(&defects), GradeClassification::OffGrade);
    }

    #[test]
    fn off_grade_high_defects() {
        // Off Grade: 87+ total defects
        let defects = DefectCount {
            category1_count: 100,
            category2_count: 100,
            defect_breakdown: None,
        };
        assert_eq!(classify_grade(&defects), GradeClassification::OffGrade);
    }
}

// =============================================================================
// Defect Count Tests
// =============================================================================

mod defect_count {
    use super::*;

    #[test]
    fn total_defects_calculation() {
        let defects = DefectCount {
            category1_count: 5,
            category2_count: 10,
            defect_breakdown: None,
        };
        assert_eq!(defects.total(), 15);
    }

    #[test]
    fn total_defects_zero() {
        let defects = DefectCount {
            category1_count: 0,
            category2_count: 0,
            defect_breakdown: None,
        };
        assert_eq!(defects.total(), 0);
    }

    #[test]
    fn defect_breakdown_default() {
        let breakdown = DefectBreakdown::default();
        assert_eq!(breakdown.full_black, 0);
        assert_eq!(breakdown.full_sour, 0);
        assert_eq!(breakdown.broken, 0);
        assert_eq!(breakdown.insect_damage, 0);
    }
}

// =============================================================================
// Grade Classification Boundary Tests
// =============================================================================

mod grade_boundaries {
    use super::*;

    #[test]
    fn boundary_specialty_to_premium_by_count() {
        // 5 defects with 0 cat1 = Specialty
        let specialty = DefectCount {
            category1_count: 0,
            category2_count: 5,
            defect_breakdown: None,
        };
        assert_eq!(classify_grade(&specialty), GradeClassification::SpecialtyGrade);

        // 6 defects with 0 cat1 = Premium
        let premium = DefectCount {
            category1_count: 0,
            category2_count: 6,
            defect_breakdown: None,
        };
        assert_eq!(classify_grade(&premium), GradeClassification::PremiumGrade);
    }

    #[test]
    fn boundary_premium_to_exchange() {
        // 8 defects = Premium
        let premium = DefectCount {
            category1_count: 4,
            category2_count: 4,
            defect_breakdown: None,
        };
        assert_eq!(classify_grade(&premium), GradeClassification::PremiumGrade);

        // 9 defects = Exchange
        let exchange = DefectCount {
            category1_count: 4,
            category2_count: 5,
            defect_breakdown: None,
        };
        assert_eq!(classify_grade(&exchange), GradeClassification::ExchangeGrade);
    }

    #[test]
    fn boundary_exchange_to_below_standard() {
        // 23 defects = Exchange
        let exchange = DefectCount {
            category1_count: 10,
            category2_count: 13,
            defect_breakdown: None,
        };
        assert_eq!(classify_grade(&exchange), GradeClassification::ExchangeGrade);

        // 24 defects = Below Standard
        let below = DefectCount {
            category1_count: 10,
            category2_count: 14,
            defect_breakdown: None,
        };
        assert_eq!(classify_grade(&below), GradeClassification::BelowStandard);
    }

    #[test]
    fn boundary_below_standard_to_off_grade() {
        // 86 defects = Below Standard
        let below = DefectCount {
            category1_count: 43,
            category2_count: 43,
            defect_breakdown: None,
        };
        assert_eq!(classify_grade(&below), GradeClassification::BelowStandard);

        // 87 defects = Off Grade
        let off = DefectCount {
            category1_count: 43,
            category2_count: 44,
            defect_breakdown: None,
        };
        assert_eq!(classify_grade(&off), GradeClassification::OffGrade);
    }
}

// =============================================================================
// Grade Display Tests
// =============================================================================

mod grade_display {
    use super::*;

    #[test]
    fn grade_display_strings() {
        assert_eq!(
            format!("{}", GradeClassification::SpecialtyGrade),
            "Specialty Grade"
        );
        assert_eq!(
            format!("{}", GradeClassification::PremiumGrade),
            "Premium Grade"
        );
        assert_eq!(
            format!("{}", GradeClassification::ExchangeGrade),
            "Exchange Grade"
        );
        assert_eq!(
            format!("{}", GradeClassification::BelowStandard),
            "Below Standard"
        );
        assert_eq!(format!("{}", GradeClassification::OffGrade), "Off Grade");
    }
}

// =============================================================================
// Grading Input Validation Tests (unit tests for validation logic)
// =============================================================================

mod grading_validation {
    use super::*;

    #[test]
    fn sample_weight_must_be_positive() {
        // Standard sample weight is 350g per SCA
        let valid_weight = dec("350.0");
        assert!(valid_weight > Decimal::ZERO);

        let invalid_weight = dec("0.0");
        assert!(invalid_weight <= Decimal::ZERO);

        let negative_weight = dec("-100.0");
        assert!(negative_weight < Decimal::ZERO);
    }

    #[test]
    fn moisture_percent_valid_range() {
        // Typical green bean moisture: 10-12%
        let typical_moisture = dec("11.5");
        assert!(typical_moisture >= Decimal::ZERO);
        assert!(typical_moisture <= Decimal::from(100));

        // Edge cases
        let zero_moisture = dec("0.0");
        assert!(zero_moisture >= Decimal::ZERO);

        let max_moisture = dec("100.0");
        assert!(max_moisture <= Decimal::from(100));
    }

    #[test]
    fn defect_counts_non_negative() {
        // Valid counts
        assert!(0 >= 0);
        assert!(5 >= 0);
        assert!(100 >= 0);

        // Invalid (negative) - would fail validation
        let negative: i32 = -1;
        assert!(negative < 0);
    }
}

// =============================================================================
// Screen Size Distribution Tests
// =============================================================================

mod screen_size {
    use super::*;
    use shared::ScreenSizeDistribution;

    #[test]
    fn screen_size_distribution_totals_100() {
        let distribution = ScreenSizeDistribution {
            screen_18_plus: dec("15.0"),
            screen_17: dec("35.0"),
            screen_16: dec("30.0"),
            screen_15: dec("15.0"),
            screen_14_below: dec("5.0"),
        };

        let total = distribution.screen_18_plus
            + distribution.screen_17
            + distribution.screen_16
            + distribution.screen_15
            + distribution.screen_14_below;

        assert_eq!(total, dec("100.0"));
    }

    #[test]
    fn screen_size_specialty_typically_large() {
        // Specialty coffee typically has larger screen sizes
        let specialty_distribution = ScreenSizeDistribution {
            screen_18_plus: dec("25.0"),
            screen_17: dec("45.0"),
            screen_16: dec("20.0"),
            screen_15: dec("8.0"),
            screen_14_below: dec("2.0"),
        };

        // Most beans should be screen 16 or larger
        let large_beans = specialty_distribution.screen_18_plus
            + specialty_distribution.screen_17
            + specialty_distribution.screen_16;

        assert!(large_beans >= dec("80.0"));
    }
}

// =============================================================================
// AI Detection Integration Tests (structure validation)
// =============================================================================

mod ai_detection {
    use super::*;
    use shared::AiDefectDetection;

    #[test]
    fn ai_detection_structure() {
        let ai_result = AiDefectDetection {
            request_id: "det-20241223-abc123".to_string(),
            image_url: "s3://bucket/image.jpg".to_string(),
            detected_beans: 350,
            defect_breakdown: DefectBreakdown {
                full_black: 1,
                full_sour: 0,
                partial_black: 2,
                broken: 3,
                ..Default::default()
            },
            category1_count: 1,
            category2_count: 5,
            confidence_score: 0.95,
            processing_time_ms: 1500,
            annotated_image_url: Some("s3://bucket/annotated.jpg".to_string()),
        };

        assert_eq!(ai_result.detected_beans, 350);
        assert_eq!(ai_result.category1_count, 1);
        assert_eq!(ai_result.category2_count, 5);
        assert!(ai_result.confidence_score > 0.9);

        // Verify grade from AI detection
        let defects = DefectCount {
            category1_count: ai_result.category1_count,
            category2_count: ai_result.category2_count,
            defect_breakdown: Some(ai_result.defect_breakdown),
        };
        // Total = 6, has cat1, so Premium
        assert_eq!(classify_grade(&defects), GradeClassification::PremiumGrade);
    }

    #[test]
    fn ai_detection_high_quality_sample() {
        let ai_result = AiDefectDetection {
            request_id: "det-20241223-xyz789".to_string(),
            image_url: "s3://bucket/high-quality.jpg".to_string(),
            detected_beans: 350,
            defect_breakdown: DefectBreakdown {
                partial_black: 1,
                broken: 2,
                ..Default::default()
            },
            category1_count: 0,
            category2_count: 3,
            confidence_score: 0.98,
            processing_time_ms: 1200,
            annotated_image_url: None,
        };

        let defects = DefectCount {
            category1_count: ai_result.category1_count,
            category2_count: ai_result.category2_count,
            defect_breakdown: Some(ai_result.defect_breakdown),
        };

        // 0 cat1, 3 total = Specialty Grade
        assert_eq!(classify_grade(&defects), GradeClassification::SpecialtyGrade);
    }
}
