//! Configuration management for the Coffee Quality Management Platform
//!
//! Supports hierarchical configuration loading:
//! 1. Default values in code
//! 2. Configuration files (development.toml, production.toml)
//! 3. Environment variable overrides with CQM_ prefix

use config::{ConfigError, Environment, File};
use serde::Deserialize;

/// Main application configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Current environment (development, production)
    pub environment: String,

    /// Server configuration
    pub server: ServerConfig,

    /// Database configuration
    pub database: DatabaseConfig,

    /// JWT authentication configuration
    pub jwt: JwtConfig,

    /// LINE OAuth configuration
    pub line: LineConfig,

    /// AWS configuration for AI services
    pub aws: AwsConfig,

    /// Weather API configuration
    pub weather: WeatherConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    /// Server port
    pub port: u16,

    /// Server host
    pub host: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    /// PostgreSQL connection URL
    pub url: String,

    /// Maximum number of connections in the pool
    pub max_connections: u32,

    /// Minimum number of connections in the pool
    pub min_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    /// Secret key for signing JWT tokens
    pub secret: String,

    /// Access token expiration in seconds
    pub access_token_expiry: i64,

    /// Refresh token expiration in seconds
    pub refresh_token_expiry: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LineConfig {
    /// LINE Channel ID
    pub channel_id: String,

    /// LINE Channel Secret
    pub channel_secret: String,

    /// LINE Messaging API access token
    pub messaging_token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AwsConfig {
    /// AWS Region
    pub region: String,

    /// S3 bucket for media storage
    pub s3_bucket: String,

    /// AI Defect Detection API endpoint
    pub ai_detection_endpoint: String,

    /// AI Defect Detection API key
    pub ai_detection_api_key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WeatherConfig {
    /// Weather API endpoint
    pub api_endpoint: String,

    /// Weather API key
    pub api_key: String,
}

impl Config {
    /// Load configuration from files and environment variables
    pub fn load() -> Result<Self, ConfigError> {
        let environment = std::env::var("CQM_ENVIRONMENT").unwrap_or_else(|_| "development".into());

        let config = config::Config::builder()
            // Start with default values
            .set_default("environment", environment.clone())?
            .set_default("server.port", 3000)?
            .set_default("server.host", "0.0.0.0")?
            .set_default("database.max_connections", 10)?
            .set_default("database.min_connections", 2)?
            .set_default("jwt.access_token_expiry", 3600)?
            .set_default("jwt.refresh_token_expiry", 604800)?
            .set_default("aws.region", "ap-southeast-1")?
            // Load environment-specific config file
            .add_source(File::with_name(&format!("config/{}", environment)).required(false))
            // Override with environment variables (CQM_ prefix)
            .add_source(
                Environment::with_prefix("CQM")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        config.try_deserialize()
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "0.0.0.0".to_string(),
        }
    }
}
