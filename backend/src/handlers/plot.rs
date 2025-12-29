//! Plot management HTTP handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use uuid::Uuid;

use crate::middleware::CurrentUser;
use crate::services::plot::{CreatePlotInput, CreateVarietyInput, PlotService, UpdatePlotInput};
use crate::AppState;

/// List all plots for the current business
pub async fn list_plots(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> impl IntoResponse {
    let service = PlotService::new(state.db.clone());
    
    match service.get_plots(current_user.0.business_id).await {
        Ok(plots) => (StatusCode::OK, Json(serde_json::json!({ "plots": plots }))).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get a specific plot with its varieties
pub async fn get_plot(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(plot_id): Path<Uuid>,
) -> impl IntoResponse {
    let service = PlotService::new(state.db.clone());
    
    match service.get_plot_with_varieties(current_user.0.business_id, plot_id).await {
        Ok(plot) => (StatusCode::OK, Json(plot)).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Create a new plot
pub async fn create_plot(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(input): Json<CreatePlotInput>,
) -> impl IntoResponse {
    let service = PlotService::new(state.db.clone());
    
    match service.create_plot(current_user.0.business_id, input).await {
        Ok(plot) => (StatusCode::CREATED, Json(plot)).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Update a plot
pub async fn update_plot(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(plot_id): Path<Uuid>,
    Json(input): Json<UpdatePlotInput>,
) -> impl IntoResponse {
    let service = PlotService::new(state.db.clone());
    
    match service.update_plot(current_user.0.business_id, plot_id, input).await {
        Ok(plot) => (StatusCode::OK, Json(plot)).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Delete a plot
pub async fn delete_plot(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(plot_id): Path<Uuid>,
) -> impl IntoResponse {
    let service = PlotService::new(state.db.clone());
    
    match service.delete_plot(current_user.0.business_id, plot_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => e.into_response(),
    }
}

/// Add a variety to a plot
pub async fn add_variety(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(plot_id): Path<Uuid>,
    Json(input): Json<CreateVarietyInput>,
) -> impl IntoResponse {
    let service = PlotService::new(state.db.clone());
    
    match service.add_variety(current_user.0.business_id, plot_id, input).await {
        Ok(variety) => (StatusCode::CREATED, Json(variety)).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Remove a variety from a plot
pub async fn remove_variety(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((plot_id, variety_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let service = PlotService::new(state.db.clone());
    
    match service.remove_variety(current_user.0.business_id, plot_id, variety_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => e.into_response(),
    }
}

/// Get plot statistics
pub async fn get_plot_statistics(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(plot_id): Path<Uuid>,
) -> impl IntoResponse {
    let service = PlotService::new(state.db.clone());
    
    match service.get_plot_statistics(current_user.0.business_id, plot_id).await {
        Ok(stats) => (StatusCode::OK, Json(stats)).into_response(),
        Err(e) => e.into_response(),
    }
}
