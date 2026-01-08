use crate::shared::AppError;
use crate::{modules::auth::services::jwt_service::TokenClaims, shared::app_state::AppState};
use axum::extract::Request;
use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::request::Parts,
    middleware::Next,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod router;

/// Authenticated user claims extracted from JWT token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    /// User ID from the token subject claim
    pub user_id: String,
    /// User email from the token email claim
    pub email: String,
    /// Token issued at timestamp
    pub issued_at: i64,
    /// Token expiration timestamp
    pub expires_at: i64,
    /// Token ID (jti claim)
    pub jti: String,
}

impl From<TokenClaims> for AuthenticatedUser {
    fn from(claims: TokenClaims) -> Self {
        AuthenticatedUser {
            user_id: claims.sub,
            email: claims.email,
            issued_at: claims.iat,
            expires_at: claims.exp,
            jti: claims.jti,
        }
    }
}

/// JWT validation error response
#[derive(Debug, Serialize)]
pub struct TokenError {
    pub error: String,
    pub message: String,
}

/**
JWT authentication extractor for use with Axum

This extractor retrieves the authenticated user from request extensions,
which are populated by the jwt_validation_middleware.

Usage with middleware:
Use in protected routes after applying the jwt_validation_middleware

Example:
    async fn protected_route(user: AuthenticatedUser) -> impl IntoResponse {
        println!("User ID: {}", user.user_id);
    }
**/
#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the authenticated user from request extensions
        // This is populated by the jwt_validation_middleware
        parts
            .extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or_else(|| {
                AppError::authentication(
                    "Missing authentication credentials. Apply jwt_validation_middleware to this route",
                )
            })
    }
}

/**
JWT validation middleware that extracts and validates tokens

This middleware validates JWT tokens from Authorization headers
and makes the authenticated user available to subsequent handlers via request extensions.

Example Usage:

```ignore
use axum::middleware;
use crate::modules::auth::middleware::jwt_validation_middleware;

// Example structure - requires actual setup
let protected_routes = Router::new()
    .route("/api/protected", get(protected_handler))
    .layer(middleware::from_fn_with_state(
        app_state.clone(),
        jwt_validation_middleware,
    ));
```
**/
pub async fn jwt_validation_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> axum::response::Response {
    // Extract Authorization header
    tracing::debug!("validating JWT for request: {}", request.uri());
    let authorization = request.headers().get("Authorization");

    tracing::debug!("Authorization header: {:?}", authorization);
    let auth_header = authorization
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    match auth_header {
        None => AppError::authentication("Missing authorization header").into_response(),
        Some(token) => {
            match state.jwt_service.validate_token(token) {
                Ok(claims) => {
                    // Token is valid, store claims in request extensions
                    let user: AuthenticatedUser = claims.into();
                    request.extensions_mut().insert(user);
                    next.run(request).await
                }
                Err(e) => e.into_response(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_authenticated_user_from_token_claims() {
        let claims = TokenClaims {
            sub: "user-123".to_string(),
            email: "user@example.com".to_string(),
            iat: Utc::now().timestamp(),
            exp: Utc::now().timestamp() + 3600,
            token_type: "access".to_string(),
            jti: uuid::Uuid::new_v4().to_string(),
        };

        let user: AuthenticatedUser = claims.into();

        assert_eq!(user.user_id, "user-123");
        assert_eq!(user.email, "user@example.com");
        assert!(user.expires_at > user.issued_at);
    }

    #[test]
    fn test_token_error_serialization() {
        let error = TokenError {
            error: "Unauthorized".to_string(),
            message: "Invalid token".to_string(),
        };

        let json = serde_json::to_string(&error).expect("Failed to serialize");
        assert!(json.contains("Unauthorized"));
        assert!(json.contains("Invalid token"));
    }
}
