/// Password value object - ensures passwords meet minimum requirements
use crate::shared::kernel::traits::Validate;

#[derive(Debug, Clone)]
pub struct Password(String);

impl Password {
    pub fn new(password: impl Into<String>) -> Result<Self, String> {
        let password = password.into();
        let password_obj = Password(password);
        password_obj.validate()?;
        Ok(password_obj)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl Validate for Password {
    fn validate(&self) -> Result<(), String> {
        if self.0.is_empty() {
            return Err("Password cannot be empty".to_string());
        }

        if self.0.len() < 8 {
            return Err("Password must be at least 8 characters long".to_string());
        }

        if self.0.len() > 128 {
            return Err("Password must be less than 128 characters".to_string());
        }

        // At least one uppercase, one lowercase, one digit
        let has_uppercase = self.0.chars().any(|c| c.is_uppercase());
        let has_lowercase = self.0.chars().any(|c| c.is_lowercase());
        let has_digit = self.0.chars().any(|c| c.is_numeric());

        if !has_uppercase {
            return Err("Password must contain at least one uppercase letter".to_string());
        }

        if !has_lowercase {
            return Err("Password must contain at least one lowercase letter".to_string());
        }

        if !has_digit {
            return Err("Password must contain at least one digit".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_password() {
        let password = Password::new("ValidPass123").unwrap();
        assert_eq!(password.as_str(), "ValidPass123");
    }

    #[test]
    fn test_password_too_short() {
        let result = Password::new("Short1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least 8 characters"));
    }

    #[test]
    fn test_password_missing_uppercase() {
        let result = Password::new("lowercase123");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("uppercase"));
    }

    #[test]
    fn test_password_missing_lowercase() {
        let result = Password::new("UPPERCASE123");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("lowercase"));
    }

    #[test]
    fn test_password_missing_digit() {
        let result = Password::new("NoDigitPass");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("digit"));
    }

    #[test]
    fn test_password_empty() {
        let result = Password::new("");
        assert!(result.is_err());
    }
}
