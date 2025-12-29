//! Weather API client for fetching weather data
//!
//! Integrates with OpenWeatherMap API for current conditions and forecasts

use chrono::{DateTime, Utc};
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

/// Weather API client
#[derive(Clone)]
pub struct WeatherClient {
    client: Client,
    api_key: String,
    base_url: String,
}

/// Current weather conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentWeather {
    pub timestamp: DateTime<Utc>,
    pub temperature_celsius: Decimal,
    pub feels_like_celsius: Decimal,
    pub humidity_percent: i32,
    pub pressure_hpa: i32,
    pub wind_speed_mps: Decimal,
    pub wind_direction_deg: i32,
    pub cloud_coverage_percent: i32,
    pub visibility_meters: i32,
    pub weather_condition: String,
    pub weather_description: String,
    pub weather_icon: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rain_1h_mm: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rain_3h_mm: Option<Decimal>,
    pub sunrise: DateTime<Utc>,
    pub sunset: DateTime<Utc>,
}

/// Weather forecast for a specific time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastItem {
    pub timestamp: DateTime<Utc>,
    pub temperature_celsius: Decimal,
    pub feels_like_celsius: Decimal,
    pub temp_min_celsius: Decimal,
    pub temp_max_celsius: Decimal,
    pub humidity_percent: i32,
    pub pressure_hpa: i32,
    pub wind_speed_mps: Decimal,
    pub wind_direction_deg: i32,
    pub cloud_coverage_percent: i32,
    pub weather_condition: String,
    pub weather_description: String,
    pub weather_icon: String,
    pub pop: Decimal, // Probability of precipitation (0-1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rain_3h_mm: Option<Decimal>,
}

/// 7-day weather forecast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherForecast {
    pub location_name: String,
    pub latitude: Decimal,
    pub longitude: Decimal,
    pub timezone_offset_seconds: i32,
    pub forecasts: Vec<ForecastItem>,
}

/// OpenWeatherMap API response for current weather
#[derive(Debug, Deserialize)]
struct OWMCurrentResponse {
    coord: OWMCoord,
    weather: Vec<OWMWeather>,
    main: OWMMain,
    visibility: Option<i32>,
    wind: OWMWind,
    clouds: OWMClouds,
    rain: Option<OWMRain>,
    dt: i64,
    sys: OWMSys,
    timezone: i32,
    name: String,
}

#[derive(Debug, Deserialize)]
struct OWMCoord {
    lat: f64,
    lon: f64,
}

#[derive(Debug, Deserialize)]
struct OWMWeather {
    main: String,
    description: String,
    icon: String,
}

#[derive(Debug, Deserialize)]
struct OWMMain {
    temp: f64,
    feels_like: f64,
    temp_min: f64,
    temp_max: f64,
    pressure: i32,
    humidity: i32,
}

#[derive(Debug, Deserialize)]
struct OWMWind {
    speed: f64,
    deg: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct OWMClouds {
    all: i32,
}

#[derive(Debug, Deserialize)]
struct OWMRain {
    #[serde(rename = "1h")]
    one_hour: Option<f64>,
    #[serde(rename = "3h")]
    three_hour: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct OWMSys {
    sunrise: i64,
    sunset: i64,
}

/// OpenWeatherMap API response for forecast
#[derive(Debug, Deserialize)]
struct OWMForecastResponse {
    city: OWMCity,
    list: Vec<OWMForecastItem>,
}

#[derive(Debug, Deserialize)]
struct OWMCity {
    name: String,
    coord: OWMCoord,
    timezone: i32,
}

#[derive(Debug, Deserialize)]
struct OWMForecastItem {
    dt: i64,
    main: OWMMain,
    weather: Vec<OWMWeather>,
    clouds: OWMClouds,
    wind: OWMWind,
    visibility: Option<i32>,
    pop: f64,
    rain: Option<OWMForecastRain>,
}

#[derive(Debug, Deserialize)]
struct OWMForecastRain {
    #[serde(rename = "3h")]
    three_hour: Option<f64>,
}

impl WeatherClient {
    /// Create a new WeatherClient
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.openweathermap.org/data/2.5".to_string(),
        }
    }

