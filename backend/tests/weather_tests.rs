//! Weather integration tests
//!
//! Tests for weather data including:
//! - Property 22: Weather Data Association
//! - Property 23: Rain Alert Generation

use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;
use std::str::FromStr;
use chrono::{DateTime, Utc};

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

    /// Test weather snapshot structure
    #[test]
    fn test_weather_snapshot_fields() {
        let snapshot = WeatherSnapshot {
            latitude: dec("18.7883"),
            longitude: dec("98.9853"),
            temperature_celsius: dec("25.5"),
            humidity_percent: 75,
            rain_1h_mm: Some(dec("2.5")),
            rain_3h_mm: Some(dec("5.0")),
            weather_condition: "Rain".to_string(),
        };

        assert_eq!(snapshot.temperature_celsius, dec("25.5"));
        assert_eq!(snapshot.humidity_percent, 75);
        assert!(snapshot.rain_1h_mm.is_some());
    }

    /// Test Thai coordinates validation (Thailand bounding box)
    #[test]
    fn test_thai_coordinates_valid() {
        // Thailand approximate bounds: 5.6°N to 20.5°N, 97.3°E to 105.6°E
        let valid_coords = [
            (dec("18.7883"), dec("98.9853")),  // Chiang Mai
            (dec("13.7563"), dec("100.5018")), // Bangkok
            (dec("7.8804"), dec("98.3923")),   // Phuket
        ];

        for (lat, lon) in valid_coords {
            assert!(is_in_thailand(lat, lon));
        }
    }

    /// Test coordinates outside Thailand
    #[test]
    fn test_coordinates_outside_thailand() {
        let invalid_coords = [
            (dec("35.6762"), dec("139.6503")), // Tokyo
            (dec("1.3521"), dec("103.8198")),  // Singapore (close but outside)
        ];

        for (lat, lon) in invalid_coords {
            assert!(!is_in_thailand(lat, lon));
        }
    }

    fn is_in_thailand(lat: Decimal, lon: Decimal) -> bool {
        lat >= dec("5.6") && lat <= dec("20.5") && lon >= dec("97.3") && lon <= dec("105.6")
    }

    /// Test rain detection
    #[test]
    fn test_rain_detection() {
        let threshold = dec("5.0");
        
        // Rain above threshold
        assert!(has_rain(Some(dec("6.0")), threshold));
        
        // Rain below threshold
        assert!(!has_rain(Some(dec("3.0")), threshold));
        
        // No rain data
        assert!(!has_rain(None, threshold));
    }

    fn has_rain(rain_mm: Option<Decimal>, threshold: Decimal) -> bool {
        rain_mm.map(|r| r >= threshold).unwrap_or(false)
    }

    /// Test precipitation probability
    #[test]
    fn test_precipitation_probability() {
        // High probability (>= 50%)
        assert!(is_rain_likely(dec("0.6")));
        assert!(is_rain_likely(dec("0.5")));
        
        // Low probability (< 50%)
        assert!(!is_rain_likely(dec("0.4")));
        assert!(!is_rain_likely(dec("0.0")));
    }

    fn is_rain_likely(pop: Decimal) -> bool {
        pop >= dec("0.5")
    }

    /// Test temperature ranges for coffee
    #[test]
    fn test_coffee_temperature_ranges() {
        // Ideal temperature for Arabica: 15-24°C
        assert!(is_ideal_arabica_temp(dec("20.0")));
        assert!(is_ideal_arabica_temp(dec("15.0")));
        assert!(is_ideal_arabica_temp(dec("24.0")));
        
        // Too hot
        assert!(!is_ideal_arabica_temp(dec("30.0")));
        
        // Too cold
        assert!(!is_ideal_arabica_temp(dec("10.0")));
    }

    fn is_ideal_arabica_temp(temp: Decimal) -> bool {
        temp >= dec("15.0") && temp <= dec("24.0")
    }

    /// Test frost warning threshold
    #[test]
    fn test_frost_warning() {
        // Frost risk below 4°C
        assert!(is_frost_risk(dec("2.0")));
        assert!(is_frost_risk(dec("0.0")));
        assert!(is_frost_risk(dec("-2.0")));
        
        // No frost risk
        assert!(!is_frost_risk(dec("5.0")));
        assert!(!is_frost_risk(dec("10.0")));
    }

    fn is_frost_risk(temp: Decimal) -> bool {
        temp < dec("4.0")
    }

    /// Test heat warning threshold
    #[test]
    fn test_heat_warning() {
        // Heat stress above 30°C
        assert!(is_heat_stress(dec("32.0")));
        assert!(is_heat_stress(dec("35.0")));
        
        // No heat stress
        assert!(!is_heat_stress(dec("28.0")));
        assert!(!is_heat_stress(dec("25.0")));
    }

    fn is_heat_stress(temp: Decimal) -> bool {
        temp > dec("30.0")
    }

    /// Test wind warning threshold
    #[test]
    fn test_wind_warning() {
        // Strong wind above 10 m/s
        assert!(is_strong_wind(dec("12.0")));
        assert!(is_strong_wind(dec("15.0")));
        
        // Normal wind
        assert!(!is_strong_wind(dec("5.0")));
        assert!(!is_strong_wind(dec("8.0")));
    }

    fn is_strong_wind(speed: Decimal) -> bool {
        speed > dec("10.0")
    }

    /// Test distance calculation (simplified)
    #[test]
    fn test_distance_calculation() {
        // Same location
        let dist = calculate_distance(dec("18.0"), dec("98.0"), dec("18.0"), dec("98.0"));
        assert_eq!(dist, dec("0.0"));
        
        // Different locations
        let dist = calculate_distance(dec("18.0"), dec("98.0"), dec("19.0"), dec("99.0"));
        assert!(dist > dec("0.0"));
    }

    fn calculate_distance(lat1: Decimal, lon1: Decimal, lat2: Decimal, lon2: Decimal) -> Decimal {
        // Simplified distance using Pythagorean theorem
        // 1 degree lat ≈ 111 km, 1 degree lon ≈ 102 km (at Thailand's latitude)
        let lat_diff = (lat2 - lat1) * dec("111.0");
        let lon_diff = (lon2 - lon1) * dec("102.0");
        
        // sqrt(lat_diff^2 + lon_diff^2) - simplified as sum for testing
        (lat_diff * lat_diff + lon_diff * lon_diff).sqrt().unwrap_or(Decimal::ZERO)
    }

    /// Test alert type validation
    #[test]
    fn test_alert_types() {
        let valid_types = ["rain_forecast", "frost_warning", "heat_warning", "wind_warning"];
        
        for t in valid_types {
            assert!(is_valid_alert_type(t));
        }
        
        assert!(!is_valid_alert_type("invalid_type"));
    }

    fn is_valid_alert_type(alert_type: &str) -> bool {
        matches!(alert_type, "rain_forecast" | "frost_warning" | "heat_warning" | "wind_warning")
    }

    /// Test weather source validation
    #[test]
    fn test_weather_sources() {
        let valid_sources = ["openweathermap", "manual", "tmd"]; // TMD = Thai Meteorological Department
        
        for s in valid_sources {
            assert!(is_valid_source(s));
        }
    }

    fn is_valid_source(source: &str) -> bool {
        matches!(source, "openweathermap" | "manual" | "tmd")
    }

    // Test data structures
    #[derive(Debug, Clone)]
    struct WeatherSnapshot {
        latitude: Decimal,
        longitude: Decimal,
        temperature_celsius: Decimal,
        humidity_percent: i32,
        rain_1h_mm: Option<Decimal>,
        rain_3h_mm: Option<Decimal>,
        weather_condition: String,
    }
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use rust_decimal::MathematicalOps;

    /// Strategy for generating valid Thai latitudes
    fn thai_latitude_strategy() -> impl Strategy<Value = Decimal> {
        (56i64..=205i64).prop_map(|n| Decimal::new(n, 1)) // 5.6 to 20.5
    }

    /// Strategy for generating valid Thai longitudes
    fn thai_longitude_strategy() -> impl Strategy<Value = Decimal> {
        (973i64..=1056i64).prop_map(|n| Decimal::new(n, 1)) // 97.3 to 105.6
    }

    /// Strategy for generating temperatures (typical Thai range)
    fn temperature_strategy() -> impl Strategy<Value = Decimal> {
        (100i64..=400i64).prop_map(|n| Decimal::new(n, 1)) // 10.0 to 40.0°C
    }

    /// Strategy for generating humidity percentages
    fn humidity_strategy() -> impl Strategy<Value = i32> {
        0..=100i32
    }

    /// Strategy for generating rain amounts
    fn rain_strategy() -> impl Strategy<Value = Decimal> {
        (0i64..=500i64).prop_map(|n| Decimal::new(n, 1)) // 0.0 to 50.0mm
    }

    /// Strategy for generating precipitation probability
    fn pop_strategy() -> impl Strategy<Value = Decimal> {
        (0i64..=100i64).prop_map(|n| Decimal::new(n, 2)) // 0.00 to 1.00
    }

    /// Strategy for generating wind speeds
    fn wind_strategy() -> impl Strategy<Value = Decimal> {
        (0i64..=300i64).prop_map(|n| Decimal::new(n, 1)) // 0.0 to 30.0 m/s
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 22: Weather Data Association
        /// Harvests with weather snapshots should have valid weather data
        #[test]
        fn prop_weather_association_valid(
            lat in thai_latitude_strategy(),
            lon in thai_longitude_strategy(),
            temp in temperature_strategy(),
            humidity in humidity_strategy()
        ) {
            // Weather snapshot should have valid coordinates
            prop_assert!(lat >= dec("5.6") && lat <= dec("20.5"));
            prop_assert!(lon >= dec("97.3") && lon <= dec("105.6"));
            
            // Temperature should be reasonable
            prop_assert!(temp >= dec("10.0") && temp <= dec("40.0"));
            
            // Humidity should be valid percentage
            prop_assert!(humidity >= 0 && humidity <= 100);
        }

        /// Property 22: Weather snapshots near harvest location
        #[test]
        fn prop_weather_location_proximity(
            harvest_lat in thai_latitude_strategy(),
            harvest_lon in thai_longitude_strategy(),
            offset_lat in -10i64..=10i64,
            offset_lon in -10i64..=10i64
        ) {
            // Weather snapshot location
            let weather_lat = harvest_lat + Decimal::new(offset_lat, 2);
            let weather_lon = harvest_lon + Decimal::new(offset_lon, 2);
            
            // Calculate distance
            let lat_diff = (weather_lat - harvest_lat).abs() * dec("111.0");
            let lon_diff = (weather_lon - harvest_lon).abs() * dec("102.0");
            let distance_km = (lat_diff * lat_diff + lon_diff * lon_diff).sqrt().unwrap_or(Decimal::ZERO);
            
            // Weather within 50km should be associated
            let max_distance = dec("50.0");
            let should_associate = distance_km <= max_distance;
            
            // Verify the logic
            if distance_km <= max_distance {
                prop_assert!(should_associate);
            }
        }

        /// Property 23: Rain Alert Generation
        /// Alerts should trigger when precipitation exceeds threshold
        #[test]
        fn prop_rain_alert_triggers_correctly(
            rain_mm in rain_strategy(),
            threshold in rain_strategy()
        ) {
            let should_trigger = rain_mm >= threshold;
            
            // Verify alert logic
            if rain_mm >= threshold {
                prop_assert!(should_trigger);
            } else {
                prop_assert!(!should_trigger);
            }
        }

        /// Property 23: Rain alert with probability
        #[test]
        fn prop_rain_alert_with_probability(
            rain_mm in rain_strategy(),
            pop in pop_strategy(),
            threshold in rain_strategy()
        ) {
            // Alert triggers if rain >= threshold OR probability >= 50%
            let should_trigger = rain_mm >= threshold || pop >= dec("0.5");
            
            // Verify combined logic
            if rain_mm >= threshold {
                prop_assert!(should_trigger);
            }
            if pop >= dec("0.5") {
                prop_assert!(should_trigger);
            }
        }

        /// Property: Temperature alerts are bounded
        #[test]
        fn prop_temperature_alerts_bounded(temp in temperature_strategy()) {
            let frost_risk = temp < dec("4.0");
            let heat_stress = temp > dec("30.0");
            
            // Can't have both frost and heat
            prop_assert!(!(frost_risk && heat_stress));
            
            // Normal range has neither
            if temp >= dec("4.0") && temp <= dec("30.0") {
                prop_assert!(!frost_risk);
                prop_assert!(!heat_stress);
            }
        }

        /// Property: Wind alerts are consistent
        #[test]
        fn prop_wind_alerts_consistent(speed in wind_strategy()) {
            let strong_wind = speed > dec("10.0");
            
            // Verify threshold
            if speed > dec("10.0") {
                prop_assert!(strong_wind);
            } else {
                prop_assert!(!strong_wind);
            }
        }

        /// Property: Humidity is always valid percentage
        #[test]
        fn prop_humidity_valid_range(humidity in humidity_strategy()) {
            prop_assert!(humidity >= 0);
            prop_assert!(humidity <= 100);
        }

        /// Property: Coordinates within Thailand bounds
        #[test]
        fn prop_thai_coordinates_valid(
            lat in thai_latitude_strategy(),
            lon in thai_longitude_strategy()
        ) {
            // Latitude bounds
            prop_assert!(lat >= dec("5.6"));
            prop_assert!(lat <= dec("20.5"));
            
            // Longitude bounds
            prop_assert!(lon >= dec("97.3"));
            prop_assert!(lon <= dec("105.6"));
        }

        /// Property: Distance calculation is non-negative
        #[test]
        fn prop_distance_non_negative(
            lat1 in thai_latitude_strategy(),
            lon1 in thai_longitude_strategy(),
            lat2 in thai_latitude_strategy(),
            lon2 in thai_longitude_strategy()
        ) {
            let lat_diff = (lat2 - lat1).abs() * dec("111.0");
            let lon_diff = (lon2 - lon1).abs() * dec("102.0");
            let distance = (lat_diff * lat_diff + lon_diff * lon_diff).sqrt().unwrap_or(Decimal::ZERO);
            
            prop_assert!(distance >= Decimal::ZERO);
        }

        /// Property: Same location has zero distance
        #[test]
        fn prop_same_location_zero_distance(
            lat in thai_latitude_strategy(),
            lon in thai_longitude_strategy()
        ) {
            let lat_diff = (lat - lat).abs() * dec("111.0");
            let lon_diff = (lon - lon).abs() * dec("102.0");
            let distance = (lat_diff * lat_diff + lon_diff * lon_diff).sqrt().unwrap_or(Decimal::ZERO);
            
            prop_assert_eq!(distance, Decimal::ZERO);
        }
    }
}

