use crate::modules::auth::dto::{AuthResponse, RefreshTokenRequest, SignInRequest, SignUpRequest};
use crate::modules::auth::middleware::router::{protected_route, public_router};
use crate::modules::auth::middleware::AuthenticatedUser;
use crate::modules::auth::services::jwt_service::JwtService;
use crate::modules::auth::services::password_service::PasswordService;
use crate::shared::app_state::{AppContext, AppState};
use crate::shared::AppError;
use axum::{extract::State, Json};
use axum::{routing::post, Router};
use sqlx::PgPool;
use std::sync::Arc;

pub struct AuthState {
    pub pool: PgPool,
    pub jwt_service: JwtService,
}

pub fn auth_routes(ctx: &AppContext) -> Router<Arc<AppState>> {
    let public = public_router()
        .route("/api/auth/sign-up", post(sign_up))
        .route("/api/auth/sign-in", post(sign_in));

    let protected = protected_route(Router::new().route("/api/auth/refresh", post(refresh)), ctx);

    public.merge(protected)
}

/// Create a new user account
///
/// Register a new user with an email and password. The password must meet minimum requirements:
/// - Minimum 8 characters, maximum 128 characters
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one digit
///
/// Returns JWT tokens for immediate authentication.
#[utoipa::path(
    post,
    path = "/api/auth/sign-up",
    request_body = SignUpRequest,
    responses(
        (status = 200, description = "User created successfully", body = AuthResponse),
        (status = 400, description = "Invalid email format or weak password"),
        (status = 409, description = "User with this email already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Authentication"
)]
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
    let (access_token, claims) = app
        .jwt_service
        .generate_access_token(&user_id, email.as_str())?;
    let refresh_token = app.jwt_service.generate_refresh_token()?;

    app.jwt_service
        .save_refresh_token(&claims.jti, &refresh_token)
        .await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: app.jwt_service.get_expiry_hours() * 3600,
    }))
}

/// Authenticate user with email and password
///
/// Sign in with existing credentials to obtain JWT tokens.
/// Returns the same token structure as sign-up.
#[utoipa::path(
    post,
    path = "/api/auth/sign-in",
    request_body = SignInRequest,
    responses(
        (status = 200, description = "Authentication successful", body = AuthResponse),
        (status = 400, description = "Invalid email format"),
        (status = 401, description = "Invalid email or password"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Authentication"
)]
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
    let (access_token, claims) = app
        .jwt_service
        .generate_access_token(&user_id, email.as_str())?;
    let refresh_token = app.jwt_service.generate_refresh_token()?;

    app.jwt_service
        .save_refresh_token(&claims.jti, &refresh_token)
        .await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: app.jwt_service.get_expiry_hours() * 3600,
    }))
}

/// Exchange refresh token for new access token
///
/// Use the refresh token obtained from sign-up or sign-in to get a new access token
/// without re-entering credentials. Returns a new pair of tokens.
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refresh successful", body = AuthResponse),
        (status = 401, description = "Invalid or expired refresh token"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Authentication"
)]
pub async fn refresh(
    user: AuthenticatedUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let stored_token = state
        .jwt_service
        .get_refresh_token(user.jti.as_str())
        .await?
        .ok_or_else(|| {
            AppError::AuthenticationError("Refresh token not found or expired".to_string())
        })?;

    // Verify the provided token matches the stored token (both are plain, not hashed)
    if payload.refresh_token != stored_token {
        return Err(AppError::AuthenticationError(
            "Invalid refresh token".to_string(),
        ));
    }

    let (new_access_token, claims) = state
        .jwt_service
        .generate_access_token(&user.user_id, &user.email)?;
    let new_refresh_token = state.jwt_service.generate_refresh_token()?;

    state
        .jwt_service
        .save_refresh_token(&claims.jti, &new_refresh_token)
        .await?;
    tokio::spawn({
        let jwt_service = state.jwt_service.clone();
        let jti = user.jti.clone();
        async move {
            if let Err(e) = jwt_service.delete_refresh_token(&jti).await {
                tracing::error!("failed to delete old refresh token: {}", e);
            }
            tracing::debug!("old refresh token deleted successfully");
        }
    });

    Ok(Json(AuthResponse {
        access_token: new_access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_service.get_expiry_hours() * 3600,
    }))
}
