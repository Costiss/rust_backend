/// Shared error types for the entire application
/// Following the Result pattern for functional error handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::fmt;

/// Application-wide result type
pub type AppResult<T> = Result<T, AppError>;

/// Core application error type
#[derive(Debug)]
pub enum AppError {
    /// Validation error - bad input
    ValidationError(String),
    /// Authentication error - invalid credentials
    AuthenticationError(String),
    /// Authorization error - insufficient permissions
    AuthorizationError,
    /// Not found error
    NotFound(String),
    /// Database error
    DatabaseError(String),
    /// JWT error
    JwtError(String),
    /// Internal server error
    InternalError(String),
    /// External service error
    ExternalServiceError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            AppError::AuthorizationError => write!(f, "Not authorized"),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::JwtError(msg) => write!(f, "JWT error: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            AppError::ExternalServiceError(msg) => write!(f, "External service error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

/// HTTP response for errors
#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::AuthenticationError(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::AuthorizationError => (StatusCode::FORBIDDEN, "Not authorized".to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::DatabaseError(msg) => {
                // Don't expose internal DB errors to clients
                tracing::error!("Database error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::JwtError(msg) => {
                tracing::warn!("JWT error: {}", msg);
                (StatusCode::UNAUTHORIZED, "Invalid token".to_string())
            }
            AppError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::ExternalServiceError(msg) => {
                tracing::error!("External service error: {}", msg);
                (
                    StatusCode::BAD_GATEWAY,
                    "External service unavailable".to_string(),
                )
            }
        };

        let error_response = ErrorResponse {
            error: status.canonical_reason().unwrap_or("Error").to_string(),
            message: error_message,
        };

        (status, Json(error_response)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Record not found".to_string()),
            _ => AppError::DatabaseError(err.to_string()),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::JwtError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_display() {
        let err = AppError::ValidationError("Invalid email".to_string());
        assert_eq!(err.to_string(), "Validation error: Invalid email");
    }

    #[test]
    fn test_authentication_error_display() {
        let err = AppError::AuthenticationError("Invalid credentials".to_string());
        assert_eq!(err.to_string(), "Authentication error: Invalid credentials");
    }
}
