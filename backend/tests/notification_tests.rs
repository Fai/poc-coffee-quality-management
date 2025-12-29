//! Notification service tests
//!
//! Tests for notification management including:
//! - Property 24: Notification Preference Respect

use proptest::prelude::*;

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;

    /// Test notification types
    #[test]
    fn test_notification_types() {
        let types = [
            "low_inventory",
            "certification_expiring",
            "processing_milestone",
            "weather_alert",
            "harvest_reminder",
            "quality_alert",
            "system",
        ];

        assert_eq!(types.len(), 7);

        // All types should be snake_case
        for t in types {
            assert!(t.chars().all(|c| c.is_lowercase() || c == '_'));
        }
    }

    /// Test notification channels
    #[test]
    fn test_notification_channels() {
        let channels = ["line", "in_app", "email"];

        assert_eq!(channels.len(), 3);
    }

    /// Test notification statuses
    #[test]
    fn test_notification_statuses() {
        let statuses = ["pending", "sent", "failed", "read"];

        assert_eq!(statuses.len(), 4);
    }

    /// Test default preferences
    #[test]
    fn test_default_preferences() {
        let prefs = NotificationPreferences::default();

        // All notification types should be enabled by default
        assert!(prefs.line_enabled);
        assert!(prefs.low_inventory_enabled);
        assert!(prefs.certification_expiring_enabled);
        assert!(prefs.processing_milestone_enabled);
        assert!(prefs.weather_alert_enabled);
        assert!(prefs.harvest_reminder_enabled);
        assert!(prefs.quality_alert_enabled);
    }

    /// Test channel selection - LINE connected and enabled
    #[test]
    fn test_channel_selection_line_enabled() {
        let line_connected = true;
        let line_enabled = true;

        let channel = determine_channel(line_connected, line_enabled);
        assert_eq!(channel, "line");
    }

    /// Test channel selection - LINE connected but disabled
    #[test]
    fn test_channel_selection_line_disabled() {
        let line_connected = true;
        let line_enabled = false;

        let channel = determine_channel(line_connected, line_enabled);
        assert_eq!(channel, "in_app");
    }

    /// Test channel selection - LINE not connected
    #[test]
    fn test_channel_selection_line_not_connected() {
        let line_connected = false;
        let line_enabled = true;

        let channel = determine_channel(line_connected, line_enabled);
        assert_eq!(channel, "in_app");
    }

    /// Test notification type enabled check
    #[test]
    fn test_notification_type_enabled() {
        let prefs = NotificationPreferences::default();

        assert!(is_type_enabled(&prefs, "low_inventory"));
        assert!(is_type_enabled(&prefs, "certification_expiring"));
        assert!(is_type_enabled(&prefs, "system")); // System always enabled
    }

    /// Test notification type disabled
    #[test]
    fn test_notification_type_disabled() {
        let mut prefs = NotificationPreferences::default();
        prefs.low_inventory_enabled = false;

        assert!(!is_type_enabled(&prefs, "low_inventory"));
        assert!(is_type_enabled(&prefs, "certification_expiring"));
    }

    /// Test priority levels
    #[test]
    fn test_priority_levels() {
        // Higher number = more urgent
        let low_priority = 0;
        let medium_priority = 1;
        let high_priority = 2;

        assert!(high_priority > medium_priority);
        assert!(medium_priority > low_priority);
    }

    /// Test notification message formatting
    #[test]
    fn test_notification_message_format() {
        let title = "Low Inventory Alert";
        let message = "Lot 'Test Lot' has fallen below threshold";

        let formatted = format!("{}\n\n{}", title, message);
        assert!(formatted.contains(title));
        assert!(formatted.contains(message));
    }
}

// ============================================================================
// Helper Types and Functions
// ============================================================================

/// Notification preferences (simplified for testing)
#[derive(Debug, Clone)]
pub struct NotificationPreferences {
    pub line_enabled: bool,
    pub email_enabled: bool,
    pub low_inventory_enabled: bool,
    pub certification_expiring_enabled: bool,
    pub processing_milestone_enabled: bool,
    pub weather_alert_enabled: bool,
    pub harvest_reminder_enabled: bool,
    pub quality_alert_enabled: bool,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            line_enabled: true,
            email_enabled: true,
            low_inventory_enabled: true,
            certification_expiring_enabled: true,
            processing_milestone_enabled: true,
            weather_alert_enabled: true,
            harvest_reminder_enabled: true,
            quality_alert_enabled: true,
        }
    }
}

