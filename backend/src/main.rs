//! Coffee Quality Management Platform - Backend Server
//!
//! A comprehensive system for Thai coffee farmers, processors, and roasters
//! to manage quality control, traceability, and operations.

use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod external;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;

pub use config::Config;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Arc<Config>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cqm_server=debug,tower_http=debug,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = config::Config::load()?;

    tracing::info!("Starting Coffee Quality Management Server");
    tracing::info!("Environment: {}", config.environment);

    // Create database connection pool
    tracing::info!("Connecting to database...");
    let db_pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&config.database.url)
        .await?;

    tracing::info!("Database connection established");

    // Run migrations in development
    if config.environment == "development" {
        tracing::info!("Running database migrations...");
        sqlx::migrate!("./migrations").run(&db_pool).await?;
        tracing::info!("Migrations completed");
    }

    // Create application state
    let state = AppState {
        db: db_pool,
        config: Arc::new(config.clone()),
    };

    // Build application
    let app = create_app(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Create the application router with all routes and middleware
fn create_app(state: AppState) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .nest("/api/v1", routes::api_routes())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

/// Root endpoint
async fn root() -> &'static str {
    "Coffee Quality Management Platform API v1.0"
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}
