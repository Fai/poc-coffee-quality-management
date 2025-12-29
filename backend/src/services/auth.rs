//! Authentication service for user registration, login, and token management

use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::error::{AppError, AppResult};
use shared::types::Language;

/// Authentication service
#[derive(Clone)]
pub struct AuthService {
    db: PgPool,
    jwt_secret: String,
    access_token_expiry: i64,
    refresh_token_expiry: i64,
}

/// Input for registering a new business with owner account
#[derive(Debug, Deserialize)]
pub struct RegisterBusinessInput {
    pub business_name: String,
    pub business_type: String, // farmer, processor, roaster, multi
    pub business_code: String, // Short code for traceability (e.g., "DOI")
    pub owner_name: String,
    pub email: String,
    pub password: String,
    pub phone: Option<String>,
    pub province: Option<String>,
    pub preferred_language: Option<Language>,
}

/// Response after successful registration
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub business_id: Uuid,
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // User ID
    pub business_id: String,
    pub role_id: String,
    pub permissions: Vec<String>,
    pub exp: i64,
    pub iat: i64,
}

/// Authentication tokens
#[derive(Debug, Serialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// User info from database
#[derive(Debug, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub business_id: Uuid,
    pub role_id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub preferred_language: String,
    pub is_active: bool,
}

impl AuthService {
    /// Create a new AuthService instance
    pub fn new(db: PgPool, config: &Config) -> Self {
        Self {
            db,
            jwt_secret: config.jwt.secret.clone(),
            access_token_expiry: config.jwt.access_token_expiry,
            refresh_token_expiry: config.jwt.refresh_token_expiry,
        }
    }

    /// Register a new business with owner account
    pub async fn register_business(
        &self,
        input: RegisterBusinessInput,
    ) -> AppResult<RegisterResponse> {
        // Validate business code format (3-10 uppercase alphanumeric)
        if !Self::is_valid_business_code(&input.business_code) {
            return Err(AppError::Validation {
                field: "business_code".to_string(),
                message: "Business code must be 3-10 uppercase alphanumeric characters".to_string(),
                message_th: "รหัสธุรกิจต้องเป็นตัวอักษรพิมพ์ใหญ่หรือตัวเลข 3-10 ตัว".to_string(),
            });
        }

        // Validate business type
        let valid_types = ["farmer", "processor", "roaster", "multi"];
        if !valid_types.contains(&input.business_type.as_str()) {
            return Err(AppError::Validation {
                field: "business_type".to_string(),
                message: "Invalid business type".to_string(),
                message_th: "ประเภทธุรกิจไม่ถูกต้อง".to_string(),
            });
        }

        // Check if business code already exists
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM businesses WHERE business_code = $1",
        )
        .bind(&input.business_code)
        .fetch_one(&self.db)
        .await?;

        if existing > 0 {
            return Err(AppError::Conflict {
                resource: "business".to_string(),
                message: "Business code already exists".to_string(),
                message_th: "รหัสธุรกิจนี้มีอยู่แล้ว".to_string(),
            });
        }

        // Hash password
        let password_hash = hash(&input.password, DEFAULT_COST)
            .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?;

        let language = input.preferred_language.unwrap_or(Language::Thai);
        let language_str = match language {
            Language::Thai => "th",
            Language::English => "en",
        };

        // Start transaction
        let mut tx = self.db.begin().await?;

        // Create business (triggers will create default roles)
        let business_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO businesses (name, business_type, business_code, phone, province, preferred_language)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
        .bind(&input.business_name)
        .bind(&input.business_type)
        .bind(&input.business_code)
        .bind(&input.phone)
        .bind(&input.province)
        .bind(language_str)
        .fetch_one(&mut *tx)
        .await?;

