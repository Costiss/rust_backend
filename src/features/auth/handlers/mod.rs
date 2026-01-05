/// Auth endpoint handlers
use crate::features::auth::models::{AuthResponse, RefreshTokenRequest, SignInRequest, SignUpRequest};
use crate::features::auth::services::{JwtService, PasswordService};
use crate::shared::AppError;
use axum::{extract::State, Json};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthState {
    pub pool: PgPool,
    pub jwt_service: JwtService,
}

/// Sign up handler - creates a new user and returns tokens
pub async fn sign_up(
    State(state): State<Arc<AuthState>>,
    Json(payload): Json<SignUpRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Validate email and password using value objects
    let email = crate::domain::Email::new(&payload.email)
        .map_err(|e| AppError::ValidationError(e))?;

    let password = crate::domain::Password::new(&payload.password)
        .map_err(|e| AppError::ValidationError(e))?;

    // Check if user already exists
    let existing = sqlx::query_as::<_, (Uuid,)>(
        "SELECT id FROM users WHERE email = $1"
    )
    .bind(email.as_str())
    .fetch_optional(&state.pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::ValidationError(
            "User with this email already exists".to_string(),
        ));
    }

    // Hash password
    let password_hash = PasswordService::hash_password(password.as_str())?;

    // Create user
    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(user_id)
    .bind(email.as_str())
    .bind(&password_hash)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await?;

    // Generate tokens
    let access_token = state.jwt_service.generate_access_token(user_id, email.as_str())?;
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
    .bind(now)
    .execute(&state.pool)
    .await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_service.get_expiry_hours() * 3600,
    }))
}

/// Sign in handler - authenticates user and returns tokens
pub async fn sign_in(
    State(state): State<Arc<AuthState>>,
    Json(payload): Json<SignInRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let email = crate::domain::Email::new(&payload.email)
        .map_err(|e| AppError::ValidationError(e))?;

    // Look up user
    let user_record = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT id, password_hash FROM users WHERE email = $1"
    )
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
    let access_token = state.jwt_service.generate_access_token(user_id, email.as_str())?;
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
    State(state): State<Arc<AuthState>>,
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
    let new_access_token = state.jwt_service.generate_access_token(user_id, &user_email)?;
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
