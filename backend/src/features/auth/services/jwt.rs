/// JWT token service for creating and validating tokens
use crate::shared::AppError;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String, // subject (user_id)
    pub email: String,
    pub iat: i64, // issued at
    pub exp: i64, // expiration
    pub token_type: String,
}

#[derive(Clone)]
pub struct JwtService {
    secret: String,
    expiry_hours: i64,
}

impl JwtService {
    pub fn new(secret: String, expiry_hours: i64) -> Self {
        Self {
            secret,
            expiry_hours,
        }
    }

    /// Generate a new access token
    pub fn generate_access_token(&self, user_id: &str, email: &str) -> Result<String, AppError> {
        let now = Utc::now().timestamp();
        let exp = now + (self.expiry_hours * 3600);

        let claims = TokenClaims {
            sub: user_id.to_string(),
            email: email.to_string(),
            iat: now,
            exp,
            token_type: "access".to_string(),
        };

        let key = EncodingKey::from_secret(self.secret.as_bytes());
        encode(&Header::default(), &claims, &key).map_err(|e| AppError::JwtError(e.to_string()))
    }

    /// Generate a refresh token (longer expiry)
    pub fn generate_refresh_token(&self, user_id: &str) -> Result<String, AppError> {
        let now = Utc::now().timestamp();
        let exp = now + (7 * 24 * 3600); // 7 days

        let claims = TokenClaims {
            sub: user_id.to_string(),
            email: String::new(),
            iat: now,
            exp,
            token_type: "refresh".to_string(),
        };

        let key = EncodingKey::from_secret(self.secret.as_bytes());
        encode(&Header::default(), &claims, &key).map_err(|e| AppError::JwtError(e.to_string()))
    }

    /// Validate and decode a token
    pub fn validate_token(&self, token: &str) -> Result<TokenClaims, AppError> {
        let key = DecodingKey::from_secret(self.secret.as_bytes());
        let validation = Validation::default();

        decode::<TokenClaims>(token, &key, &validation)
            .map(|data| data.claims)
            .map_err(|e| AppError::JwtError(e.to_string()))
    }

    pub fn get_expiry_hours(&self) -> i64 {
        self.expiry_hours
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_validate_token() {
        let service = JwtService::new("secret_key".to_string(), 24);
        let user_id = Uuid::new_v4();
        let email = "test@example.com";

        let token = service
            .generate_access_token(user_id, email)
            .expect("Failed to generate token");

        let claims = service
            .validate_token(&token)
            .expect("Failed to validate token");

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn test_invalid_token() {
        let service = JwtService::new("secret_key".to_string(), 24);
        let result = service.validate_token("invalid_token");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_refresh_token() {
        let service = JwtService::new("secret_key".to_string(), 24);
        let user_id = Uuid::new_v4();

        let token = service
            .generate_refresh_token(user_id)
            .expect("Failed to generate refresh token");

        let claims = service
            .validate_token(&token)
            .expect("Failed to validate refresh token");

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.token_type, "refresh");
    }
}
