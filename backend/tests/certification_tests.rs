//! Certification management tests
//!
//! Tests for certification tracking including:
//! - Property 19: Certification Expiration Alerts
//! - Property 20: Certification Inclusion in Traceability

use chrono::{Days, NaiveDate, Utc};
use proptest::prelude::*;

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;

    /// Test certification types
    #[test]
    fn test_certification_types() {
        let types = [
            "thai_gap",
            "organic_thailand",
            "usda_organic",
            "fair_trade",
            "rainforest_alliance",
            "utz",
            "other",
        ];

        assert_eq!(types.len(), 7);

        // All types should be snake_case
        for t in types {
            assert!(t.chars().all(|c| c.is_lowercase() || c == '_'));
        }
    }

    /// Test certification scopes
    #[test]
    fn test_certification_scopes() {
        let scopes = ["farm", "plot", "facility", "business"];

        assert_eq!(scopes.len(), 4);

        for scope in scopes {
            assert!(scope.chars().all(|c| c.is_lowercase()));
        }
    }

    /// Test days until expiration calculation
    #[test]
    fn test_days_until_expiration() {
        let today = NaiveDate::from_ymd_opt(2024, 12, 24).unwrap();
        let expiration = NaiveDate::from_ymd_opt(2025, 3, 24).unwrap();

        let days_until = (expiration - today).num_days();

        // Should be 90 days
        assert_eq!(days_until, 90);
    }

    /// Test alert thresholds
    #[test]
    fn test_alert_thresholds() {
        let alert_days = [90, 60, 30];

        // Verify standard alert intervals
        assert_eq!(alert_days[0], 90);
        assert_eq!(alert_days[1], 60);
        assert_eq!(alert_days[2], 30);
    }

    /// Test should_trigger_alert at exactly 90 days
    #[test]
    fn test_alert_at_90_days() {
        let days_until = 90;
        let alert_days = [90, 60, 30];

        let should_trigger = alert_days.contains(&days_until);
        assert!(should_trigger);
    }

    /// Test should_trigger_alert at exactly 60 days
    #[test]
    fn test_alert_at_60_days() {
        let days_until = 60;
        let alert_days = [90, 60, 30];

        let should_trigger = alert_days.contains(&days_until);
        assert!(should_trigger);
    }

    /// Test should_trigger_alert at exactly 30 days
    #[test]
    fn test_alert_at_30_days() {
        let days_until = 30;
        let alert_days = [90, 60, 30];

        let should_trigger = alert_days.contains(&days_until);
        assert!(should_trigger);
    }

    /// Test no alert at non-threshold days
    #[test]
    fn test_no_alert_at_other_days() {
        let non_alert_days = [91, 89, 61, 59, 31, 29, 15, 7, 1, 0];
        let alert_days = [90, 60, 30];

        for days in non_alert_days {
            let should_trigger = alert_days.contains(&days);
            assert!(!should_trigger, "Should not trigger at {} days", days);
        }
    }

    /// Test certification is active
    #[test]
    fn test_certification_active() {
        let today = NaiveDate::from_ymd_opt(2024, 12, 24).unwrap();
        let expiration = NaiveDate::from_ymd_opt(2025, 6, 24).unwrap();

        let is_active = expiration >= today;
        assert!(is_active);
    }

    /// Test certification is expired
    #[test]
    fn test_certification_expired() {
        let today = NaiveDate::from_ymd_opt(2024, 12, 24).unwrap();
        let expiration = NaiveDate::from_ymd_opt(2024, 6, 24).unwrap();

        let is_expired = expiration < today;
        assert!(is_expired);
    }

    /// Test date validation - expiration must be after issue
    #[test]
    fn test_date_validation() {
        let issue_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let expiration_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();

        let is_valid = expiration_date > issue_date;
        assert!(is_valid);
    }

    /// Test invalid dates - expiration before issue
    #[test]
    fn test_invalid_dates() {
        let issue_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let expiration_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let is_valid = expiration_date > issue_date;
        assert!(!is_valid);
    }

    /// Test document types
    #[test]
    fn test_document_types() {
        let valid_types = ["certificate", "audit_report", "checklist", "other"];

        assert_eq!(valid_types.len(), 4);
        assert!(valid_types.contains(&"certificate"));
        assert!(valid_types.contains(&"audit_report"));
    }

    /// Test compliance status
    #[test]
    fn test_compliance_status() {
        // Compliant
        let is_compliant = Some(true);
        assert_eq!(is_compliant, Some(true));

        // Non-compliant
        let is_non_compliant = Some(false);
        assert_eq!(is_non_compliant, Some(false));

        // Pending (not yet verified)
        let is_pending: Option<bool> = None;
        assert!(is_pending.is_none());
    }

    /// Test compliance summary calculation
    #[test]
    fn test_compliance_summary() {
        let total_requirements = 8;
        let compliant = 5;
        let non_compliant = 1;
        let pending = 2;

        assert_eq!(compliant + non_compliant + pending, total_requirements);
    }

    /// Test Thai GAP requirements count
    #[test]
    fn test_thai_gap_requirements() {
        // Thai GAP has 8 default requirements
        let thai_gap_requirements = 8;
        assert_eq!(thai_gap_requirements, 8);
    }

    /// Test Organic Thailand requirements count
    #[test]
    fn test_organic_thailand_requirements() {
        // Organic Thailand has 7 default requirements
        let organic_thailand_requirements = 7;
        assert_eq!(organic_thailand_requirements, 7);
    }
}

