//! Roast profile management tests
//!
//! Tests for roasting operations including:
//! - Property 16: Roast Weight Loss Calculation

use proptest::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;

// Helper to create Decimal from string
fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;

    /// Test weight loss calculation formula
    /// weight_loss = ((green - roasted) / green) × 100
    #[test]
    fn test_weight_loss_calculation_basic() {
        let green_weight = dec("100.0");
        let roasted_weight = dec("85.0");
        
        let weight_loss = calculate_weight_loss(green_weight, roasted_weight);
        
        // (100 - 85) / 100 * 100 = 15%
        assert_eq!(weight_loss, dec("15.0"));
    }

    #[test]
    fn test_weight_loss_calculation_typical_light_roast() {
        // Light roast typically loses 12-14%
        let green_weight = dec("500.0");
        let roasted_weight = dec("435.0"); // 13% loss
        
        let weight_loss = calculate_weight_loss(green_weight, roasted_weight);
        
        // (500 - 435) / 500 * 100 = 13%
        assert_eq!(weight_loss, dec("13.0"));
    }

    #[test]
    fn test_weight_loss_calculation_typical_dark_roast() {
        // Dark roast typically loses 18-22%
        let green_weight = dec("500.0");
        let roasted_weight = dec("400.0"); // 20% loss
        
        let weight_loss = calculate_weight_loss(green_weight, roasted_weight);
        
        // (500 - 400) / 500 * 100 = 20%
        assert_eq!(weight_loss, dec("20.0"));
    }

    #[test]
    fn test_weight_loss_calculation_with_decimals() {
        let green_weight = dec("123.456");
        let roasted_weight = dec("100.000");
        
        let weight_loss = calculate_weight_loss(green_weight, roasted_weight);
        
        // (123.456 - 100) / 123.456 * 100 ≈ 18.99%
        assert!(weight_loss > dec("18.0") && weight_loss < dec("20.0"));
    }

    #[test]
    fn test_weight_loss_zero_green_weight() {
        let green_weight = Decimal::ZERO;
        let roasted_weight = dec("85.0");
        
        let weight_loss = calculate_weight_loss(green_weight, roasted_weight);
        
        // Should return 0 to avoid division by zero
        assert_eq!(weight_loss, Decimal::ZERO);
    }

    /// Test development time ratio (DTR) calculation
    /// DTR = (development_time / total_time) × 100
    #[test]
    fn test_dtr_calculation_basic() {
        let development_time = 90; // seconds after first crack
        let total_time = 600; // 10 minutes total
        
        let dtr = calculate_dtr(development_time, total_time);
        
        // 90 / 600 * 100 = 15%
        assert_eq!(dtr, dec("15.0"));
    }

    #[test]
    fn test_dtr_calculation_typical_light() {
        // Light roast: DTR typically 15-20%
        let development_time = 60;
        let total_time = 480; // 8 minutes
        
        let dtr = calculate_dtr(development_time, total_time);
        
        // 60 / 480 * 100 = 12.5%
        assert_eq!(dtr, dec("12.5"));
    }

    #[test]
    fn test_dtr_calculation_typical_dark() {
        // Dark roast: DTR typically 20-25%
        let development_time = 150;
        let total_time = 720; // 12 minutes
        
        let dtr = calculate_dtr(development_time, total_time);
        
        // 150 / 720 * 100 ≈ 20.83%
        assert!(dtr > dec("20.0") && dtr < dec("21.0"));
    }

    #[test]
    fn test_dtr_zero_total_time() {
        let development_time = 90;
        let total_time = 0;
        
        let dtr = calculate_dtr(development_time, total_time);
        
        // Should return 0 to avoid division by zero
        assert_eq!(dtr, Decimal::ZERO);
    }

    /// Test roast status transitions
    #[test]
    fn test_roast_status_values() {
        assert_eq!(RoastStatus::InProgress.as_str(), "in_progress");
        assert_eq!(RoastStatus::Completed.as_str(), "completed");
        assert_eq!(RoastStatus::Failed.as_str(), "failed");
    }

    #[test]
    fn test_roast_status_from_str() {
        assert_eq!(RoastStatus::from_str("in_progress"), Some(RoastStatus::InProgress));
        assert_eq!(RoastStatus::from_str("completed"), Some(RoastStatus::Completed));
        assert_eq!(RoastStatus::from_str("failed"), Some(RoastStatus::Failed));
        assert_eq!(RoastStatus::from_str("invalid"), None);
    }

    /// Test roast level values
    #[test]
    fn test_roast_level_values() {
        assert_eq!(RoastLevel::Light.as_str(), "light");
        assert_eq!(RoastLevel::MediumLight.as_str(), "medium_light");
        assert_eq!(RoastLevel::Medium.as_str(), "medium");
        assert_eq!(RoastLevel::MediumDark.as_str(), "medium_dark");
        assert_eq!(RoastLevel::Dark.as_str(), "dark");
    }

    /// Test temperature checkpoint ordering
    #[test]
    fn test_temperature_checkpoint_ordering() {
        let mut checkpoints = vec![
            TemperatureCheckpoint { time_seconds: 300, temp_celsius: dec("180.0"), notes: None },
            TemperatureCheckpoint { time_seconds: 60, temp_celsius: dec("100.0"), notes: None },
            TemperatureCheckpoint { time_seconds: 180, temp_celsius: dec("150.0"), notes: None },
        ];
        
        checkpoints.sort_by_key(|c| c.time_seconds);
        
        assert_eq!(checkpoints[0].time_seconds, 60);
        assert_eq!(checkpoints[1].time_seconds, 180);
        assert_eq!(checkpoints[2].time_seconds, 300);
    }

    /// Test weight loss is always positive when roasted < green
    #[test]
    fn test_weight_loss_always_positive() {
        let green_weight = dec("100.0");
        let roasted_weight = dec("80.0");
        
        let weight_loss = calculate_weight_loss(green_weight, roasted_weight);
        
        assert!(weight_loss > Decimal::ZERO);
    }

    /// Test typical weight loss ranges
    #[test]
    fn test_weight_loss_typical_ranges() {
        // Light roast: 12-14%
        let light_loss = calculate_weight_loss(dec("100.0"), dec("87.0"));
        assert!(light_loss >= dec("12.0") && light_loss <= dec("14.0"));
        
        // Medium roast: 14-16%
        let medium_loss = calculate_weight_loss(dec("100.0"), dec("85.0"));
        assert!(medium_loss >= dec("14.0") && medium_loss <= dec("16.0"));
        
        // Dark roast: 18-22%
        let dark_loss = calculate_weight_loss(dec("100.0"), dec("80.0"));
        assert!(dark_loss >= dec("18.0") && dark_loss <= dec("22.0"));
    }

    // Helper functions for tests
    pub fn calculate_weight_loss(green_weight: Decimal, roasted_weight: Decimal) -> Decimal {
        if green_weight.is_zero() {
            return Decimal::ZERO;
        }
        ((green_weight - roasted_weight) / green_weight) * Decimal::from(100)
    }

    pub fn calculate_dtr(development_time: i32, total_time: i32) -> Decimal {
        if total_time <= 0 {
            return Decimal::ZERO;
        }
        (Decimal::from(development_time) / Decimal::from(total_time)) * Decimal::from(100)
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RoastStatus {
        InProgress,
        Completed,
        Failed,
    }

    impl RoastStatus {
        pub fn as_str(&self) -> &'static str {
            match self {
                RoastStatus::InProgress => "in_progress",
                RoastStatus::Completed => "completed",
                RoastStatus::Failed => "failed",
            }
        }

        pub fn from_str(s: &str) -> Option<Self> {
            match s {
                "in_progress" => Some(RoastStatus::InProgress),
                "completed" => Some(RoastStatus::Completed),
                "failed" => Some(RoastStatus::Failed),
                _ => None,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RoastLevel {
        Light,
        MediumLight,
        Medium,
        MediumDark,
        Dark,
    }

    impl RoastLevel {
        pub fn as_str(&self) -> &'static str {
            match self {
                RoastLevel::Light => "light",
                RoastLevel::MediumLight => "medium_light",
                RoastLevel::Medium => "medium",
                RoastLevel::MediumDark => "medium_dark",
                RoastLevel::Dark => "dark",
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct TemperatureCheckpoint {
        pub time_seconds: i32,
        pub temp_celsius: Decimal,
        pub notes: Option<String>,
    }
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use super::unit_tests::{calculate_weight_loss, calculate_dtr};

    /// Strategy for generating valid green bean weights (positive decimals)
    fn green_weight_strategy() -> impl Strategy<Value = Decimal> {
        (100i64..=10000i64).prop_map(|n| Decimal::new(n, 1)) // 10.0 to 1000.0 kg
    }

    /// Strategy for generating weight loss percentages (typical roast range)
    fn weight_loss_percent_strategy() -> impl Strategy<Value = Decimal> {
        (100i64..=250i64).prop_map(|n| Decimal::new(n, 1)) // 10.0% to 25.0%
    }

    /// Strategy for generating development times (seconds)
    fn development_time_strategy() -> impl Strategy<Value = i32> {
        30..=300i32 // 30 seconds to 5 minutes
    }

    /// Strategy for generating total roast times (seconds)
    fn total_time_strategy() -> impl Strategy<Value = i32> {
        300..=1200i32 // 5 to 20 minutes
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 16: Roast Weight Loss Calculation
        /// Verify weight_loss = ((green - roasted) / green) × 100
        #[test]
        fn prop_weight_loss_calculation_correct(
            green_weight in green_weight_strategy(),
            loss_percent in weight_loss_percent_strategy()
        ) {
            // Calculate roasted weight from loss percentage
            let roasted_weight = green_weight * (Decimal::from(100) - loss_percent) / Decimal::from(100);
            
            // Calculate weight loss using our function
            let calculated_loss = calculate_weight_loss(green_weight, roasted_weight);
            
            // Should match the original loss percentage (within rounding tolerance)
            let diff = (calculated_loss - loss_percent).abs();
            prop_assert!(diff < dec("0.01"), 
                "Weight loss calculation mismatch: expected {}, got {}", 
                loss_percent, calculated_loss);
        }

        /// Property 16: Weight loss is always between 0 and 100 when roasted < green
        #[test]
        fn prop_weight_loss_bounded(
            green_weight in green_weight_strategy(),
            loss_percent in weight_loss_percent_strategy()
        ) {
            let roasted_weight = green_weight * (Decimal::from(100) - loss_percent) / Decimal::from(100);
            let weight_loss = calculate_weight_loss(green_weight, roasted_weight);
            
            prop_assert!(weight_loss >= Decimal::ZERO);
            prop_assert!(weight_loss <= Decimal::from(100));
        }

        /// Property 16: Weight loss increases as roasted weight decreases
        #[test]
        fn prop_weight_loss_monotonic(
            green_weight in green_weight_strategy(),
            loss1 in 100i64..=150i64,
            loss2 in 151i64..=250i64
        ) {
            let loss_percent1 = Decimal::new(loss1, 1);
            let loss_percent2 = Decimal::new(loss2, 1);
            
            let roasted1 = green_weight * (Decimal::from(100) - loss_percent1) / Decimal::from(100);
            let roasted2 = green_weight * (Decimal::from(100) - loss_percent2) / Decimal::from(100);
            
            let weight_loss1 = calculate_weight_loss(green_weight, roasted1);
            let weight_loss2 = calculate_weight_loss(green_weight, roasted2);
            
            // Higher loss percentage should result in higher calculated loss
            prop_assert!(weight_loss2 > weight_loss1);
        }

        /// Property: DTR calculation is correct
        #[test]
        fn prop_dtr_calculation_correct(
            dev_time in development_time_strategy(),
            total_time in total_time_strategy()
        ) {
            // Ensure development time is less than total time
            if dev_time >= total_time {
                return Ok(());
            }
            
            let dtr = calculate_dtr(dev_time, total_time);
            
            // DTR should be between 0 and 100
            prop_assert!(dtr >= Decimal::ZERO);
            prop_assert!(dtr <= Decimal::from(100));
            
            // Verify calculation: DTR = (dev_time / total_time) * 100
            let expected = (Decimal::from(dev_time) / Decimal::from(total_time)) * Decimal::from(100);
            prop_assert_eq!(dtr, expected);
        }

        /// Property: DTR is bounded by development time ratio
        #[test]
        fn prop_dtr_bounded(
            dev_time in development_time_strategy(),
            total_time in total_time_strategy()
        ) {
            if dev_time >= total_time || total_time <= 0 {
                return Ok(());
            }
            
            let dtr = calculate_dtr(dev_time, total_time);
            
            // DTR should be positive
            prop_assert!(dtr > Decimal::ZERO);
            
            // DTR should be less than 100 (since dev_time < total_time)
            prop_assert!(dtr < Decimal::from(100));
        }

        /// Property: Weight loss formula is reversible
        #[test]
        fn prop_weight_loss_reversible(
            green_weight in green_weight_strategy(),
            loss_percent in weight_loss_percent_strategy()
        ) {
            // Calculate roasted weight
            let roasted_weight = green_weight * (Decimal::from(100) - loss_percent) / Decimal::from(100);
            
            // Calculate weight loss
            let calculated_loss = calculate_weight_loss(green_weight, roasted_weight);
            
            // Reverse: calculate roasted weight from loss
            let reversed_roasted = green_weight * (Decimal::from(100) - calculated_loss) / Decimal::from(100);
            
            // Should match original roasted weight (within tolerance)
            let diff = (reversed_roasted - roasted_weight).abs();
            prop_assert!(diff < dec("0.001"));
        }

        /// Property: Zero green weight returns zero loss
        #[test]
        fn prop_zero_green_weight_returns_zero(roasted in green_weight_strategy()) {
            let weight_loss = calculate_weight_loss(Decimal::ZERO, roasted);
            prop_assert_eq!(weight_loss, Decimal::ZERO);
        }

        /// Property: Zero total time returns zero DTR
        #[test]
        fn prop_zero_total_time_returns_zero(dev_time in development_time_strategy()) {
            let dtr = calculate_dtr(dev_time, 0);
            prop_assert_eq!(dtr, Decimal::ZERO);
        }

        /// Property: Weight loss with equal weights is zero
        #[test]
        fn prop_equal_weights_zero_loss(weight in green_weight_strategy()) {
            let weight_loss = calculate_weight_loss(weight, weight);
            prop_assert_eq!(weight_loss, Decimal::ZERO);
        }
    }
}

// ============================================================================
// Integration Test Helpers
// ============================================================================

#[cfg(test)]
mod integration_helpers {
    use super::*;
    use super::unit_tests::{calculate_weight_loss, calculate_dtr, RoastStatus, RoastLevel};

    /// Simulate a complete roast session
    pub fn simulate_roast_session(
        green_weight: Decimal,
        roasted_weight: Decimal,
        first_crack_time: i32,
        drop_time: i32,
    ) -> Result<RoastSessionResult, &'static str> {
        // Validate inputs
        if green_weight <= Decimal::ZERO {
            return Err("Green weight must be positive");
        }
        if roasted_weight <= Decimal::ZERO {
            return Err("Roasted weight must be positive");
        }
        if roasted_weight >= green_weight {
            return Err("Roasted weight must be less than green weight");
        }
        if first_crack_time <= 0 {
            return Err("First crack time must be positive");
        }
        if drop_time <= first_crack_time {
            return Err("Drop time must be after first crack");
        }

        let weight_loss = calculate_weight_loss(green_weight, roasted_weight);
        let development_time = drop_time - first_crack_time;
        let dtr = calculate_dtr(development_time, drop_time);

        // Determine roast level based on weight loss
        let roast_level = if weight_loss < dec("14.0") {
            RoastLevel::Light
        } else if weight_loss < dec("16.0") {
            RoastLevel::MediumLight
        } else if weight_loss < dec("18.0") {
            RoastLevel::Medium
        } else if weight_loss < dec("20.0") {
            RoastLevel::MediumDark
        } else {
            RoastLevel::Dark
        };

        Ok(RoastSessionResult {
            green_weight,
            roasted_weight,
            weight_loss_percent: weight_loss,
            first_crack_time,
            drop_time,
            development_time,
            dtr,
            roast_level,
            status: RoastStatus::Completed,
        })
    }

    #[derive(Debug)]
    pub struct RoastSessionResult {
        pub green_weight: Decimal,
        pub roasted_weight: Decimal,
        pub weight_loss_percent: Decimal,
        pub first_crack_time: i32,
        pub drop_time: i32,
        pub development_time: i32,
        pub dtr: Decimal,
        pub roast_level: RoastLevel,
        pub status: RoastStatus,
    }

    #[test]
    fn test_simulate_roast_session_light() {
        let result = simulate_roast_session(
            dec("500.0"),
            dec("435.0"), // 13% loss
            420,          // 7 min first crack
            480,          // 8 min drop
        ).unwrap();

        assert_eq!(result.roast_level, RoastLevel::Light);
        assert_eq!(result.development_time, 60);
        assert_eq!(result.status, RoastStatus::Completed);
    }

    #[test]
    fn test_simulate_roast_session_dark() {
        let result = simulate_roast_session(
            dec("500.0"),
            dec("400.0"), // 20% loss
            480,          // 8 min first crack
            720,          // 12 min drop
        ).unwrap();

        assert_eq!(result.roast_level, RoastLevel::Dark);
        assert_eq!(result.development_time, 240);
        assert_eq!(result.status, RoastStatus::Completed);
    }

    #[test]
    fn test_simulate_roast_session_invalid_weights() {
        let result = simulate_roast_session(
            dec("500.0"),
            dec("600.0"), // Invalid: roasted > green
            420,
            480,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_simulate_roast_session_invalid_times() {
        let result = simulate_roast_session(
            dec("500.0"),
            dec("400.0"),
            480,  // First crack at 8 min
            420,  // Invalid: drop before first crack
        );

        assert!(result.is_err());
    }
}