        // Get the owner role (created by trigger)
        let owner_role_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM roles WHERE business_id = $1 AND name = 'owner'",
        )
        .bind(business_id)
        .fetch_one(&mut *tx)
        .await?;

        // Create owner user
        let user_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO users (business_id, role_id, email, password_hash, name, phone, preferred_language)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
        )
        .bind(business_id)
        .bind(owner_role_id)
        .bind(&input.email)
        .bind(&password_hash)
        .bind(&input.owner_name)
        .bind(&input.phone)
        .bind(language_str)
        .fetch_one(&mut *tx)
        .await?;

        // Create notification preferences with defaults
        sqlx::query(
            "INSERT INTO notification_preferences (user_id) VALUES ($1)",
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        // Commit transaction
        tx.commit().await?;

        // Get user permissions for token
        let permissions = self.get_user_permissions(user_id).await?;

        // Generate tokens
        let tokens = self.generate_tokens(user_id, business_id, owner_role_id, &permissions)?;

        // Store refresh token
        self.store_refresh_token(user_id, &tokens.refresh_token).await?;

        Ok(RegisterResponse {
            business_id,
            user_id,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            token_type: tokens.token_type,
            expires_in: tokens.expires_in,
        })
    }

    /// Authenticate user with email and password
    pub async fn login(&self, email: &str, password: &str) -> AppResult<AuthTokens> {
        // Find user by email
        let user = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, business_id, role_id, email, password_hash, name, preferred_language, is_active
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized {
            message: "Invalid email or password".to_string(),
            message_th: "อีเมลหรือรหัสผ่านไม่ถูกต้อง".to_string(),
        })?;

        // Check if user is active
        if !user.is_active {
            return Err(AppError::Unauthorized {
                message: "Account is disabled".to_string(),
                message_th: "บัญชีถูกปิดใช้งาน".to_string(),
            });
        }

        // Verify password
        let valid = verify(password, &user.password_hash)
            .map_err(|e| AppError::Internal(format!("Password verification failed: {}", e)))?;

        if !valid {
            return Err(AppError::Unauthorized {
                message: "Invalid email or password".to_string(),
                message_th: "อีเมลหรือรหัสผ่านไม่ถูกต้อง".to_string(),
            });
        }

        // Update last login
        sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
            .bind(user.id)
            .execute(&self.db)
            .await?;

        // Get permissions
        let permissions = self.get_user_permissions(user.id).await?;

        // Generate tokens
        let tokens = self.generate_tokens(user.id, user.business_id, user.role_id, &permissions)?;

        // Store refresh token
        self.store_refresh_token(user.id, &tokens.refresh_token).await?;

        Ok(tokens)
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(&self, refresh_token: &str) -> AppResult<AuthTokens> {
        // Hash the refresh token to look up
        let token_hash = Self::hash_token(refresh_token);

        // Find valid refresh token
        let token_record = sqlx::query_as::<_, (Uuid, Uuid)>(
            r#"
            SELECT rt.user_id, u.business_id
            FROM refresh_tokens rt
            JOIN users u ON u.id = rt.user_id
            WHERE rt.token_hash = $1
              AND rt.expires_at > NOW()
              AND rt.revoked_at IS NULL
              AND u.is_active = true
            "#,
        )
        .bind(&token_hash)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized {
            message: "Invalid or expired refresh token".to_string(),
            message_th: "โทเค็นรีเฟรชไม่ถูกต้องหรือหมดอายุ".to_string(),
        })?;

        let (user_id, business_id) = token_record;

        // Get user's role
        let role_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT role_id FROM users WHERE id = $1",
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        // Revoke old refresh token
        sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW() WHERE token_hash = $1")
            .bind(&token_hash)
            .execute(&self.db)
            .await?;

        // Get permissions
        let permissions = self.get_user_permissions(user_id).await?;

        // Generate new tokens
        let tokens = self.generate_tokens(user_id, business_id, role_id, &permissions)?;

        // Store new refresh token
        self.store_refresh_token(user_id, &tokens.refresh_token).await?;

        Ok(tokens)
    }

    /// Validate access token and return claims
    pub fn validate_token(&self, token: &str) -> AppResult<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AppError::Unauthorized {
            message: format!("Invalid token: {}", e),
            message_th: "โทเค็นไม่ถูกต้อง".to_string(),
        })?;

        Ok(token_data.claims)
    }

    /// Get user permissions from database
    async fn get_user_permissions(&self, user_id: Uuid) -> AppResult<Vec<String>> {
        let permissions = sqlx::query_scalar::<_, String>(
            r#"
            SELECT CONCAT(p.resource, ':', p.action)
            FROM users u
            JOIN role_permissions rp ON rp.role_id = u.role_id
            JOIN permissions p ON p.id = rp.permission_id
            WHERE u.id = $1
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(permissions)
    }

    /// Generate access and refresh tokens
    fn generate_tokens(
        &self,
        user_id: Uuid,
        business_id: Uuid,
        role_id: Uuid,
        permissions: &[String],
    ) -> AppResult<AuthTokens> {
        let now = Utc::now();
        let access_exp = now + Duration::seconds(self.access_token_expiry);
        let refresh_exp = now + Duration::seconds(self.refresh_token_expiry);

        // Access token claims
        let access_claims = Claims {
            sub: user_id.to_string(),
            business_id: business_id.to_string(),
            role_id: role_id.to_string(),
            permissions: permissions.to_vec(),
            exp: access_exp.timestamp(),
            iat: now.timestamp(),
        };

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))?;

        // Refresh token (simple random token)
        let refresh_token = Uuid::new_v4().to_string();

        Ok(AuthTokens {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.access_token_expiry,
        })
    }

    /// Store refresh token in database
    async fn store_refresh_token(&self, user_id: Uuid, token: &str) -> AppResult<()> {
        let token_hash = Self::hash_token(token);
        let expires_at = Utc::now() + Duration::seconds(self.refresh_token_expiry);

        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(user_id)
        .bind(&token_hash)
        .bind(expires_at)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Hash a token for storage
    fn hash_token(token: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        token.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Validate business code format
    fn is_valid_business_code(code: &str) -> bool {
        code.len() >= 3
            && code.len() <= 10
            && code.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_business_codes() {
        assert!(AuthService::is_valid_business_code("DOI"));
        assert!(AuthService::is_valid_business_code("CMI123"));
        assert!(AuthService::is_valid_business_code("ABCDEFGHIJ"));
    }

    #[test]
    fn test_invalid_business_codes() {
        assert!(!AuthService::is_valid_business_code("AB")); // Too short
        assert!(!AuthService::is_valid_business_code("ABCDEFGHIJK")); // Too long
        assert!(!AuthService::is_valid_business_code("abc")); // Lowercase
        assert!(!AuthService::is_valid_business_code("AB-C")); // Special char
    }
}