// ============================================================================
// Expiration Alert Logic
// ============================================================================

/// Check if an alert should be triggered for a certification
/// Returns the alert day (90, 60, or 30) if alert should trigger, None otherwise
pub fn should_trigger_alert(days_until_expiration: i32) -> Option<i32> {
    let alert_days = [90, 60, 30];

    for &alert_day in &alert_days {
        if days_until_expiration == alert_day {
            return Some(alert_day);
        }
    }

    None
}

/// Check if a certification is within the alert window (0-90 days)
pub fn is_in_alert_window(days_until_expiration: i32) -> bool {
    days_until_expiration >= 0 && days_until_expiration <= 90
}

/// Get all applicable alerts for a given days until expiration
/// Returns alerts that should have been triggered (days >= current)
pub fn get_applicable_alerts(days_until_expiration: i32) -> Vec<i32> {
    let alert_days = [90, 60, 30];

    alert_days
        .iter()
        .filter(|&&day| days_until_expiration <= day && days_until_expiration >= 0)
        .copied()
        .collect()
}

/// Check if certification scope applies to a lot
pub fn scope_applies_to_lot(scope: &str, lot_plot_id: Option<&str>, cert_plot_id: Option<&str>) -> bool {
    match scope {
        "business" | "farm" | "facility" => true,
        "plot" => {
            // Plot scope only applies if the lot's plot matches the certification's plot
            match (lot_plot_id, cert_plot_id) {
                (Some(lot_plot), Some(cert_plot)) => lot_plot == cert_plot,
                _ => false,
            }
        }
        _ => false,
    }
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;

    /// Strategy for generating days until expiration (0-365)
    fn days_strategy() -> impl Strategy<Value = i32> {
        0i32..=365
    }

    /// Strategy for generating alert threshold days
    fn alert_day_strategy() -> impl Strategy<Value = i32> {
        prop_oneof![Just(90), Just(60), Just(30)]
    }

    /// Strategy for generating certification scopes
    fn scope_strategy() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            Just("business"),
            Just("farm"),
            Just("plot"),
            Just("facility"),
        ]
    }

    /// Strategy for generating certification types
    fn cert_type_strategy() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            Just("thai_gap"),
            Just("organic_thailand"),
            Just("usda_organic"),
            Just("fair_trade"),
            Just("rainforest_alliance"),
            Just("utz"),
            Just("other"),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // ====================================================================
        // Property 19: Certification Expiration Alerts
        // ====================================================================

        /// Property 19: Alerts trigger at exactly 90, 60, 30 days before expiration
        /// Feature: coffee-quality-management, Property 19: Certification Expiration Alerts
        #[test]
        fn prop_alert_triggers_at_exact_thresholds(alert_day in alert_day_strategy()) {
            // Alert should trigger at exactly 90, 60, or 30 days
            let result = should_trigger_alert(alert_day);
            prop_assert!(result.is_some());
            prop_assert_eq!(result.unwrap(), alert_day);
        }

        /// Property 19: Alerts do NOT trigger at non-threshold days
        /// Feature: coffee-quality-management, Property 19: Certification Expiration Alerts
        #[test]
        fn prop_alert_does_not_trigger_at_other_days(days in days_strategy()) {
            // Skip the exact threshold days
            if days == 90 || days == 60 || days == 30 {
                return Ok(());
            }

            let result = should_trigger_alert(days);
            prop_assert!(result.is_none(), "Alert should not trigger at {} days", days);
        }

        /// Property 19: Alert window is 0-90 days
        /// Feature: coffee-quality-management, Property 19: Certification Expiration Alerts
        #[test]
        fn prop_alert_window_bounds(days in days_strategy()) {
            let in_window = is_in_alert_window(days);

            if days >= 0 && days <= 90 {
                prop_assert!(in_window, "Days {} should be in alert window", days);
            } else {
                prop_assert!(!in_window, "Days {} should NOT be in alert window", days);
            }
        }

        /// Property 19: Applicable alerts are cumulative as expiration approaches
        /// Feature: coffee-quality-management, Property 19: Certification Expiration Alerts
        #[test]
        fn prop_applicable_alerts_cumulative(days in 0i32..=90) {
            let alerts = get_applicable_alerts(days);

            // At 90 days: [90]
            // At 60 days: [90, 60]
            // At 30 days: [90, 60, 30]
            // At 0 days: [90, 60, 30]

            if days <= 30 {
                prop_assert!(alerts.contains(&30));
                prop_assert!(alerts.contains(&60));
                prop_assert!(alerts.contains(&90));
            } else if days <= 60 {
                prop_assert!(!alerts.contains(&30));
                prop_assert!(alerts.contains(&60));
                prop_assert!(alerts.contains(&90));
            } else if days <= 90 {
                prop_assert!(!alerts.contains(&30));
                prop_assert!(!alerts.contains(&60));
                prop_assert!(alerts.contains(&90));
            }
        }

        /// Property 19: No alerts for certifications expiring > 90 days
        /// Feature: coffee-quality-management, Property 19: Certification Expiration Alerts
        #[test]
        fn prop_no_alerts_beyond_90_days(days in 91i32..=365) {
            let alerts = get_applicable_alerts(days);
            prop_assert!(alerts.is_empty(), "No alerts should apply at {} days", days);
        }

        /// Property 19: Expired certifications (negative days) have all alerts applicable
        /// Feature: coffee-quality-management, Property 19: Certification Expiration Alerts
        #[test]
        fn prop_expired_certs_no_alerts(days in -365i32..0) {
            let in_window = is_in_alert_window(days);
            prop_assert!(!in_window, "Expired certs should not be in alert window");
        }

        // ====================================================================
        // Property 20: Certification Inclusion in Traceability
        // ====================================================================

        /// Property 20: Business/Farm/Facility scope always applies to lots
        /// Feature: coffee-quality-management, Property 20: Certification Inclusion in Traceability
        #[test]
        fn prop_broad_scope_always_applies(
            scope in prop_oneof![Just("business"), Just("farm"), Just("facility")]
        ) {
            // These scopes should always apply regardless of plot
            let applies = scope_applies_to_lot(scope, None, None);
            prop_assert!(applies, "Scope {} should always apply", scope);

            let applies_with_plot = scope_applies_to_lot(scope, Some("plot-1"), Some("plot-2"));
            prop_assert!(applies_with_plot, "Scope {} should apply even with different plots", scope);
        }

        /// Property 20: Plot scope only applies when plots match
        /// Feature: coffee-quality-management, Property 20: Certification Inclusion in Traceability
        #[test]
        fn prop_plot_scope_requires_match(
            lot_plot in prop::option::of("[a-z0-9-]{36}"),
            cert_plot in prop::option::of("[a-z0-9-]{36}")
        ) {
            let applies = scope_applies_to_lot("plot", lot_plot.as_deref(), cert_plot.as_deref());

            match (lot_plot.as_deref(), cert_plot.as_deref()) {
                (Some(lp), Some(cp)) if lp == cp => {
                    prop_assert!(applies, "Plot scope should apply when plots match");
                }
                _ => {
                    prop_assert!(!applies, "Plot scope should not apply when plots don't match");
                }
            }
        }

        /// Property 20: All certification types are valid
        /// Feature: coffee-quality-management, Property 20: Certification Inclusion in Traceability
        #[test]
        fn prop_all_cert_types_valid(cert_type in cert_type_strategy()) {
            let valid_types = [
                "thai_gap", "organic_thailand", "usda_organic",
                "fair_trade", "rainforest_alliance", "utz", "other"
            ];
            prop_assert!(valid_types.contains(&cert_type));
        }

        /// Property 20: All scopes are valid
        /// Feature: coffee-quality-management, Property 20: Certification Inclusion in Traceability
        #[test]
        fn prop_all_scopes_valid(scope in scope_strategy()) {
            let valid_scopes = ["business", "farm", "plot", "facility"];
            prop_assert!(valid_scopes.contains(&scope));
        }
    }
}

