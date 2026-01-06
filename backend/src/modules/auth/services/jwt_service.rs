use core::time;

use crate::{
    infrastructure::config::Config,
    shared::{AppError, CacheService, RedisCacheService},
    AppResult,
};
use base64::Engine;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String, // subject (user_id)
    pub email: String,
    pub iat: i64, // issued at
    pub exp: i64, // expiration
    pub token_type: String,
    pub jti: String,
}

#[derive(Clone)]
pub struct JwtService {
    secret: String,
    expiry_hours: i64,
    refresh_expiry_days: u64,
    cache: RedisCacheService,
}

impl JwtService {
    pub fn new(config: &Config, cache: &RedisCacheService) -> Self {
        Self {
            secret: config.jwt_secret.clone(),
            expiry_hours: config.jwt_expiry_hours,
            refresh_expiry_days: config.refresh_token_expiry_days,
            cache: cache.clone(),
        }
    }

    /// Generate a new access token
    pub fn generate_access_token(
        &self,
        user_id: &String,
        email: &str,
    ) -> Result<(String, TokenClaims), AppError> {
        let now = Utc::now().timestamp();
        let exp = now + (self.expiry_hours * 3600);

        let claims = TokenClaims {
            sub: user_id.to_string(),
            email: email.to_string(),
            iat: now,
            exp,
            token_type: "access".to_string(),
            jti: Uuid::new_v4().to_string(),
        };

        let key = EncodingKey::from_secret(self.secret.as_bytes());
        let token = encode(&Header::default(), &claims, &key)
            .map_err(|e| AppError::JwtError(e.to_string()))?;

        Ok((token, claims))
    }

    /// Generate a refresh token using random 32 bytes encoded in base64
    pub fn generate_refresh_token(&self) -> Result<String, AppError> {
        let mut rng = rand::thread_rng();
        let mut bytes = vec![0u8; 32];
        rng.fill(&mut bytes[..]);

        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        Ok(engine.encode(&bytes))
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

    fn refresh_token_cache_key(&self, jti: &str) -> String {
        format!("refresh_token:{}", jti)
    }

    pub async fn save_refresh_token(&self, jti: &str, refresh_token: &str) -> AppResult<()> {
        let key = self.refresh_token_cache_key(jti);

        let ttl = time::Duration::from_secs(self.refresh_expiry_days * 24 * 3600);

        self.cache.set(&key, &refresh_token, Some(ttl)).await?;

        Ok(())
    }

    pub async fn get_refresh_token(&self, jti: &str) -> AppResult<Option<String>> {
        let key = self.refresh_token_cache_key(jti);
        match self.cache.get::<String>(&key).await {
            Ok(token) => Ok(Some(token)),
            Err(_) => Ok(None),
        }
    }

    pub async fn delete_refresh_token(&self, jti: &str) -> AppResult<()> {
        let key = self.refresh_token_cache_key(jti);
        self.cache.delete(&key).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use ulid::Ulid;

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

        async fn get_latest_refresh_token_by_user_id(
            &self,
            user_id: &str,
        ) -> AppResult<Option<(String,)>> {
            let tokens = self.saved_tokens.lock().unwrap();
            let result = tokens
                .iter()
                .filter(|t| t.user_id == user_id)
                .max_by_key(|t| t.expires_at)
                .map(|t| (t.token_hash.clone(),));
            Ok(result)
        }

        async fn get_user_id_from_refresh_token(
            &self,
            token_hash: &str,
        ) -> AppResult<Option<(String, String)>> {
            let tokens = self.saved_tokens.lock().unwrap();
            let result = tokens
                .iter()
                .find(|t| t.token_hash == token_hash)
                .map(|t| (t.user_id.clone(), t.token_hash.clone()));
            Ok(result)
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
            jti: Uuid::new_v4().to_string(),
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
            jti: Uuid::new_v4().to_string(),
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

        let id = Ulid::new().to_string();
        let expires_at = Utc::now() + chrono::Duration::days(7);

        // Call the trait method (tokens are stored plainly now)
        mock_repo
            .save_refresh_token(&id, user_id, refresh_token, expires_at)
            .await
            .expect("Failed to save refresh token");

        // Verify the token was saved
        let saved_tokens = mock_repo.get_saved_tokens();
        assert_eq!(saved_tokens.len(), 1);
        assert_eq!(saved_tokens[0].user_id, user_id);
        assert_eq!(saved_tokens[0].token_hash, refresh_token);
    }
}
