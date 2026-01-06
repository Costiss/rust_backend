use crate::features::auth::models::{
    AuthResponse, RefreshTokenRequest, SignInRequest, SignUpRequest,
};
use crate::features::auth::services::{JwtService, PasswordService};
use crate::modules::auth::services::jwt_service;
use crate::shared::app_state::{self, AppState};
use crate::shared::AppError;
use axum::{extract::State, Json};
use axum::{routing::post, Router};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

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
    let password_hash = PasswordService::hash_password(password.as_str())?;
    let created = app
        .user_service
        .create_user(email.as_str(), password_hash.as_str())
        .await?;
    let user_id = created.id_str();

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
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SignInRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let email = crate::domain::Email::new(&payload.email).map_err(AppError::ValidationError)?;

    // Look up user
    let user_record =
        sqlx::query_as::<_, (Uuid, String)>("SELECT id, password_hash FROM users WHERE email = $1")
            .bind(email.as_str())
            .fetch_optional(&state.pool)
            .await?;

    let (user_id, password_hash) = user_record
        .ok_or_else(|| AppError::AuthenticationError("Invalid email or password".to_string()))?;

    // Verify password
    let password_valid = PasswordService::verify_password(&payload.password, &password_hash)?;

    if !password_valid {
        return Err(AppError::AuthenticationError(
            "Invalid email or password".to_string(),
        ));
    }

    // Generate tokens
    let access_token = state
        .jwt_service
        .generate_access_token(user_id, email.as_str())?;
    let refresh_token = state.jwt_service.generate_refresh_token(user_id)?;

    // Hash and store refresh token
    let refresh_token_hash = PasswordService::hash_password(&refresh_token)?;
    sqlx::query(
        "INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, created_at) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(refresh_token_hash)
    .bind(chrono::Utc::now() + chrono::Duration::days(7))
    .bind(chrono::Utc::now())
    .execute(&state.pool)
    .await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_service.get_expiry_hours() * 3600,
    }))
}

/// Refresh token handler - exchanges refresh token for new access token
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Validate refresh token
    let claims = state.jwt_service.validate_token(&payload.refresh_token)?;

    if claims.token_type != "refresh" {
        return Err(AppError::AuthenticationError(
            "Invalid token type".to_string(),
        ));
    }

    // Verify refresh token hasn't been revoked
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::JwtError("Invalid user ID in token".to_string()))?;

    let token_record = sqlx::query_as::<_, (String,)>(
        "SELECT token_hash FROM refresh_tokens WHERE user_id = $1 AND expires_at > NOW() ORDER BY created_at DESC LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?;

    let stored_hash = token_record
        .ok_or_else(|| AppError::AuthenticationError("Invalid refresh token".to_string()))?
        .0;

    // Verify the token hash matches
    let token_valid = PasswordService::verify_password(&payload.refresh_token, &stored_hash)?;

    if !token_valid {
        return Err(AppError::AuthenticationError(
            "Invalid refresh token".to_string(),
        ));
    }

    // Look up user email
    let user_email = sqlx::query_as::<_, (String,)>("SELECT email FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?
        .0;

    // Generate new tokens
    let new_access_token = state
        .jwt_service
        .generate_access_token(user_id, &user_email)?;
    let new_refresh_token = state.jwt_service.generate_refresh_token(user_id)?;

    // Store new refresh token
    let new_refresh_token_hash = PasswordService::hash_password(&new_refresh_token)?;
    sqlx::query(
        "INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, created_at) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(new_refresh_token_hash)
    .bind(chrono::Utc::now() + chrono::Duration::days(7))
    .bind(chrono::Utc::now())
    .execute(&state.pool)
    .await?;

    Ok(Json(AuthResponse {
        access_token: new_access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_service.get_expiry_hours() * 3600,
    }))
}
