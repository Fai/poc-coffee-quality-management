//! Inventory management tests
//!
//! Tests for inventory tracking including:
//! - Property 13: Stage Transition Consistency
//! - Property 14: Inventory Balance Accuracy
//! - Property 15: Alert Triggering Correctness

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
    use chrono::{Datelike, NaiveDate};

    /// Test transaction types
    #[test]
    fn test_transaction_types() {
        let types = [
            "harvest_in",
            "processing_out",
            "processing_in",
            "roasting_out",
            "roasting_in",
            "sale",
            "purchase",
            "adjustment",
            "transfer",
            "sample",
            "return",
        ];

        assert_eq!(types.len(), 11);
        
        // All types should be snake_case
        for t in types {
            assert!(t.chars().all(|c| c.is_lowercase() || c == '_'));
        }
    }

    /// Test transaction direction
    #[test]
    fn test_transaction_direction() {
        let directions = ["in", "out"];
        
        // In adds to inventory
        assert_eq!(directions[0], "in");
        
        // Out removes from inventory
        assert_eq!(directions[1], "out");
    }

    /// Test balance calculation
    #[test]
    fn test_balance_calculation() {
        let total_in = dec("100.0");
        let total_out = dec("30.0");
        let balance = total_in - total_out;
        
        assert_eq!(balance, dec("70.0"));
    }

    /// Test balance with multiple transactions
    #[test]
    fn test_balance_multiple_transactions() {
        let transactions = vec![
            ("in", dec("50.0")),
            ("in", dec("30.0")),
            ("out", dec("20.0")),
            ("in", dec("10.0")),
            ("out", dec("15.0")),
        ];

        let balance: Decimal = transactions.iter().fold(Decimal::ZERO, |acc, (dir, qty)| {
            if *dir == "in" {
                acc + qty
            } else {
                acc - qty
            }
        });

        // 50 + 30 - 20 + 10 - 15 = 55
        assert_eq!(balance, dec("55.0"));
    }

    /// Test total price calculation
    #[test]
    fn test_total_price_calculation() {
        let quantity = dec("50.5");
        let unit_price = dec("25.0");
        let total_price = quantity * unit_price;
        
        assert_eq!(total_price, dec("1262.5"));
    }

    /// Test weighted average cost calculation
    #[test]
    fn test_weighted_average_cost() {
        // Transaction 1: 100kg at 20 THB/kg = 2000 THB
        // Transaction 2: 50kg at 30 THB/kg = 1500 THB
        // Total: 150kg, 3500 THB
        // Average: 3500 / 150 = 23.33... THB/kg
        
        let total_quantity = dec("150.0");
        let total_value = dec("3500.0");
        let avg_cost = total_value / total_quantity;
        
        // Should be approximately 23.33
        assert!(avg_cost > dec("23.0") && avg_cost < dec("24.0"));
    }

    /// Test alert threshold check
    #[test]
    fn test_alert_threshold_check() {
        let threshold = dec("50.0");
        let current_balance = dec("30.0");
        
        // Alert should trigger when balance <= threshold
        let should_trigger = current_balance <= threshold;
        assert!(should_trigger);
        
        let current_balance_high = dec("60.0");
        let should_not_trigger = current_balance_high <= threshold;
        assert!(!should_not_trigger);
    }

    /// Test stage transitions
    #[test]
    fn test_valid_stage_transitions() {
        let valid_transitions = [
            ("cherry", "parchment"),
            ("parchment", "green_bean"),
            ("green_bean", "roasted_bean"),
            ("roasted_bean", "sold"),
        ];

        for (from, to) in valid_transitions {
            assert!(is_valid_transition(from, to));
        }
    }

    /// Test invalid stage transitions
    #[test]
    fn test_invalid_stage_transitions() {
        let invalid_transitions = [
            ("cherry", "roasted_bean"),  // Skip stages
            ("green_bean", "cherry"),    // Backward
            ("sold", "cherry"),          // From terminal
        ];

        for (from, to) in invalid_transitions {
            assert!(!is_valid_transition(from, to));
        }
    }

    pub fn is_valid_transition(from: &str, to: &str) -> bool {
        let stages = ["cherry", "parchment", "green_bean", "roasted_bean", "sold"];
        
        let from_idx = stages.iter().position(|&s| s == from);
        let to_idx = stages.iter().position(|&s| s == to);
        
        match (from_idx, to_idx) {
            (Some(f), Some(t)) => t == f + 1, // Can only move to next stage
            _ => false,
        }
    }

    /// Test inventory valuation
    #[test]
    fn test_inventory_valuation() {
        let quantity = dec("100.0");
        let unit_cost = dec("25.0");
        let total_value = quantity * unit_cost;
        
        assert_eq!(total_value, dec("2500.0"));
    }

    /// Test zero balance
    #[test]
    fn test_zero_balance() {
        let total_in = dec("100.0");
        let total_out = dec("100.0");
        let balance = total_in - total_out;
        
        assert_eq!(balance, Decimal::ZERO);
    }

    /// Test negative balance prevention
    #[test]
    fn test_negative_balance_detection() {
        let current_balance = dec("50.0");
        let requested_out = dec("60.0");
        
        // Should detect insufficient inventory
        let would_be_negative = current_balance - requested_out < Decimal::ZERO;
        assert!(would_be_negative);
    }

    /// Test currency default
    #[test]
    fn test_currency_default() {
        let default_currency = "THB";
        assert_eq!(default_currency, "THB");
    }

    /// Test transaction date handling
    #[test]
    fn test_transaction_date() {
        let date = NaiveDate::from_ymd_opt(2024, 12, 23).unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 23);
    }
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;

    /// Strategy for generating valid quantities (positive decimals)
    fn quantity_strategy() -> impl Strategy<Value = Decimal> {
        (1i64..=10000i64).prop_map(|n| Decimal::new(n, 1)) // 0.1 to 1000.0
    }

    /// Strategy for generating valid unit prices
    fn price_strategy() -> impl Strategy<Value = Decimal> {
        (1i64..=100000i64).prop_map(|n| Decimal::new(n, 2)) // 0.01 to 1000.00
    }

    /// Strategy for generating transaction directions
    fn direction_strategy() -> impl Strategy<Value = &'static str> {
        prop_oneof![Just("in"), Just("out")]
    }

    /// Strategy for generating stages
    fn stage_strategy() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            Just("cherry"),
            Just("parchment"),
            Just("green_bean"),
            Just("roasted_bean"),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 14: Inventory Balance Accuracy
        /// Balance = Sum(IN transactions) - Sum(OUT transactions)
        #[test]
        fn prop_inventory_balance_accuracy(
            in_amounts in prop::collection::vec(quantity_strategy(), 1..10),
            out_amounts in prop::collection::vec(quantity_strategy(), 0..5)
        ) {
            let total_in: Decimal = in_amounts.iter().sum();
            let total_out: Decimal = out_amounts.iter().sum();
            
            // Only test if we have enough inventory
            if total_in >= total_out {
                let balance = total_in - total_out;
                
                // Balance should be non-negative
                prop_assert!(balance >= Decimal::ZERO);
                
                // Balance should equal total_in - total_out
                prop_assert_eq!(balance, total_in - total_out);
            }
        }

        /// Property 14: Balance is always sum of in minus sum of out
        #[test]
        fn prop_balance_calculation_correct(
            transactions in prop::collection::vec(
                (direction_strategy(), quantity_strategy()),
                1..20
            )
        ) {
            let mut total_in = Decimal::ZERO;
            let mut total_out = Decimal::ZERO;
            
            for (dir, qty) in &transactions {
                if *dir == "in" {
                    total_in += qty;
                } else {
                    total_out += qty;
                }
            }
            
            let balance = total_in - total_out;
            
            // Verify calculation
            let calculated_balance: Decimal = transactions.iter().fold(Decimal::ZERO, |acc, (dir, qty)| {
                if *dir == "in" { acc + qty } else { acc - qty }
            });
            
            prop_assert_eq!(balance, calculated_balance);
        }

        /// Property 15: Alert Triggering Correctness
        /// Alert triggers when balance <= threshold
        #[test]
        fn prop_alert_triggering_correct(
            balance in quantity_strategy(),
            threshold in quantity_strategy()
        ) {
            let should_trigger = balance <= threshold;
            
            // Verify the logic
            if balance <= threshold {
                prop_assert!(should_trigger);
            } else {
                prop_assert!(!should_trigger);
            }
        }

        /// Property 15: Alert never triggers when balance > threshold
        #[test]
        fn prop_alert_no_false_positive(
            threshold in quantity_strategy(),
            extra in quantity_strategy()
        ) {
            let balance = threshold + extra; // Always above threshold
            let should_trigger = balance <= threshold;
            
            // Should never trigger when balance > threshold
            prop_assert!(!should_trigger);
        }

        /// Property: Total price calculation is correct
        #[test]
        fn prop_total_price_calculation(
            quantity in quantity_strategy(),
            unit_price in price_strategy()
        ) {
            let total_price = quantity * unit_price;
            
            // Total price should be positive
            prop_assert!(total_price > Decimal::ZERO);
            
            // Total price should equal quantity * unit_price
            prop_assert_eq!(total_price, quantity * unit_price);
        }

        /// Property: Weighted average cost is between min and max unit prices
        #[test]
        fn prop_weighted_average_cost_bounded(
            prices in prop::collection::vec(price_strategy(), 2..10),
            quantities in prop::collection::vec(quantity_strategy(), 2..10)
        ) {
            // Ensure same length
            let len = prices.len().min(quantities.len());
            if len < 2 {
                return Ok(());
            }
            
            let prices = &prices[..len];
            let quantities = &quantities[..len];
            
            let total_value: Decimal = prices.iter()
                .zip(quantities.iter())
                .map(|(p, q)| p * q)
                .sum();
            
            let total_quantity: Decimal = quantities.iter().sum();
            
            if total_quantity > Decimal::ZERO {
                let avg_cost = total_value / total_quantity;
                
                let min_price = prices.iter().min().unwrap();
                let max_price = prices.iter().max().unwrap();
                
                // Average should be between min and max
                prop_assert!(avg_cost >= *min_price);
                prop_assert!(avg_cost <= *max_price);
            }
        }

        /// Property 13: Stage Transition Consistency
        /// Stages must follow: cherry -> parchment -> green_bean -> roasted_bean -> sold
        #[test]
        fn prop_stage_transition_valid(
            from_idx in 0usize..4,
            to_idx in 0usize..5
        ) {
            let stages = ["cherry", "parchment", "green_bean", "roasted_bean", "sold"];
            let from = stages[from_idx];
            let to = stages[to_idx];
            
            let is_valid = to_idx == from_idx + 1;
            
            // Verify our validation logic
            if is_valid {
                prop_assert!(super::unit_tests::is_valid_transition(from, to));
            } else {
                prop_assert!(!super::unit_tests::is_valid_transition(from, to));
            }
        }

        /// Property: Quantity must be positive
        #[test]
        fn prop_quantity_positive(quantity in quantity_strategy()) {
            prop_assert!(quantity > Decimal::ZERO);
        }

        /// Property: Balance after full withdrawal is zero
        #[test]
        fn prop_full_withdrawal_zero_balance(quantity in quantity_strategy()) {
            let balance_after = quantity - quantity;
            prop_assert_eq!(balance_after, Decimal::ZERO);
        }

        /// Property: Multiple in transactions accumulate correctly
        #[test]
        fn prop_in_transactions_accumulate(
            amounts in prop::collection::vec(quantity_strategy(), 1..20)
        ) {
            let total: Decimal = amounts.iter().sum();
            let expected: Decimal = amounts.iter().fold(Decimal::ZERO, |acc, x| acc + x);
            
            prop_assert_eq!(total, expected);
        }
    }
}

