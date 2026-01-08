use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[schema(example = json!({"email": "user@example.com", "password": "SecurePassword123"}))]
pub struct SignUpRequest {
    /// User email address
    #[schema(example = "user@example.com")]
    pub email: String,
    /// Password (min 8 chars, must contain uppercase, lowercase, and number)
    #[schema(example = "SecurePassword123")]
    pub password: String,
    // User's Birthdate
    #[schema(example = "1990-01-01")]
    pub birthdate: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[schema(example = json!({"email": "user@example.com", "password": "SecurePassword123"}))]
pub struct SignInRequest {
    /// User email address
    #[schema(example = "user@example.com")]
    pub email: String,
    /// User password
    #[schema(example = "SecurePassword123")]
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct RefreshTokenRequest {
    /// Refresh token obtained from sign-up or sign-in
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuthResponse {
    /// JWT access token for API requests
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
    /// Refresh token for obtaining new access tokens
    #[schema(example = "dGhpc2lzYXJlZnJlc2h0b2tlbmV4YW1wbGU...")]
    pub refresh_token: String,
    /// Token type (always "Bearer")
    #[schema(example = "Bearer")]
    pub token_type: String,
    /// Token expiration time in seconds
    #[schema(example = 86400)]
    pub expires_in: i64,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UserResponse {
    /// User unique identifier
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: String,
    /// User email address
    #[schema(example = "user@example.com")]
    pub email: String,
}
