/// Core kernel types and traits for the application

pub mod result {
    use super::super::errors::AppError;

    /// Represents a successful operation that may return a value
    pub type AppResult<T> = Result<T, AppError>;

    /// Extension trait for Result to add application-specific utilities
    pub trait ResultExt<T> {
        fn context(self, msg: &str) -> AppResult<T>;
    }

    impl<T> ResultExt<T> for AppResult<T> {
        fn context(self, msg: &str) -> AppResult<T> {
            self.map_err(|err| match err {
                AppError::DatabaseError(e) => AppError::DatabaseError(format!("{}: {}", msg, e)),
                AppError::InternalError(e) => AppError::InternalError(format!("{}: {}", msg, e)),
                other => other,
            })
        }
    }
}

/// Common traits for domain types
pub mod traits {
    /// A type that can be validated
    pub trait Validate {
        fn validate(&self) -> Result<(), String>;
    }

    /// A type that has an ID
    pub trait HasId {
        type Id;
        fn id(&self) -> Self::Id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::kernel::result::ResultExt;

    #[test]
    fn test_result_ext() {
        let err: result::AppResult<i32> =
            Err(crate::shared::errors::AppError::InternalError("test".to_string()));
        let result = err.context("Additional context");
        assert!(result.is_err());
    }
}
