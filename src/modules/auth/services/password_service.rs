use crate::shared::AppError;

pub struct PasswordService;

impl PasswordService {
    /// Hash a plain text password
    pub fn hash_password(password: &str) -> Result<String, AppError> {
        bcrypt::hash(password, 12).map_err(|e| AppError::InternalError(e.to_string()))
    }

    /// Verify a plain text password against a hash
    pub fn verify_password(plain: &str, hash: &str) -> Result<bool, AppError> {
        bcrypt::verify(plain, hash).map_err(|e| AppError::InternalError(e.to_string()))
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