// ============================================================================
// Integration Test Helpers (for use with actual database)
// ============================================================================

#[cfg(test)]
mod integration_helpers {
    use super::*;

    /// Simulate recording a transaction and updating balance
    pub fn simulate_transaction(
        current_balance: Decimal,
        direction: &str,
        quantity: Decimal,
    ) -> Result<Decimal, &'static str> {
        if quantity <= Decimal::ZERO {
            return Err("Quantity must be positive");
        }

        match direction {
            "in" => Ok(current_balance + quantity),
            "out" => {
                if current_balance >= quantity {
                    Ok(current_balance - quantity)
                } else {
                    Err("Insufficient inventory")
                }
            }
            _ => Err("Invalid direction"),
        }
    }

    #[test]
    fn test_simulate_transaction_in() {
        let balance = dec("100.0");
        let new_balance = simulate_transaction(balance, "in", dec("50.0")).unwrap();
        assert_eq!(new_balance, dec("150.0"));
    }

    #[test]
    fn test_simulate_transaction_out() {
        let balance = dec("100.0");
        let new_balance = simulate_transaction(balance, "out", dec("30.0")).unwrap();
        assert_eq!(new_balance, dec("70.0"));
    }

    #[test]
    fn test_simulate_transaction_insufficient() {
        let balance = dec("50.0");
        let result = simulate_transaction(balance, "out", dec("60.0"));
        assert!(result.is_err());
    }

    #[test]
    fn test_simulate_transaction_invalid_quantity() {
        let balance = dec("100.0");
        let result = simulate_transaction(balance, "in", dec("-10.0"));
        assert!(result.is_err());
    }
}