/// Determine notification channel based on LINE connection and preferences
pub fn determine_channel(line_connected: bool, line_enabled: bool) -> &'static str {
    if line_connected && line_enabled {
        "line"
    } else {
        "in_app"
    }
}

/// Check if a notification type is enabled
pub fn is_type_enabled(prefs: &NotificationPreferences, notification_type: &str) -> bool {
    match notification_type {
        "low_inventory" => prefs.low_inventory_enabled,
        "certification_expiring" => prefs.certification_expiring_enabled,
        "processing_milestone" => prefs.processing_milestone_enabled,
        "weather_alert" => prefs.weather_alert_enabled,
        "harvest_reminder" => prefs.harvest_reminder_enabled,
        "quality_alert" => prefs.quality_alert_enabled,
        "system" => true, // System notifications always enabled
        _ => false,
    }
}

/// Should send notification based on preferences and connection
pub fn should_send_notification(
    prefs: &NotificationPreferences,
    notification_type: &str,
    line_connected: bool,
) -> (bool, &'static str) {
    // Check if notification type is enabled
    if !is_type_enabled(prefs, notification_type) {
        return (false, "disabled");
    }

    // Determine channel
    let channel = determine_channel(line_connected, prefs.line_enabled);
    (true, channel)
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;

    /// Strategy for generating notification types
    fn notification_type_strategy() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            Just("low_inventory"),
            Just("certification_expiring"),
            Just("processing_milestone"),
            Just("weather_alert"),
            Just("harvest_reminder"),
            Just("quality_alert"),
            Just("system"),
        ]
    }

    /// Strategy for generating notification preferences
    fn preferences_strategy() -> impl Strategy<Value = NotificationPreferences> {
        (
            any::<bool>(), // line_enabled
            any::<bool>(), // email_enabled
            any::<bool>(), // low_inventory_enabled
            any::<bool>(), // certification_expiring_enabled
            any::<bool>(), // processing_milestone_enabled
            any::<bool>(), // weather_alert_enabled
            any::<bool>(), // harvest_reminder_enabled
            any::<bool>(), // quality_alert_enabled
        )
            .prop_map(
                |(
                    line_enabled,
                    email_enabled,
                    low_inventory_enabled,
                    certification_expiring_enabled,
                    processing_milestone_enabled,
                    weather_alert_enabled,
                    harvest_reminder_enabled,
                    quality_alert_enabled,
                )| {
                    NotificationPreferences {
                        line_enabled,
                        email_enabled,
                        low_inventory_enabled,
                        certification_expiring_enabled,
                        processing_milestone_enabled,
                        weather_alert_enabled,
                        harvest_reminder_enabled,
                        quality_alert_enabled,
                    }
                },
            )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // ====================================================================
        // Property 24: Notification Preference Respect
        // ====================================================================

        /// Property 24: LINE is used only when connected AND enabled
        /// Feature: coffee-quality-management, Property 24: Notification Preference Respect
        #[test]
        fn prop_line_used_only_when_connected_and_enabled(
            line_connected in any::<bool>(),
            line_enabled in any::<bool>()
        ) {
            let channel = determine_channel(line_connected, line_enabled);

            if line_connected && line_enabled {
                prop_assert_eq!(channel, "line");
            } else {
                prop_assert_eq!(channel, "in_app");
            }
        }

        /// Property 24: Disabled notification types are not sent
        /// Feature: coffee-quality-management, Property 24: Notification Preference Respect
        #[test]
        fn prop_disabled_types_not_sent(
            prefs in preferences_strategy(),
            notification_type in notification_type_strategy(),
            line_connected in any::<bool>()
        ) {
            let (should_send, _channel) = should_send_notification(&prefs, notification_type, line_connected);

            let type_enabled = is_type_enabled(&prefs, notification_type);

            if !type_enabled {
                prop_assert!(!should_send, "Disabled notification type should not be sent");
            }
        }

        /// Property 24: System notifications are always enabled
        /// Feature: coffee-quality-management, Property 24: Notification Preference Respect
        #[test]
        fn prop_system_notifications_always_enabled(
            prefs in preferences_strategy()
        ) {
            let enabled = is_type_enabled(&prefs, "system");
            prop_assert!(enabled, "System notifications should always be enabled");
        }

        /// Property 24: Channel selection is deterministic
        /// Feature: coffee-quality-management, Property 24: Notification Preference Respect
        #[test]
        fn prop_channel_selection_deterministic(
            line_connected in any::<bool>(),
            line_enabled in any::<bool>()
        ) {
            let channel1 = determine_channel(line_connected, line_enabled);
            let channel2 = determine_channel(line_connected, line_enabled);

            prop_assert_eq!(channel1, channel2, "Channel selection should be deterministic");
        }

        /// Property 24: Fallback to in-app when LINE not available
        /// Feature: coffee-quality-management, Property 24: Notification Preference Respect
        #[test]
        fn prop_fallback_to_in_app(
            line_enabled in any::<bool>()
        ) {
            // When LINE is not connected, always fall back to in-app
            let channel = determine_channel(false, line_enabled);
            prop_assert_eq!(channel, "in_app");
        }

        /// Property 24: Enabled types with LINE connected use LINE
        /// Feature: coffee-quality-management, Property 24: Notification Preference Respect
        #[test]
        fn prop_enabled_types_use_line_when_available(
            notification_type in notification_type_strategy()
        ) {
            let mut prefs = NotificationPreferences::default();
            prefs.line_enabled = true;

            let (should_send, channel) = should_send_notification(&prefs, notification_type, true);

            // All default types are enabled
            prop_assert!(should_send);
            prop_assert_eq!(channel, "line");
        }

        /// Property 24: Notification type preference independence
        /// Feature: coffee-quality-management, Property 24: Notification Preference Respect
        #[test]
        fn prop_type_preferences_independent(
            prefs in preferences_strategy()
        ) {
            // Changing one type's preference shouldn't affect others
            let low_inv = is_type_enabled(&prefs, "low_inventory");
            let cert_exp = is_type_enabled(&prefs, "certification_expiring");

            // These should match the individual preference settings
            prop_assert_eq!(low_inv, prefs.low_inventory_enabled);
            prop_assert_eq!(cert_exp, prefs.certification_expiring_enabled);
        }
    }
}

