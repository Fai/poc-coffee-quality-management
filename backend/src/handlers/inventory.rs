//! HTTP handlers for inventory management endpoints

use axum::{
    extract::{Path, State},
    Json,
};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::error::AppResult;
use crate::middleware::CurrentUser;
use crate::services::inventory::{
    CreateAlertInput, InventoryAlert, InventoryBalance, InventoryService, InventorySummary,
    InventoryTransaction, InventoryValuation, RecordTransactionInput, UpdateAlertInput,
};
use crate::AppState;

/// Record an inventory transaction
pub async fn record_transaction(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(input): Json<RecordTransactionInput>,
) -> AppResult<Json<InventoryTransaction>> {
    let service = InventoryService::new(state.db);
    let transaction = service
        .record_transaction(current_user.0.business_id, current_user.0.user_id, input)
        .await?;
    Ok(Json(transaction))
}

/// Get inventory balance for a lot
pub async fn get_inventory_balance(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(lot_id): Path<Uuid>,
) -> AppResult<Json<InventoryBalance>> {
    let service = InventoryService::new(state.db);
    let balance = service
        .get_balance(current_user.0.business_id, lot_id)
        .await?;
    Ok(Json(balance))
}

/// Get transactions for a lot
pub async fn get_lot_transactions(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(lot_id): Path<Uuid>,
) -> AppResult<Json<Vec<InventoryTransaction>>> {
    let service = InventoryService::new(state.db);
    let transactions = service
        .get_transactions(current_user.0.business_id, lot_id)
        .await?;
    Ok(Json(transactions))
}

/// List all transactions for the business
pub async fn list_transactions(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<Vec<InventoryTransaction>>> {
    let service = InventoryService::new(state.db);
    let transactions = service
        .list_transactions(current_user.0.business_id)
        .await?;
    Ok(Json(transactions))
}

/// Create an inventory alert
pub async fn create_alert(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(input): Json<CreateAlertInput>,
) -> AppResult<Json<InventoryAlert>> {
    let service = InventoryService::new(state.db);
    let alert = service
        .create_alert(current_user.0.business_id, input)
        .await?;
    Ok(Json(alert))
}

/// Update an inventory alert
pub async fn update_alert(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(alert_id): Path<Uuid>,
    Json(input): Json<UpdateAlertInput>,
) -> AppResult<Json<InventoryAlert>> {
    let service = InventoryService::new(state.db);
    let alert = service
        .update_alert(current_user.0.business_id, alert_id, input)
        .await?;
    Ok(Json(alert))
}

/// Delete an inventory alert
pub async fn delete_alert(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(alert_id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let service = InventoryService::new(state.db);
    service
        .delete_alert(current_user.0.business_id, alert_id)
        .await?;
    Ok(Json(()))
}

/// List all alerts for the business
pub async fn list_alerts(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<Vec<InventoryAlert>>> {
    let service = InventoryService::new(state.db);
    let alerts = service.list_alerts(current_user.0.business_id).await?;
    Ok(Json(alerts))
}

/// Get triggered alerts (alerts where balance is below threshold)
pub async fn get_triggered_alerts(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<Vec<TriggeredAlertResponse>>> {
    let service = InventoryService::new(state.db);
    let alerts = service
        .get_triggered_alerts(current_user.0.business_id)
        .await?;
    
    let response: Vec<TriggeredAlertResponse> = alerts
        .into_iter()
        .map(|(alert, balance)| TriggeredAlertResponse {
            alert,
            current_balance_kg: balance,
        })
        .collect();
    
    Ok(Json(response))
}

/// Get inventory valuation for a lot
pub async fn get_inventory_valuation(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(lot_id): Path<Uuid>,
) -> AppResult<Json<InventoryValuation>> {
    let service = InventoryService::new(state.db);
    let valuation = service
        .get_valuation(current_user.0.business_id, lot_id)
        .await?;
    Ok(Json(valuation))
}

/// Get inventory summary by stage
pub async fn get_inventory_summary(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<Json<Vec<InventorySummary>>> {
    let service = InventoryService::new(state.db);
    let summary = service
        .get_summary_by_stage(current_user.0.business_id)
        .await?;
    Ok(Json(summary))
}

/// Response for triggered alerts with current balance
#[derive(Debug, serde::Serialize)]
pub struct TriggeredAlertResponse {
    #[serde(flatten)]
    pub alert: InventoryAlert,
    pub current_balance_kg: Decimal,
}
