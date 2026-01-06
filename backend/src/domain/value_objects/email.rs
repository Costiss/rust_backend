/// Email value object - ensures only valid emails are created
use crate::shared::kernel::traits::Validate;
use regex::Regex;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    pub fn new(email: impl Into<String>) -> Result<Self, String> {
        let email = email.into();
        let email_obj = Email(email);
        email_obj.validate()?;
        Ok(email_obj)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl Validate for Email {
    fn validate(&self) -> Result<(), String> {
        // Simple email validation using regex
        let email_regex = Regex::new(
            r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
        ).unwrap();

        if self.0.is_empty() {
            return Err("Email cannot be empty".to_string());
        }

        if self.0.len() > 254 {
            return Err("Email is too long (max 254 characters)".to_string());
        }

        if !email_regex.is_match(&self.0) {
            return Err("Invalid email format".to_string());
        }

        Ok(())
    }
}

impl FromStr for Email {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Email::new(s)
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl serde::Serialize for Email {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Email::new(s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let email = Email::new("user@example.com").unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn test_invalid_email_no_at() {
        let result = Email::new("userexample.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_email_empty() {
        let result = Email::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_email_from_str() {
        let email: Email = "test@example.com".parse().unwrap();
        assert_eq!(email.as_str(), "test@example.com");
    }
}
