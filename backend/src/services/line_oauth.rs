//! LINE OAuth service for authentication and account linking
//!
//! Implements LINE Login OAuth 2.0 flow for:
//! - User authentication via LINE
//! - Linking LINE accounts to existing users
//! - Token management and refresh

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// LINE OAuth service
#[derive(Clone)]
pub struct LineOAuthService {
    db: PgPool,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    http_client: reqwest::Client,
}

/// LINE OAuth configuration
#[derive(Debug, Clone)]
pub struct LineOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

/// LINE connection record
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct LineConnection {
    pub id: Uuid,
    pub user_id: Uuid,
    pub line_user_id: String,
    pub display_name: Option<String>,
    pub picture_url: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub connected_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// LINE token response from OAuth
#[derive(Debug, Deserialize)]
pub struct LineTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub scope: Option<String>,
    pub id_token: Option<String>,
}

/// LINE user profile
#[derive(Debug, Deserialize, Serialize)]
pub struct LineUserProfile {
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "pictureUrl")]
    pub picture_url: Option<String>,
    #[serde(rename = "statusMessage")]
    pub status_message: Option<String>,
}

/// LINE ID token claims (decoded JWT)
#[derive(Debug, Deserialize)]
pub struct LineIdTokenClaims {
    pub iss: String,
    pub sub: String,  // LINE user ID
    pub aud: String,  // Channel ID
    pub exp: i64,
    pub iat: i64,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub email: Option<String>,
}

/// Result of LINE OAuth callback
#[derive(Debug, Serialize)]
pub struct LineOAuthResult {
    pub is_new_connection: bool,
    pub user_id: Option<Uuid>,
    pub line_user_id: String,
    pub display_name: String,
    pub picture_url: Option<String>,
}

/// Input for linking LINE to existing user
#[derive(Debug, Deserialize)]
pub struct LinkLineInput {
    pub authorization_code: String,
    pub state: Option<String>,
}

impl LineOAuthService {
    /// Create a new LINE OAuth service
    pub fn new(db: PgPool, config: LineOAuthConfig) -> Self {
        Self {
            db,
            client_id: config.client_id,
            client_secret: config.client_secret,
            redirect_uri: config.redirect_uri,
            http_client: reqwest::Client::new(),
        }
    }

    /// Create from environment variables
    pub fn from_env(db: PgPool) -> Option<Self> {
        let client_id = std::env::var("LINE_CHANNEL_ID").ok()?;
        let client_secret = std::env::var("LINE_CHANNEL_SECRET").ok()?;
        let redirect_uri = std::env::var("LINE_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:3000/auth/line/callback".to_string());

        Some(Self::new(
            db,
            LineOAuthConfig {
                client_id,
                client_secret,
                redirect_uri,
            },
        ))
    }

