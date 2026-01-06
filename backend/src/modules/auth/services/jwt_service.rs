use crate::{
    infrastructure::database::Database,
    modules::auth::{
        repository::refresh_token::RefreshTokenRepository,
        services::password_service::PasswordService,
    },
    shared::AppError,
    AppResult,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

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
    pool: Database,
}

impl JwtService {
    pub fn new(secret: String, expiry_hours: i64, pool: &Database) -> Self {
        Self {
            secret,
            expiry_hours,
            pool: pool.clone(),
        }
    }

    /// Generate a new access token
    pub fn generate_access_token(&self, user_id: &String, email: &str) -> Result<String, AppError> {
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
    pub fn generate_refresh_token(&self, user_id: &String) -> Result<String, AppError> {
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

    pub async fn save_refresh_token(
        &self,
        user_id: &String,
        refresh_token: &String,
    ) -> AppResult<()> {
        let token_hash = PasswordService::hash_password(&refresh_token)?;
        let id = Ulid::new().to_string();
        let expires_at = Utc::now() + chrono::Duration::days(7);

        self.pool
            .save_refresh_token(&id, user_id, &token_hash, expires_at)
            .await
    }

    pub async fn get_latest_refresh_token_by_user_id(
        &self,
        user_id: &String,
    ) -> AppResult<Option<(String,)>> {
        self.pool.get_latest_refresh_token_by_user_id(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Mock database implementation for testing
    struct MockRefreshTokenRepository {
        saved_tokens: Mutex<Vec<SavedToken>>,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct SavedToken {
        id: String,
        user_id: String,
        token_hash: String,
        expires_at: chrono::DateTime<chrono::Utc>,
    }

    #[async_trait::async_trait]
    impl RefreshTokenRepository for MockRefreshTokenRepository {
        async fn save_refresh_token(
            &self,
            id: &str,
            user_id: &str,
            token_hash: &str,
            expires_at: chrono::DateTime<chrono::Utc>,
        ) -> AppResult<()> {
            let mut tokens = self.saved_tokens.lock().unwrap();
            tokens.push(SavedToken {
                id: id.to_string(),
                user_id: user_id.to_string(),
                token_hash: token_hash.to_string(),
                expires_at,
            });
            Ok(())
        }
    }

    impl MockRefreshTokenRepository {
        fn new() -> Self {
            Self {
                saved_tokens: Mutex::new(Vec::new()),
            }
        }

        fn get_saved_tokens(&self) -> Vec<SavedToken> {
            self.saved_tokens.lock().unwrap().clone()
        }
    }

    #[test]
    fn test_generate_and_validate_token() {
        let user_id = "user-123";
        let email = "test@example.com";
        let secret = "secret_key".to_string();
        let expiry_hours = 24;

        let now = Utc::now().timestamp();
        let exp = now + (expiry_hours * 3600);

        let claims = TokenClaims {
            sub: user_id.to_string(),
            email: email.to_string(),
            iat: now,
            exp,
            token_type: "access".to_string(),
        };

        let key = jsonwebtoken::EncodingKey::from_secret(secret.as_bytes());
        let token = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &key)
            .expect("Failed to generate token");

        let validation = jsonwebtoken::Validation::default();
        let key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());
        let decoded = jsonwebtoken::decode::<TokenClaims>(&token, &key, &validation)
            .expect("Failed to validate token");

        assert_eq!(decoded.claims.sub, user_id);
        assert_eq!(decoded.claims.email, email);
        assert_eq!(decoded.claims.token_type, "access");
    }

    #[test]
    fn test_invalid_token() {
        let secret = "secret_key".to_string();
        let key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());
        let validation = jsonwebtoken::Validation::default();

        let result = jsonwebtoken::decode::<TokenClaims>("invalid_token", &key, &validation);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_refresh_token() {
        let user_id = "user-123";
        let secret = "secret_key".to_string();

        let now = Utc::now().timestamp();
        let exp = now + (7 * 24 * 3600); // 7 days

        let claims = TokenClaims {
            sub: user_id.to_string(),
            email: String::new(),
            iat: now,
            exp,
            token_type: "refresh".to_string(),
        };

        let key = jsonwebtoken::EncodingKey::from_secret(secret.as_bytes());
        let token = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims, &key)
            .expect("Failed to generate refresh token");

        let validation = jsonwebtoken::Validation::default();
        let key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());
        let decoded = jsonwebtoken::decode::<TokenClaims>(&token, &key, &validation)
            .expect("Failed to validate refresh token");

        assert_eq!(decoded.claims.sub, user_id);
        assert_eq!(decoded.claims.token_type, "refresh");
    }

    #[tokio::test]
    async fn test_save_refresh_token() {
        let mock_repo = MockRefreshTokenRepository::new();
        let user_id = "user-123";
        let refresh_token = "test_token_12345";
        let token_hash =
            PasswordService::hash_password(refresh_token).expect("Failed to hash token");

        let id = Ulid::new().to_string();
        let expires_at = Utc::now() + chrono::Duration::days(7);

        // Call the trait method
        mock_repo
            .save_refresh_token(&id, user_id, &token_hash, expires_at)
            .await
            .expect("Failed to save refresh token");

        // Verify the token was saved
        let saved_tokens = mock_repo.get_saved_tokens();
        assert_eq!(saved_tokens.len(), 1);
        assert_eq!(saved_tokens[0].user_id, user_id);
        assert_eq!(saved_tokens[0].token_hash, token_hash);
    }
}