// ============================================================================
// Integration Test Helpers
// ============================================================================

#[cfg(test)]
mod integration_helpers {
    use super::*;

    /// Simulate sending a notification
    pub fn simulate_send_notification(
        prefs: &NotificationPreferences,
        notification_type: &str,
        line_connected: bool,
    ) -> Result<(&'static str, &'static str), &'static str> {
        let (should_send, channel) = should_send_notification(prefs, notification_type, line_connected);

        if !should_send {
            return Err("Notification type disabled");
        }

        Ok((channel, "sent"))
    }

    #[test]
    fn test_simulate_send_line() {
        let prefs = NotificationPreferences::default();
        let result = simulate_send_notification(&prefs, "low_inventory", true);

        assert!(result.is_ok());
        let (channel, status) = result.unwrap();
        assert_eq!(channel, "line");
        assert_eq!(status, "sent");
    }

    #[test]
    fn test_simulate_send_in_app() {
        let prefs = NotificationPreferences::default();
        let result = simulate_send_notification(&prefs, "low_inventory", false);

        assert!(result.is_ok());
        let (channel, status) = result.unwrap();
        assert_eq!(channel, "in_app");
        assert_eq!(status, "sent");
    }

    #[test]
    fn test_simulate_send_disabled() {
        let mut prefs = NotificationPreferences::default();
        prefs.low_inventory_enabled = false;

        let result = simulate_send_notification(&prefs, "low_inventory", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_simulate_send_system_always_works() {
        let mut prefs = NotificationPreferences::default();
        // Disable everything except system (which can't be disabled)
        prefs.low_inventory_enabled = false;
        prefs.certification_expiring_enabled = false;
        prefs.processing_milestone_enabled = false;
        prefs.weather_alert_enabled = false;
        prefs.harvest_reminder_enabled = false;
        prefs.quality_alert_enabled = false;

        let result = simulate_send_notification(&prefs, "system", true);
        assert!(result.is_ok());
    }
}