    /// Generate LINE OAuth authorization URL
    pub fn get_authorization_url(&self, state: &str) -> String {
        // URL encode the redirect URI and state
        let encoded_redirect = self.redirect_uri
            .replace(":", "%3A")
            .replace("/", "%2F");
        let encoded_state = state
            .replace("-", "%2D");
        
        format!(
            "https://access.line.me/oauth2/v2.1/authorize?\
            response_type=code&\
            client_id={}&\
            redirect_uri={}&\
            state={}&\
            scope=profile%20openid%20email",
            self.client_id,
            encoded_redirect,
            encoded_state
        )
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code(&self, code: &str) -> AppResult<LineTokenResponse> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.redirect_uri),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];

        let response = self
            .http_client
            .post("https://api.line.me/oauth2/v2.1/token")
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("LINE API error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalService(format!(
                "LINE token exchange failed: {}",
                error_text
            )));
        }

        let token_response: LineTokenResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalService(format!("Failed to parse LINE response: {}", e)))?;

        Ok(token_response)
    }

    /// Get user profile from LINE
    pub async fn get_user_profile(&self, access_token: &str) -> AppResult<LineUserProfile> {
        let response = self
            .http_client
            .get("https://api.line.me/v2/profile")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("LINE API error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalService(format!(
                "LINE profile fetch failed: {}",
                error_text
            )));
        }

        let profile: LineUserProfile = response
            .json()
            .await
            .map_err(|e| AppError::ExternalService(format!("Failed to parse LINE profile: {}", e)))?;

        Ok(profile)
    }

    /// Refresh LINE access token
    pub async fn refresh_token(&self, refresh_token: &str) -> AppResult<LineTokenResponse> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];

        let response = self
            .http_client
            .post("https://api.line.me/oauth2/v2.1/token")
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("LINE API error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalService(format!(
                "LINE token refresh failed: {}",
                error_text
            )));
        }

        let token_response: LineTokenResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalService(format!("Failed to parse LINE response: {}", e)))?;

        Ok(token_response)
    }

    /// Revoke LINE access token
    pub async fn revoke_token(&self, access_token: &str) -> AppResult<()> {
        let params = [
            ("access_token", access_token),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];

        let response = self
            .http_client
            .post("https://api.line.me/oauth2/v2.1/revoke")
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("LINE API error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalService(format!(
                "LINE token revoke failed: {}",
                error_text
            )));
        }

        Ok(())
    }

    // ========================================================================
    // Database Operations
    // ========================================================================

    /// Handle LINE OAuth callback - link to existing user or return profile for new user
    pub async fn handle_callback(
        &self,
        code: &str,
        user_id: Option<Uuid>,
    ) -> AppResult<LineOAuthResult> {
        // Exchange code for tokens
        let tokens = self.exchange_code(code).await?;

        // Get user profile
        let profile = self.get_user_profile(&tokens.access_token).await?;

        // Check if LINE user is already connected
        let existing = self.get_connection_by_line_id(&profile.user_id).await?;

        if let Some(connection) = existing {
            // Already connected - update tokens
            self.update_connection_tokens(
                connection.id,
                &tokens.access_token,
                tokens.refresh_token.as_deref(),
                tokens.expires_in,
            )
            .await?;

            return Ok(LineOAuthResult {
                is_new_connection: false,
                user_id: Some(connection.user_id),
                line_user_id: profile.user_id,
                display_name: profile.display_name,
                picture_url: profile.picture_url,
            });
        }

        // New connection
        if let Some(uid) = user_id {
            // Link to existing user
            self.create_connection(
                uid,
                &profile.user_id,
                &profile.display_name,
                profile.picture_url.as_deref(),
                &tokens.access_token,
                tokens.refresh_token.as_deref(),
                tokens.expires_in,
            )
            .await?;

            Ok(LineOAuthResult {
                is_new_connection: true,
                user_id: Some(uid),
                line_user_id: profile.user_id,
                display_name: profile.display_name,
                picture_url: profile.picture_url,
            })
        } else {
            // Return profile for potential new user creation
            Ok(LineOAuthResult {
                is_new_connection: true,
                user_id: None,
                line_user_id: profile.user_id,
                display_name: profile.display_name,
                picture_url: profile.picture_url,
            })
        }
    }

    /// Create a new LINE connection
    pub async fn create_connection(
        &self,
        user_id: Uuid,
        line_user_id: &str,
        display_name: &str,
        picture_url: Option<&str>,
        access_token: &str,
        refresh_token: Option<&str>,
        expires_in: i64,
    ) -> AppResult<LineConnection> {
        let token_expires_at = Utc::now() + Duration::seconds(expires_in);

        let connection = sqlx::query_as::<_, LineConnection>(
            r#"
            INSERT INTO line_connections (
                user_id, line_user_id, display_name, picture_url,
                access_token, refresh_token, token_expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_id, line_user_id, display_name, picture_url,
                      access_token, refresh_token, token_expires_at,
                      connected_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(line_user_id)
        .bind(display_name)
        .bind(picture_url)
        .bind(access_token)
        .bind(refresh_token)
        .bind(token_expires_at)
        .fetch_one(&self.db)
        .await?;

        Ok(connection)
    }

    /// Get LINE connection by user ID
    pub async fn get_connection(&self, user_id: Uuid) -> AppResult<Option<LineConnection>> {
        let connection = sqlx::query_as::<_, LineConnection>(
            r#"
            SELECT id, user_id, line_user_id, display_name, picture_url,
                   access_token, refresh_token, token_expires_at,
                   connected_at, updated_at
            FROM line_connections
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(connection)
    }

    /// Get LINE connection by LINE user ID
    pub async fn get_connection_by_line_id(
        &self,
        line_user_id: &str,
    ) -> AppResult<Option<LineConnection>> {
        let connection = sqlx::query_as::<_, LineConnection>(
            r#"
            SELECT id, user_id, line_user_id, display_name, picture_url,
                   access_token, refresh_token, token_expires_at,
                   connected_at, updated_at
            FROM line_connections
            WHERE line_user_id = $1
            "#,
        )
        .bind(line_user_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(connection)
    }

    /// Update connection tokens
    pub async fn update_connection_tokens(
        &self,
        connection_id: Uuid,
        access_token: &str,
        refresh_token: Option<&str>,
        expires_in: i64,
    ) -> AppResult<()> {
        let token_expires_at = Utc::now() + Duration::seconds(expires_in);

        sqlx::query(
            r#"
            UPDATE line_connections
            SET access_token = $2,
                refresh_token = COALESCE($3, refresh_token),
                token_expires_at = $4,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(connection_id)
        .bind(access_token)
        .bind(refresh_token)
        .bind(token_expires_at)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Disconnect LINE from user
    pub async fn disconnect(&self, user_id: Uuid) -> AppResult<bool> {
        // Get connection to revoke token
        if let Some(connection) = self.get_connection(user_id).await? {
            // Revoke token if available
            if let Some(token) = &connection.access_token {
                let _ = self.revoke_token(token).await; // Ignore errors
            }

            // Delete connection
            let result = sqlx::query("DELETE FROM line_connections WHERE user_id = $1")
                .bind(user_id)
                .execute(&self.db)
                .await?;

            Ok(result.rows_affected() > 0)
        } else {
            Ok(false)
        }
    }

    /// Check if user has LINE connected
    pub async fn is_connected(&self, user_id: Uuid) -> AppResult<bool> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM line_connections WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(count > 0)
    }

    /// Get valid access token (refresh if needed)
    pub async fn get_valid_access_token(&self, user_id: Uuid) -> AppResult<Option<String>> {
        let connection = match self.get_connection(user_id).await? {
            Some(c) => c,
            None => return Ok(None),
        };

        // Check if token is expired or about to expire (within 5 minutes)
        let needs_refresh = connection
            .token_expires_at
            .map(|exp| exp < Utc::now() + Duration::minutes(5))
            .unwrap_or(true);

        if needs_refresh {
            // Try to refresh
            if let Some(refresh_token) = &connection.refresh_token {
                match self.refresh_token(refresh_token).await {
                    Ok(new_tokens) => {
                        self.update_connection_tokens(
                            connection.id,
                            &new_tokens.access_token,
                            new_tokens.refresh_token.as_deref(),
                            new_tokens.expires_in,
                        )
                        .await?;
                        return Ok(Some(new_tokens.access_token));
                    }
                    Err(_) => {
                        // Refresh failed, return existing token if available
                        return Ok(connection.access_token);
                    }
                }
            }
        }

        Ok(connection.access_token)
    }
}