    /// Create a new WeatherClient with custom base URL (for testing)
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
        }
    }

    /// Fetch current weather conditions by GPS coordinates
    pub async fn get_current_weather(
        &self,
        latitude: Decimal,
        longitude: Decimal,
    ) -> AppResult<CurrentWeather> {
        let url = format!(
            "{}/weather?lat={}&lon={}&appid={}&units=metric",
            self.base_url, latitude, longitude, self.api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Weather API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Weather API error: {} - {}",
                status, body
            )));
        }

        let data: OWMCurrentResponse = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse weather response: {}", e)))?;

        Ok(self.convert_current_response(data))
    }

    /// Fetch 7-day weather forecast by GPS coordinates
    pub async fn get_forecast(
        &self,
        latitude: Decimal,
        longitude: Decimal,
    ) -> AppResult<WeatherForecast> {
        let url = format!(
            "{}/forecast?lat={}&lon={}&appid={}&units=metric",
            self.base_url, latitude, longitude, self.api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Weather API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!(
                "Weather API error: {} - {}",
                status, body
            )));
        }

        let data: OWMForecastResponse = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse forecast response: {}", e)))?;

        Ok(self.convert_forecast_response(data))
    }

    /// Convert OpenWeatherMap current response to our format
    fn convert_current_response(&self, data: OWMCurrentResponse) -> CurrentWeather {
        let weather = data.weather.first();
        
        CurrentWeather {
            timestamp: DateTime::from_timestamp(data.dt, 0).unwrap_or_else(Utc::now),
            temperature_celsius: Decimal::from_f64_retain(data.main.temp).unwrap_or_default(),
            feels_like_celsius: Decimal::from_f64_retain(data.main.feels_like).unwrap_or_default(),
            humidity_percent: data.main.humidity,
            pressure_hpa: data.main.pressure,
            wind_speed_mps: Decimal::from_f64_retain(data.wind.speed).unwrap_or_default(),
            wind_direction_deg: data.wind.deg.unwrap_or(0),
            cloud_coverage_percent: data.clouds.all,
            visibility_meters: data.visibility.unwrap_or(10000),
            weather_condition: weather.map(|w| w.main.clone()).unwrap_or_default(),
            weather_description: weather.map(|w| w.description.clone()).unwrap_or_default(),
            weather_icon: weather.map(|w| w.icon.clone()).unwrap_or_default(),
            rain_1h_mm: data.rain.as_ref().and_then(|r| r.one_hour).map(|v| Decimal::from_f64_retain(v).unwrap_or_default()),
            rain_3h_mm: data.rain.as_ref().and_then(|r| r.three_hour).map(|v| Decimal::from_f64_retain(v).unwrap_or_default()),
            sunrise: DateTime::from_timestamp(data.sys.sunrise, 0).unwrap_or_else(Utc::now),
            sunset: DateTime::from_timestamp(data.sys.sunset, 0).unwrap_or_else(Utc::now),
        }
    }

    /// Convert OpenWeatherMap forecast response to our format
    fn convert_forecast_response(&self, data: OWMForecastResponse) -> WeatherForecast {
        let forecasts = data
            .list
            .into_iter()
            .map(|item| {
                let weather = item.weather.first();
                ForecastItem {
                    timestamp: DateTime::from_timestamp(item.dt, 0).unwrap_or_else(Utc::now),
                    temperature_celsius: Decimal::from_f64_retain(item.main.temp).unwrap_or_default(),
                    feels_like_celsius: Decimal::from_f64_retain(item.main.feels_like).unwrap_or_default(),
                    temp_min_celsius: Decimal::from_f64_retain(item.main.temp_min).unwrap_or_default(),
                    temp_max_celsius: Decimal::from_f64_retain(item.main.temp_max).unwrap_or_default(),
                    humidity_percent: item.main.humidity,
                    pressure_hpa: item.main.pressure,
                    wind_speed_mps: Decimal::from_f64_retain(item.wind.speed).unwrap_or_default(),
                    wind_direction_deg: item.wind.deg.unwrap_or(0),
                    cloud_coverage_percent: item.clouds.all,
                    weather_condition: weather.map(|w| w.main.clone()).unwrap_or_default(),
                    weather_description: weather.map(|w| w.description.clone()).unwrap_or_default(),
                    weather_icon: weather.map(|w| w.icon.clone()).unwrap_or_default(),
                    pop: Decimal::from_f64_retain(item.pop).unwrap_or_default(),
                    rain_3h_mm: item.rain.and_then(|r| r.three_hour).map(|v| Decimal::from_f64_retain(v).unwrap_or_default()),
                }
            })
            .collect();

        WeatherForecast {
            location_name: data.city.name,
            latitude: Decimal::from_f64_retain(data.city.coord.lat).unwrap_or_default(),
            longitude: Decimal::from_f64_retain(data.city.coord.lon).unwrap_or_default(),
            timezone_offset_seconds: data.city.timezone,
            forecasts,
        }
    }
}

/// Check if rain is expected in the forecast
pub fn has_rain_forecast(forecast: &WeatherForecast, threshold_mm: Decimal) -> bool {
    forecast.forecasts.iter().any(|f| {
        f.rain_3h_mm.map(|r| r >= threshold_mm).unwrap_or(false)
            || f.pop >= Decimal::from_f64_retain(0.5).unwrap_or_default()
    })
}

/// Get days with rain in forecast
pub fn get_rainy_days(forecast: &WeatherForecast, threshold_mm: Decimal) -> Vec<&ForecastItem> {
    forecast
        .forecasts
        .iter()
        .filter(|f| {
            f.rain_3h_mm.map(|r| r >= threshold_mm).unwrap_or(false)
                || f.pop >= Decimal::from_f64_retain(0.5).unwrap_or_default()
        })
        .collect()
}
