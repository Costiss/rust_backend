use crate::modules::auth::dto::{AuthResponse, RefreshTokenRequest, SignInRequest, SignUpRequest};
use crate::modules::auth::services::jwt_service::JwtService;
use crate::modules::auth::services::password_service::PasswordService;
use crate::shared::app_state::AppState;
use crate::shared::AppError;
use axum::{extract::State, Json};
use axum::{routing::post, Router};
use sqlx::PgPool;
use std::sync::Arc;

pub struct AuthState {
    pub pool: PgPool,
    pub jwt_service: JwtService,
}

pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/auth/sign-up", post(sign_up))
        .route("/api/auth/sign-in", post(sign_in))
        .route("/api/auth/refresh", post(refresh))
}

/// Sign up handler - creates a new user and returns tokens
pub async fn sign_up(
    State(app): State<Arc<AppState>>,
    Json(payload): Json<SignUpRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Validate email and password using value objects
    let email = crate::shared::objects::email::Email::new(&payload.email)
        .map_err(AppError::ValidationError)?;

    let password = crate::modules::users::objects::password::Password::new(&payload.password)
        .map_err(AppError::ValidationError)?;

    let existing = app.user_service.get_user_by_email(email.as_str()).await?;
    if existing.is_some() {
        return Err(AppError::ValidationError(
            "User with this email already exists".to_string(),
        ));
    }

    // Hash password
    let password_hash = PasswordService::hash_password(password.as_str()).await?;
    let created = app
        .user_service
        .create_user(email.as_str(), password_hash.as_str())
        .await?;
    let user_id = created.id_string();

    // Generate tokens
    let access_token = app
        .jwt_service
        .generate_access_token(&user_id, email.as_str())?;
    let refresh_token = app.jwt_service.generate_refresh_token(&user_id)?;

    app.jwt_service
        .save_refresh_token(&user_id, &refresh_token)
        .await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: app.jwt_service.get_expiry_hours() * 3600,
    }))
}

/// Sign in handler - authenticates user and returns tokens
pub async fn sign_in(
    State(app): State<Arc<AppState>>,
    Json(payload): Json<SignInRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let email = crate::shared::objects::email::Email::new(&payload.email)
        .map_err(AppError::ValidationError)?;

    let user = app
        .user_service
        .get_user_by_email(email.as_str())
        .await?
        .ok_or_else(|| AppError::AuthenticationError("Invalid email or password".to_string()))?;

    // Verify password
    let hash = user.password_hash().to_string();
    let password_valid = PasswordService::verify_password(&payload.password, &hash).await?;

    if !password_valid {
        return Err(AppError::AuthenticationError(
            "Invalid email or password".to_string(),
        ));
    }

    let user_id = user.id_string();

    // Generate tokens
    let access_token = app
        .jwt_service
        .generate_access_token(&user_id, email.as_str())?;
    let refresh_token = app.jwt_service.generate_refresh_token(&user_id)?;

    // Store refresh token
    app.jwt_service
        .save_refresh_token(&user_id, &refresh_token)
        .await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: app.jwt_service.get_expiry_hours() * 3600,
    }))
}

/// Refresh token handler - exchanges refresh token for new access token
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Get user_id from the stored token
    let stored_token = state
        .jwt_service
        .get_latest_refresh_token_by_user_id_with_token(&payload.refresh_token)
        .await?;

    let (user_id, stored_token_plain) = stored_token
        .ok_or_else(|| AppError::AuthenticationError("Invalid refresh token".to_string()))?;

    // Verify the provided token matches the stored token (both are plain, not hashed)
    if payload.refresh_token != stored_token_plain {
        return Err(AppError::AuthenticationError(
            "Invalid refresh token".to_string(),
        ));
    }

    let user = state.user_service.get_user_by_id(&user_id).await?;
    let user_email = user
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?
        .email()
        .as_str()
        .to_string();

    // Generate new tokens
    let new_access_token = state
        .jwt_service
        .generate_access_token(&user_id, &user_email)?;
    let new_refresh_token = state.jwt_service.generate_refresh_token(&user_id)?;

    state
        .jwt_service
        .save_refresh_token(&user_id, &new_refresh_token)
        .await?;

    Ok(Json(AuthResponse {
        access_token: new_access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_service.get_expiry_hours() * 3600,
    }))
}
