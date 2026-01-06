use crate::shared::AppError;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use once_cell::sync::Lazy;
use rand_core::OsRng;

/// Static Argon2 instance to avoid recreating the struct on every call
static ARGON2: Lazy<Argon2> = Lazy::new(Argon2::default);

pub struct PasswordService;

impl PasswordService {
    /// Hash a plain text password using Argon2 asynchronously
    /// Uses tokio::task::spawn_blocking to prevent blocking the async runtime
    pub async fn hash_password(password: &str) -> Result<String, AppError> {
        let password = password.to_string();

        tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(OsRng);

            ARGON2
                .hash_password(password.as_bytes(), &salt)
                .map(|hash| hash.to_string())
                .map_err(|e| AppError::InternalError(e.to_string()))
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }

    /// Verify a plain text password against an Argon2 hash asynchronously
    /// Uses tokio::task::spawn_blocking to prevent blocking the async runtime
    pub async fn verify_password(plain: &str, hash: &str) -> Result<bool, AppError> {
        let plain = plain.to_string();
        let hash = hash.to_string();

        tokio::task::spawn_blocking(move || {
            let parsed_hash =
                PasswordHash::new(&hash).map_err(|e| AppError::InternalError(e.to_string()))?;

            match ARGON2.verify_password(plain.as_bytes(), &parsed_hash) {
                Ok(()) => Ok(true),
                Err(argon2::password_hash::Error::Password) => Ok(false),
                Err(e) => Err(AppError::InternalError(e.to_string())),
            }
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hash_password() {
        let password = "ValidPass123";
        let hash = PasswordService::hash_password(password)
            .await
            .expect("Failed to hash");
        assert_ne!(hash, password);
    }

    #[tokio::test]
    async fn test_verify_password() {
        let password = "ValidPass123";
        let hash = PasswordService::hash_password(password)
            .await
            .expect("Failed to hash");
        let result = PasswordService::verify_password(password, &hash)
            .await
            .expect("Failed to verify");
        assert!(result);
    }

    #[tokio::test]
    async fn test_verify_wrong_password() {
        let password = "ValidPass123";
        let hash = PasswordService::hash_password(password)
            .await
            .expect("Failed to hash");
        let result = PasswordService::verify_password("WrongPass456", &hash)
            .await
            .expect("Failed to verify");
        assert!(!result);
    }
}
