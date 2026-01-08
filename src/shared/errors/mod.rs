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
#[derive(Debug, Clone)]
pub struct AppError {
    /// Human-readable error message
    pub message: String,
    /// Machine-readable error code
    pub code: &'static str,
    /// HTTP status code
    pub status_code: StatusCode,
}

impl AppError {
    /// Create a new AppError with all fields
    pub fn new(message: impl Into<String>, code: &'static str, status_code: StatusCode) -> Self {
        Self {
            message: message.into(),
            code,
            status_code,
        }
    }

    /// Validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(message, "VALIDATION_ERROR", StatusCode::BAD_REQUEST)
    }

    /// Authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::new(message, "AUTHENTICATION_ERROR", StatusCode::UNAUTHORIZED)
    }

    /// Authorization error
    pub fn authorization() -> Self {
        Self::new(
            "Not authorized",
            "AUTHORIZATION_ERROR",
            StatusCode::FORBIDDEN,
        )
    }

    /// Not found error
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(message, "NOT_FOUND", StatusCode::NOT_FOUND)
    }

    /// Database error
    pub fn database(message: impl Into<String>) -> Self {
        let msg = message.into();
        tracing::error!("Database error: {}", msg);
        Self::new(
            "Internal server error",
            "DATABASE_ERROR",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    }

    /// JWT error
    pub fn jwt(message: impl Into<String>) -> Self {
        let msg = message.into();
        tracing::warn!("JWT error: {}", msg);
        Self::new("Invalid token", "JWT_ERROR", StatusCode::UNAUTHORIZED)
    }

    /// Internal error
    pub fn internal(message: impl Into<String>) -> Self {
        let msg = message.into();
        tracing::error!("Internal error: {}", msg);
        Self::new(
            "Internal server error",
            "INTERNAL_ERROR",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    }

    /// External service error
    pub fn external_service(message: impl Into<String>) -> Self {
        let msg = message.into();
        tracing::error!("External service error: {}", msg);
        Self::new(
            "External service unavailable",
            "EXTERNAL_SERVICE_ERROR",
            StatusCode::BAD_GATEWAY,
        )
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

/// HTTP response for errors
#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub code: &'static str,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let error_response = ErrorResponse {
            code: self.code,
            message: self.message.clone(),
        };

        (self.status_code, Json(error_response)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::not_found("Record not found"),
            _ => AppError::database(err.to_string()),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::jwt(err.to_string())
    }
}

impl From<super::CacheError> for AppError {
    fn from(err: super::CacheError) -> Self {
        AppError::internal(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_display() {
        let err = AppError::validation("Invalid email");
        assert_eq!(err.code, "VALIDATION_ERROR");
        assert_eq!(err.status_code, StatusCode::BAD_REQUEST);
        assert_eq!(err.message, "Invalid email");
    }

    #[test]
    fn test_authentication_error_display() {
        let err = AppError::authentication("Invalid credentials");
        assert_eq!(err.code, "AUTHENTICATION_ERROR");
        assert_eq!(err.status_code, StatusCode::UNAUTHORIZED);
        assert_eq!(err.message, "Invalid credentials");
    }

    #[test]
    fn test_authorization_error() {
        let err = AppError::authorization();
        assert_eq!(err.code, "AUTHORIZATION_ERROR");
        assert_eq!(err.status_code, StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_not_found_error() {
        let err = AppError::not_found("User not found");
        assert_eq!(err.code, "NOT_FOUND");
        assert_eq!(err.status_code, StatusCode::NOT_FOUND);
        assert_eq!(err.message, "User not found");
    }

    #[test]
    fn test_error_code_display() {
        assert_eq!(AppError::validation("test").code, "VALIDATION_ERROR");
        assert_eq!(AppError::not_found("test").code, "NOT_FOUND");
        assert_eq!(AppError::internal("test").code, "INTERNAL_ERROR");
    }
}
