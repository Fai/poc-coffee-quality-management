//! HTTP handlers for public lot traceability endpoints

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::{
    error::AppResult,
    services::traceability::{TraceabilityService, TraceabilityView},
    AppState,
};

/// Query parameters for traceability view
#[derive(Debug, Deserialize)]
pub struct TraceabilityQuery {
    /// Language preference: "en" or "th"
    pub lang: Option<String>,
}

/// Get public traceability view for a lot by traceability code
/// This endpoint is unauthenticated - accessible via QR code scan
pub async fn get_traceability_view(
    State(state): State<AppState>,
    Path(code): Path<String>,
    Query(query): Query<TraceabilityQuery>,
) -> AppResult<Json<TraceabilityView>> {
    let service = TraceabilityService::new(state.db);
    let view = service
        .get_traceability_view(&code, query.lang.as_deref())
        .await?;
    Ok(Json(view))
}