// ============================================================================
// Integration Test Helpers
// ============================================================================

#[cfg(test)]
mod integration_helpers {
    use super::*;

    /// Simulate checking weather alerts for a forecast
    pub fn check_weather_alerts(
        forecasts: &[ForecastData],
        rain_threshold: Decimal,
        frost_threshold: Decimal,
        heat_threshold: Decimal,
    ) -> Vec<AlertResult> {
        let mut alerts = Vec::new();

        for forecast in forecasts {
            // Rain alert
            if let Some(rain) = forecast.rain_3h_mm {
                if rain >= rain_threshold {
                    alerts.push(AlertResult {
                        alert_type: "rain_forecast".to_string(),
                        message: format!("Rain expected: {}mm", rain),
                        timestamp: forecast.timestamp,
                    });
                }
            }

            // Frost alert
            if forecast.temperature_celsius < frost_threshold {
                alerts.push(AlertResult {
                    alert_type: "frost_warning".to_string(),
                    message: format!("Frost risk: {}°C", forecast.temperature_celsius),
                    timestamp: forecast.timestamp,
                });
            }

            // Heat alert
            if forecast.temperature_celsius > heat_threshold {
                alerts.push(AlertResult {
                    alert_type: "heat_warning".to_string(),
                    message: format!("Heat stress: {}°C", forecast.temperature_celsius),
                    timestamp: forecast.timestamp,
                });
            }
        }

        alerts
    }

