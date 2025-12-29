//! Authentication middleware
//!
//! JWT authentication and role-based access control middleware

use axum::{
    body::Body,
    extract::Request,
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::error::ErrorResponse;

/// Authenticated user information extracted from JWT
#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: uuid::Uuid,
    pub business_id: uuid::Uuid,
    pub role_id: uuid::Uuid,
    pub permissions: Vec<String>,
}

/// Extension trait for extracting auth user from request
impl AuthUser {
    /// Check if user has a specific permission
    pub fn has_permission(&self, resource: &str, action: &str) -> bool {
        let permission = format!("{}:{}", resource, action);
        self.permissions.contains(&permission)
    }

    /// Check if user has any of the specified permissions
    pub fn has_any_permission(&self, perms: &[(&str, &str)]) -> bool {
        perms.iter().any(|(r, a)| self.has_permission(r, a))
    }

    /// Check if user has all of the specified permissions
    pub fn has_all_permissions(&self, perms: &[(&str, &str)]) -> bool {
        perms.iter().all(|(r, a)| self.has_permission(r, a))
    }
}

/// Authentication middleware that validates JWT tokens
/// Note: This middleware extracts and validates the JWT token from the Authorization header.
/// The actual token validation is done inline to avoid state dependency issues.
pub async fn auth_middleware(mut request: Request, next: Next) -> Response {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return unauthorized_response("Missing or invalid Authorization header");
        }
    };

    // Decode and validate JWT token
    // Get JWT secret from environment (fallback for middleware without state)
    let jwt_secret = std::env::var("CQM__JWT__SECRET")
        .or_else(|_| std::env::var("CQM_JWT_SECRET"))
        .unwrap_or_else(|_| "development-secret-key".to_string());

    let claims = match decode_jwt(token, &jwt_secret) {
        Ok(claims) => claims,
        Err(msg) => {
            return unauthorized_response(&msg);
        }
    };

    // Parse UUIDs from claims
    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return unauthorized_response("Invalid user ID in token"),
    };

    let business_id = match uuid::Uuid::parse_str(&claims.business_id) {
        Ok(id) => id,
        Err(_) => return unauthorized_response("Invalid business ID in token"),
    };

    let role_id = match uuid::Uuid::parse_str(&claims.role_id) {
        Ok(id) => id,
        Err(_) => return unauthorized_response("Invalid role ID in token"),
    };

    // Create AuthUser and insert into request extensions
    let auth_user = AuthUser {
        user_id,
        business_id,
        role_id,
        permissions: claims.permissions,
    };

    request.extensions_mut().insert(auth_user);

    next.run(request).await
}

/// JWT claims structure
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Claims {
    sub: String,
    business_id: String,
    role_id: String,
    permissions: Vec<String>,
    exp: i64,
    iat: i64,
}

/// Decode and validate JWT token
fn decode_jwt(token: &str, secret: &str) -> Result<Claims, String> {
    use jsonwebtoken::{decode, DecodingKey, Validation};

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| format!("Invalid token: {}", e))
}

/// Create unauthorized response
fn unauthorized_response(message: &str) -> Response {
    let error = ErrorResponse {
        error: crate::error::ErrorDetail {
            code: "UNAUTHORIZED".to_string(),
            message_en: message.to_string(),
            message_th: "ไม่ได้รับอนุญาต".to_string(),
            field: None,
        },
    };

    (StatusCode::UNAUTHORIZED, Json(error)).into_response()
}

/// Create forbidden response
fn forbidden_response(message: &str) -> Response {
    let error = ErrorResponse {
        error: crate::error::ErrorDetail {
            code: "FORBIDDEN".to_string(),
            message_en: message.to_string(),
            message_th: "ไม่มีสิทธิ์เข้าถึง".to_string(),
            field: None,
        },
    };

    (StatusCode::FORBIDDEN, Json(error)).into_response()
}

/// Extractor for authenticated user
/// Use this in handlers to get the current user
#[derive(Clone, Debug)]
pub struct CurrentUser(pub AuthUser);

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .map(CurrentUser)
            .ok_or_else(|| {
                let error = ErrorResponse {
                    error: crate::error::ErrorDetail {
                        code: "UNAUTHORIZED".to_string(),
                        message_en: "Authentication required".to_string(),
                        message_th: "ต้องเข้าสู่ระบบก่อน".to_string(),
                        field: None,
                    },
                };
                (StatusCode::UNAUTHORIZED, Json(error))
            })
    }
}

/// Permission guard for use in handlers
/// Returns an error if the user doesn't have the required permission
pub fn check_permission(user: &AuthUser, resource: &str, action: &str) -> Result<(), Response> {
    if user.has_permission(resource, action) {
        Ok(())
    } else {
        Err(forbidden_response(&format!(
            "Permission denied: requires {}:{}",
            resource, action
        )))
    }
}