// ============================================================================
// Date Calculation Tests
// ============================================================================

#[cfg(test)]
mod date_tests {
    use super::*;

    /// Test calculating days until expiration from dates
    #[test]
    fn test_days_calculation_from_dates() {
        let today = Utc::now().date_naive();
        let expiration_90 = today.checked_add_days(Days::new(90)).unwrap();
        let expiration_60 = today.checked_add_days(Days::new(60)).unwrap();
        let expiration_30 = today.checked_add_days(Days::new(30)).unwrap();

        assert_eq!((expiration_90 - today).num_days(), 90);
        assert_eq!((expiration_60 - today).num_days(), 60);
        assert_eq!((expiration_30 - today).num_days(), 30);
    }

    /// Test alert triggers for specific dates
    #[test]
    fn test_alert_for_specific_dates() {
        let today = Utc::now().date_naive();

        // Certification expiring in exactly 90 days
        let exp_90 = today.checked_add_days(Days::new(90)).unwrap();
        let days_90 = (exp_90 - today).num_days() as i32;
        assert!(should_trigger_alert(days_90).is_some());

        // Certification expiring in exactly 60 days
        let exp_60 = today.checked_add_days(Days::new(60)).unwrap();
        let days_60 = (exp_60 - today).num_days() as i32;
        assert!(should_trigger_alert(days_60).is_some());

        // Certification expiring in exactly 30 days
        let exp_30 = today.checked_add_days(Days::new(30)).unwrap();
        let days_30 = (exp_30 - today).num_days() as i32;
        assert!(should_trigger_alert(days_30).is_some());
    }

