//! HTTP handlers for weather management endpoints

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppResult;
use crate::middleware::CurrentUser;
use crate::services::weather::{
    CreateWeatherAlertInput, StoreWeatherInput, WeatherAlert, WeatherService, WeatherSnapshot,
};
use crate::external::weather::WeatherForecast;
use crate::AppState;

/// Store a weather snapshot
pub async fn store_weather_snapshot(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(input): Json<StoreWeatherInput>,
) -> AppResult<Json<WeatherSnapshot>> {
    let service = WeatherService::new(state.db);
    let snapshot = service
        .store_snapshot(current_user.0.business_id, input)
        .await?;
    Ok(Json(snapshot))
}

/// Get a weather snapshot by ID
pub async fn get_weather_snapshot(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(snapshot_id): Path<Uuid>,
) -> AppResult<Json<WeatherSnapshot>> {
    let service = WeatherService::new(state.db);
    let snapshot = service
        .get_snapshot(current_user.0.business_id, snapshot_id)
        .await?;
    Ok(Json(snapshot))
}

/// Query parameters for weather snapshots by date range
#[derive(Debug, Deserialize)]
pub struct DateRangeQuery {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

/// Get weather snapshots for a date range
pub async fn get_weather_snapshots_by_range(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<DateRangeQuery>,
) -> AppResult<Json<Vec<WeatherSnapshot>>> {
    let service = WeatherService::new(state.db);
    let snapshots = service
        .get_snapshots_for_range(current_user.0.business_id, query.start_date, query.end_date)
        .await?;
    Ok(Json(snapshots))
}

/// Query parameters for weather snapshots by location
#[derive(Debug, Deserialize)]
pub struct LocationQuery {
    pub latitude: Decimal,
    pub longitude: Decimal,
    pub max_distance_km: Option<Decimal>,
    pub max_age_hours: Option<i32>,
}

/// Get weather snapshots near a location
pub async fn get_weather_snapshots_by_location(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<LocationQuery>,
) -> AppResult<Json<Vec<WeatherSnapshot>>> {
    let service = WeatherService::new(state.db);
    let max_distance = query.max_distance_km.unwrap_or(Decimal::from(50));
    let max_age = query.max_age_hours.unwrap_or(24);
    
    let snapshots = service
        .get_snapshots_near_location(
            current_user.0.business_id,
            query.latitude,
            query.longitude,
            max_distance,
            max_age,
        )
        .await?;
    Ok(Json(snapshots))
}

/// Link weather snapshot to harvest
#[derive(Debug, Deserialize)]
pub struct LinkWeatherInput {
    pub snapshot_id: Uuid,
}

pub async fn link_weather_to_harvest(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(harvest_id): Path<Uuid>,
    Json(input): Json<LinkWeatherInput>,
) -> AppResult<Json<()>> {
    let service = WeatherService::new(state.db);
    service
        .link_to_harvest(current_user.0.business_id, harvest_id, input.snapshot_id)
        .await?;
    Ok(Json(()))
}

/// Get weather for a harvest
pub async fn get_harvest_weather(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(harvest_id): Path<Uuid>,
) -> AppResult<Json<Option<WeatherSnapshot>>> {
    let service = WeatherService::new(state.db);
    let snapshot = service
        .get_harvest_weather(current_user.0.business_id, harvest_id)
        .await?;
    Ok(Json(snapshot))
}

/// Fetch current weather from API
pub async fn fetch_current_weather(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<LocationQuery>,
) -> AppResult<Json<WeatherSnapshot>> {
    // Get API key from config
    let api_key = std::env::var("CQM_WEATHER_API_KEY")
        .unwrap_or_else(|_| "".to_string());
    
    if api_key.is_empty() {
        return Err(crate::error::AppError::Internal(
            "Weather API key not configured".to_string(),
        ));
    }

    let service = WeatherService::with_client(state.db, api_key);
    let snapshot = service
        .fetch_and_store_current(current_user.0.business_id, query.latitude, query.longitude)
        .await?;
    Ok(Json(snapshot))
}

/// Get weather forecast
pub async fn get_weather_forecast(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<LocationQuery>,
) -> AppResult<Json<WeatherForecast>> {
    let api_key = std::env::var("CQM_WEATHER_API_KEY")
        .unwrap_or_else(|_| "".to_string());
    
    if api_key.is_empty() {
        return Err(crate::error::AppError::Internal(
            "Weather API key not configured".to_string(),
        ));
    }

    let service = WeatherService::with_client(state.db, api_key);
    let forecast = service
        .get_forecast(current_user.0.business_id, query.latitude, query.longitude)
        .await?;
    Ok(Json(forecast))
}

/// Create a weather alert
pub async fn create_weather_alert(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(input): Json<CreateWeatherAlertInput>,
) -> AppResult<Json<WeatherAlert>> {
    let service = WeatherService::new(state.db);
    let alert = service
        .create_alert(current_user.0.business_id, input)
        .await?;
    Ok(Json(alert))
}

/// List weather alerts
pub async fn list_weather_alerts(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<Vec<WeatherAlert>>> {
    let service = WeatherService::new(state.db);
    let alerts = service.list_alerts(current_user.0.business_id).await?;
    Ok(Json(alerts))
}

/// Delete a weather alert
pub async fn delete_weather_alert(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(alert_id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let service = WeatherService::new(state.db);
    service
        .delete_alert(current_user.0.business_id, alert_id)
        .await?;
    Ok(Json(()))
}

/// Check rain alerts response
#[derive(Debug, serde::Serialize)]
pub struct RainAlertResponse {
    pub alert: WeatherAlert,
    pub message: String,
}

/// Check rain alerts for a location
pub async fn check_rain_alerts(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<LocationQuery>,
) -> AppResult<Json<Vec<RainAlertResponse>>> {
    let api_key = std::env::var("CQM_WEATHER_API_KEY")
        .unwrap_or_else(|_| "".to_string());
    
    if api_key.is_empty() {
        return Err(crate::error::AppError::Internal(
            "Weather API key not configured".to_string(),
        ));
    }

    let service = WeatherService::with_client(state.db, api_key);
    let forecast = service
        .get_forecast(current_user.0.business_id, query.latitude, query.longitude)
        .await?;
    
    let triggered = service
        .check_rain_alerts(current_user.0.business_id, &forecast)
        .await?;
    
    let response: Vec<RainAlertResponse> = triggered
        .into_iter()
        .map(|(alert, message)| RainAlertResponse { alert, message })
        .collect();
    
    Ok(Json(response))
}

/// Query parameters for harvest window recommendations
#[derive(Debug, Deserialize)]
pub struct HarvestWindowQuery {
    pub latitude: Decimal,
    pub longitude: Decimal,
    pub ripeness_percent: Option<i32>,
}

/// Get harvest window recommendations
pub async fn get_harvest_window_recommendations(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<HarvestWindowQuery>,
) -> AppResult<Json<Vec<crate::services::weather::HarvestWindowRecommendation>>> {
    let api_key = std::env::var("CQM_WEATHER_API_KEY")
        .unwrap_or_else(|_| "".to_string());
    
    if api_key.is_empty() {
        return Err(crate::error::AppError::Internal(
            "Weather API key not configured".to_string(),
        ));
    }

    let service = WeatherService::with_client(state.db, api_key);
    let forecast = service
        .get_forecast(current_user.0.business_id, query.latitude, query.longitude)
        .await?;
    
    let recommendations = service.get_harvest_window_recommendations(&forecast, query.ripeness_percent);
    
    Ok(Json(recommendations))
}
