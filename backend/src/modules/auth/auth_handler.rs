use crate::modules::auth::dto::{AuthResponse, RefreshTokenRequest, SignInRequest, SignUpRequest};
use crate::modules::auth::services::jwt_service::JwtService;
use crate::modules::auth::services::password_service::PasswordService;
use crate::shared::app_state::AppState;
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
    let password_valid = PasswordService::verify_password(&payload.password, user.password_hash())?;

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
    .execute(&app.pool)
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
    // Validate refresh token
    let claims = state.jwt_service.validate_token(&payload.refresh_token)?;

    if claims.token_type != "refresh" {
        return Err(AppError::AuthenticationError(
            "Invalid token type".to_string(),
        ));
    }

    // // Verify refresh token hasn't been revoked
    // let user_id = Uuid::parse_str(&claims.sub)
    //     .map_err(|_| AppError::JwtError("Invalid user ID in token".to_string()))?;

    let user_id = &claims.sub;
    let token_hash = state
        .jwt_service
        .get_latest_refresh_token_by_user_id(user_id)
        .await?;

    let stored_hash = token_hash
        .ok_or_else(|| AppError::AuthenticationError("Invalid refresh token".to_string()))?
        .0;

    // Verify the token hash matches
    let token_valid = PasswordService::verify_password(&payload.refresh_token, &stored_hash)?;

    if !token_valid {
        return Err(AppError::AuthenticationError(
            "Invalid refresh token".to_string(),
        ));
    }

    let user = state.user_service.get_user_by_id(user_id).await?;
    let user_email = user
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?
        .email()
        .as_str()
        .to_string();

    // Generate new tokens
    let new_access_token = state
        .jwt_service
        .generate_access_token(user_id, &user_email)?;
    let new_refresh_token = state.jwt_service.generate_refresh_token(user_id)?;

    state
        .jwt_service
        .save_refresh_token(user_id, &new_refresh_token)
        .await?;

    Ok(Json(AuthResponse {
        access_token: new_access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_service.get_expiry_hours() * 3600,
    }))
}