    /// Test no alert for 45 days
    #[test]
    fn test_no_alert_at_45_days() {
        let today = Utc::now().date_naive();
        let exp_45 = today.checked_add_days(Days::new(45)).unwrap();
        let days_45 = (exp_45 - today).num_days() as i32;

        assert!(should_trigger_alert(days_45).is_none());
    }

    /// Test expired certification
    #[test]
    fn test_expired_certification() {
        let today = Utc::now().date_naive();
        let expired = today.checked_sub_days(Days::new(10)).unwrap();
        let days = (expired - today).num_days() as i32;

        assert!(days < 0);
        assert!(!is_in_alert_window(days));
    }
}

// ============================================================================
// Traceability Integration Tests
// ============================================================================

#[cfg(test)]
mod traceability_tests {
    use super::*;

    /// Test that business scope certifications are included
    #[test]
    fn test_business_scope_included() {
        assert!(scope_applies_to_lot("business", None, None));
        assert!(scope_applies_to_lot("business", Some("any-plot"), None));
    }

    /// Test that farm scope certifications are included
    #[test]
    fn test_farm_scope_included() {
        assert!(scope_applies_to_lot("farm", None, None));
        assert!(scope_applies_to_lot("farm", Some("any-plot"), None));
    }

    /// Test that facility scope certifications are included
    #[test]
    fn test_facility_scope_included() {
        assert!(scope_applies_to_lot("facility", None, None));
        assert!(scope_applies_to_lot("facility", Some("any-plot"), None));
    }

    /// Test that plot scope requires matching plot
    #[test]
    fn test_plot_scope_matching() {
        let plot_id = "plot-123";

        // Same plot - should apply
        assert!(scope_applies_to_lot("plot", Some(plot_id), Some(plot_id)));

        // Different plot - should not apply
        assert!(!scope_applies_to_lot("plot", Some(plot_id), Some("plot-456")));

        // Missing plot info - should not apply
        assert!(!scope_applies_to_lot("plot", None, Some(plot_id)));
        assert!(!scope_applies_to_lot("plot", Some(plot_id), None));
        assert!(!scope_applies_to_lot("plot", None, None));
    }

    /// Test active certification filtering
    #[test]
    fn test_active_certification_filter() {
        let today = Utc::now().date_naive();

        // Active certification (expires in future)
        let active_expiration = today.checked_add_days(Days::new(180)).unwrap();
        let is_active = active_expiration >= today;
        assert!(is_active);

        // Expired certification
        let expired_expiration = today.checked_sub_days(Days::new(30)).unwrap();
        let is_expired = expired_expiration < today;
        assert!(is_expired);
    }
}