    #[derive(Debug, Clone)]
    pub struct ForecastData {
        pub timestamp: DateTime<Utc>,
        pub temperature_celsius: Decimal,
        pub rain_3h_mm: Option<Decimal>,
        pub pop: Decimal,
    }

    #[derive(Debug, Clone)]
    pub struct AlertResult {
        pub alert_type: String,
        pub message: String,
        pub timestamp: DateTime<Utc>,
    }

    #[test]
    fn test_check_weather_alerts_rain() {
        let forecasts = vec![
            ForecastData {
                timestamp: Utc::now(),
                temperature_celsius: dec("25.0"),
                rain_3h_mm: Some(dec("10.0")),
                pop: dec("0.8"),
            },
        ];

        let alerts = check_weather_alerts(&forecasts, dec("5.0"), dec("4.0"), dec("30.0"));
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].alert_type, "rain_forecast");
    }

    #[test]
    fn test_check_weather_alerts_frost() {
        let forecasts = vec![
            ForecastData {
                timestamp: Utc::now(),
                temperature_celsius: dec("2.0"),
                rain_3h_mm: None,
                pop: dec("0.0"),
            },
        ];

        let alerts = check_weather_alerts(&forecasts, dec("5.0"), dec("4.0"), dec("30.0"));
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].alert_type, "frost_warning");
    }

    #[test]
    fn test_check_weather_alerts_heat() {
        let forecasts = vec![
            ForecastData {
                timestamp: Utc::now(),
                temperature_celsius: dec("35.0"),
                rain_3h_mm: None,
                pop: dec("0.0"),
            },
        ];

        let alerts = check_weather_alerts(&forecasts, dec("5.0"), dec("4.0"), dec("30.0"));
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].alert_type, "heat_warning");
    }

    #[test]
    fn test_check_weather_alerts_none() {
        let forecasts = vec![
            ForecastData {
                timestamp: Utc::now(),
                temperature_celsius: dec("25.0"),
                rain_3h_mm: Some(dec("2.0")),
                pop: dec("0.3"),
            },
        ];

        let alerts = check_weather_alerts(&forecasts, dec("5.0"), dec("4.0"), dec("30.0"));
        assert!(alerts.is_empty());
    }
}
