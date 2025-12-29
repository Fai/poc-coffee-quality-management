//! Route definitions for the Coffee Quality Management Platform

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::{handlers, middleware::auth_middleware, AppState};

/// Create API routes
pub fn api_routes() -> Router<AppState> {
    Router::new()
        // Health check (public)
        .route("/health", get(handlers::health_check))
        // Auth routes (public)
        .nest("/auth", auth_routes())
        // LINE webhook (public - for LINE Messaging API)
        .route("/webhook/line", post(handlers::handle_line_webhook))
        // Public traceability routes (unauthenticated - for QR code scanning)
        .route("/trace/:code", get(handlers::get_traceability_view))
        // Protected routes - role management
        .nest("/roles", role_routes())
        // Protected routes - plot management
        .nest("/plots", plot_routes())
        // Protected routes - lot management
        .nest("/lots", lot_routes())
        // Protected routes - harvest management
        .nest("/harvests", harvest_routes())
        // Protected routes - processing management
        .nest("/processing", processing_routes())
        // Protected routes - grading management
        .nest("/gradings", grading_routes())
        // Protected routes - cupping management
        .nest("/cupping", cupping_routes())
        // Protected routes - inventory management
        .nest("/inventory", inventory_routes())
        // Protected routes - roasting management
        .nest("/roasting", roasting_routes())
        // Protected routes - weather management
        .nest("/weather", weather_routes())
        // Protected routes - certification management
        .nest("/certifications", certification_routes())
        // Protected routes - notification management
        .nest("/notifications", notification_routes())
}

/// Authentication routes (public)
fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/refresh", post(handlers::refresh))
        // LINE OAuth (public endpoints)
        .route("/line", get(handlers::get_authorization_url))
        .route("/line/callback/public", get(handlers::handle_public_callback))
        // LINE OAuth (protected endpoints)
        .nest("/line", line_oauth_routes())
}

/// LINE OAuth routes (protected)
fn line_oauth_routes() -> Router<AppState> {
    Router::new()
        .route("/callback", get(handlers::handle_callback))
        .route("/status", get(handlers::get_connection_status))
        .route("/connection", get(handlers::get_connection))
        .route("/", delete(handlers::disconnect_line))
        .route_layer(middleware::from_fn(auth_middleware))
}

/// Role management routes (protected)
fn role_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_roles).post(handlers::create_role))
        .route("/permissions", get(handlers::list_permissions))
        .route(
            "/:role_id",
            get(handlers::get_role)
                .put(handlers::update_role)
                .delete(handlers::delete_role),
        )
        .route_layer(middleware::from_fn(auth_middleware))
}

/// Plot management routes (protected)
fn plot_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_plots).post(handlers::create_plot))
        .route(
            "/:plot_id",
            get(handlers::get_plot)
                .put(handlers::update_plot)
                .delete(handlers::delete_plot),
        )
        .route("/:plot_id/statistics", get(handlers::get_plot_statistics))
        .route(
            "/:plot_id/varieties",
            post(handlers::add_variety),
        )
        .route(
            "/:plot_id/varieties/:variety_id",
            delete(handlers::remove_variety),
        )
        .route_layer(middleware::from_fn(auth_middleware))
}

/// Lot management routes (protected)
fn lot_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_lots).post(handlers::create_lot))
        .route("/blend", post(handlers::blend_lots))
        .route(
            "/:lot_id",
            get(handlers::get_lot)
                .put(handlers::update_lot),
        )
        .route("/:lot_id/harvests", get(handlers::get_harvests_by_lot))
        .route("/:lot_id/processing", get(handlers::get_processing_by_lot))
        .route("/:lot_id/gradings", get(handlers::get_grading_history))
        .route("/:lot_id/gradings/compare", get(handlers::get_grading_comparison))
        .route_layer(middleware::from_fn(auth_middleware))
}

/// Harvest management routes (protected)
fn harvest_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_harvests).post(handlers::record_harvest))
        .route(
            "/:harvest_id",
            get(handlers::get_harvest)
                .put(handlers::update_harvest)
                .delete(handlers::delete_harvest),
        )
        .route_layer(middleware::from_fn(auth_middleware))
}

/// Processing management routes (protected)
fn processing_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_processing).post(handlers::start_processing))
        .route(
            "/:processing_id",
            get(handlers::get_processing),
        )
        .route("/:processing_id/fermentation", post(handlers::log_fermentation))
        .route("/:processing_id/drying", post(handlers::log_drying))
        .route("/:processing_id/complete", post(handlers::complete_processing))
        .route_layer(middleware::from_fn(auth_middleware))
}

/// Grading management routes (protected)
fn grading_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_gradings).post(handlers::record_grading))
        .route("/ai", post(handlers::record_grading_with_ai))
        .route("/:grading_id", get(handlers::get_grading))
        .route_layer(middleware::from_fn(auth_middleware))
}

/// Cupping management routes (protected)
fn cupping_routes() -> Router<AppState> {
    Router::new()
        .route("/sessions", get(handlers::list_cupping_sessions).post(handlers::create_cupping_session))
        .route("/sessions/:session_id", get(handlers::get_cupping_session))
        .route("/sessions/:session_id/samples", post(handlers::add_cupping_sample))
        .route("/lots/:lot_id/history", get(handlers::get_lot_cupping_history))
        .route("/lots/:lot_id/trend", get(handlers::get_lot_cupping_trend))
        .route_layer(middleware::from_fn(auth_middleware))
}

