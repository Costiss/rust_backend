use crate::shared::AppError;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;

pub struct PasswordService;

impl PasswordService {
    /// Hash a plain text password using Argon2
    pub fn hash_password(password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| AppError::InternalError(e.to_string()))
    }

    /// Verify a plain text password against an Argon2 hash
    pub fn verify_password(plain: &str, hash: &str) -> Result<bool, AppError> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| AppError::InternalError(e.to_string()))?;

        let argon2 = Argon2::default();

        match argon2.verify_password(plain.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(AppError::InternalError(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "ValidPass123";
        let hash = PasswordService::hash_password(password).expect("Failed to hash");
        assert_ne!(hash, password);
    }

    #[test]
    fn test_verify_password() {
        let password = "ValidPass123";
        let hash = PasswordService::hash_password(password).expect("Failed to hash");
        let result = PasswordService::verify_password(password, &hash).expect("Failed to verify");
        assert!(result);
    }

    #[test]
    fn test_verify_wrong_password() {
        let password = "ValidPass123";
        let hash = PasswordService::hash_password(password).expect("Failed to hash");
        let result =
            PasswordService::verify_password("WrongPass456", &hash).expect("Failed to verify");
        assert!(!result);
    }
}