/// Inventory management routes (protected)
fn inventory_routes() -> Router<AppState> {
    Router::new()
        // Transactions
        .route("/transactions", get(handlers::list_transactions).post(handlers::record_transaction))
        .route("/lots/:lot_id/transactions", get(handlers::get_lot_transactions))
        .route("/lots/:lot_id/balance", get(handlers::get_inventory_balance))
        .route("/lots/:lot_id/valuation", get(handlers::get_inventory_valuation))
        // Alerts
        .route("/alerts", get(handlers::list_alerts).post(handlers::create_alert))
        .route("/alerts/triggered", get(handlers::get_triggered_alerts))
        .route(
            "/alerts/:alert_id",
            put(handlers::update_alert).delete(handlers::delete_alert),
        )
        // Summary
        .route("/summary", get(handlers::get_inventory_summary))
        .route_layer(middleware::from_fn(auth_middleware))
}

/// Roasting management routes (protected)
fn roasting_routes() -> Router<AppState> {
    Router::new()
        // Profile templates
        .route("/templates", get(handlers::list_templates).post(handlers::create_template))
        .route(
            "/templates/:template_id",
            get(handlers::get_template)
                .put(handlers::update_template)
                .delete(handlers::delete_template),
        )
        // Roast sessions
        .route("/sessions", get(handlers::list_sessions).post(handlers::start_session))
        .route("/sessions/:session_id", get(handlers::get_session))
        .route("/sessions/:session_id/temperature", post(handlers::log_temperature))
        .route("/sessions/:session_id/milestones", post(handlers::log_milestones))
        .route("/sessions/:session_id/complete", post(handlers::complete_session))
        .route("/sessions/:session_id/fail", post(handlers::fail_session))
        .route("/sessions/:session_id/cuppings", get(handlers::get_session_cuppings))
        // Sessions by lot
        .route("/lots/:lot_id/sessions", get(handlers::get_sessions_by_lot))
        .route_layer(middleware::from_fn(auth_middleware))
}

/// Weather management routes (protected)
fn weather_routes() -> Router<AppState> {
    Router::new()
        // Snapshots
        .route("/snapshots", get(handlers::get_weather_snapshots_by_range).post(handlers::store_weather_snapshot))
        .route("/snapshots/:snapshot_id", get(handlers::get_weather_snapshot))
        .route("/snapshots/location", get(handlers::get_weather_snapshots_by_location))
        // Current weather and forecast (from API)
        .route("/current", get(handlers::fetch_current_weather))
        .route("/forecast", get(handlers::get_weather_forecast))
        // Harvest weather
        .route("/harvests/:harvest_id", get(handlers::get_harvest_weather).post(handlers::link_weather_to_harvest))
        // Harvest window recommendations
        .route("/harvest-windows", get(handlers::get_harvest_window_recommendations))
        // Alerts
        .route("/alerts", get(handlers::list_weather_alerts).post(handlers::create_weather_alert))
        .route("/alerts/:alert_id", delete(handlers::delete_weather_alert))
        .route("/alerts/check-rain", get(handlers::check_rain_alerts))
        .route_layer(middleware::from_fn(auth_middleware))
}


/// Certification management routes (protected)
fn certification_routes() -> Router<AppState> {
    Router::new()
        // Certifications CRUD
        .route("/", get(handlers::list_certifications).post(handlers::create_certification))
        .route(
            "/:certification_id",
            get(handlers::get_certification)
                .put(handlers::update_certification)
                .delete(handlers::delete_certification),
        )
        .route("/:certification_id/summary", get(handlers::get_certification_with_compliance))
        // Documents
        .route("/:certification_id/documents", get(handlers::list_documents).post(handlers::upload_document))
        .route("/:certification_id/documents/:document_id", delete(handlers::delete_document))
        // Compliance
        .route("/:certification_id/compliance", get(handlers::get_compliance))
        .route("/:certification_id/compliance/:requirement_id", put(handlers::update_compliance))
        // Requirements (by type)
        .route("/requirements/:cert_type", get(handlers::get_requirements))
        // Expiration alerts
        .route("/expiring", get(handlers::get_expiring_certifications))
        .route("/alerts/check", get(handlers::check_expiration_alerts))
        // Traceability integration
        .route("/for-lot", get(handlers::get_certifications_for_lot))
        .route_layer(middleware::from_fn(auth_middleware))
}

/// Notification management routes (protected)
fn notification_routes() -> Router<AppState> {
    Router::new()
        // Preferences
        .route("/preferences", get(handlers::get_preferences).put(handlers::update_preferences))
        // In-app notifications
        .route("/", get(handlers::get_notifications))
        .route("/unread-count", get(handlers::get_unread_count))
        .route("/mark-all-read", post(handlers::mark_all_as_read))
        .route("/:notification_id/read", post(handlers::mark_as_read))
        .route("/:notification_id/dismiss", post(handlers::dismiss_notification))
        // History
        .route("/history", get(handlers::get_notification_history))
        // Send (for testing/admin)
        .route("/send", post(handlers::send_notification))
        // Triggers
        .route("/triggers/inventory", post(handlers::trigger_inventory_alerts))
        .route("/triggers/certifications", post(handlers::trigger_certification_alerts))
        .route("/triggers/weather", post(handlers::trigger_weather_alerts))
        .route("/triggers/all", post(handlers::run_all_triggers))
        // Queue processing
        .route("/queue/process", post(handlers::process_queue))
        .route_layer(middleware::from_fn(auth_middleware))
}
