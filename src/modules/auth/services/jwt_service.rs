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
pub struct JwtService<Cache: CacheService = RedisCacheService> {
    secret: String,
    expiry_hours: i64,
    refresh_expiry_days: u64,
    cache: Cache,
}

impl<Cache: CacheService> JwtService<Cache> {
    pub fn new(config: &Config, cache: Cache) -> Self {
        Self {
            secret: config.jwt_secret.clone(),
            expiry_hours: config.jwt_expiry_hours,
            refresh_expiry_days: config.refresh_token_expiry_days,
            cache,
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
    use super::*;
    use crate::shared::cache::mock::MockCacheService;

    // Helper function to create a test config
    fn create_test_config() -> Config {
        Config {
            server_host: "127.0.0.1".to_string(),
            server_port: 3000,
            database_url: "postgres://localhost/test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            jwt_secret: "test-secret-key-for-testing-purposes-only-12345678".to_string(),
            jwt_expiry_hours: 1,
            refresh_token_expiry_days: 7,
        }
    }

    #[test]
    fn test_generate_access_token() {
        // Arrange
        let config = create_test_config();
        let cache = MockCacheService::new();
        let jwt_service = JwtService::new(&config, cache);
        let user_id = "user-123".to_string();
        let email = "test@example.com";

        // Act
        let result = jwt_service.generate_access_token(&user_id, email);

        // Assert
        assert!(result.is_ok(), "generate_access_token should succeed");
        let (token, claims) = result.unwrap();

        assert!(!token.is_empty(), "Token should not be empty");
        assert_eq!(claims.sub, user_id, "Subject should match user_id");
        assert_eq!(claims.email, email, "Email should match");
        assert_eq!(claims.token_type, "access", "Token type should be 'access'");
        assert!(!claims.jti.is_empty(), "JTI should not be empty");
        assert!(
            claims.iat <= claims.exp,
            "Issued at should be before expiration"
        );
    }

    #[test]
    fn test_generate_access_token_expiry() {
        // Arrange
        let config = create_test_config();
        let cache = MockCacheService::new();
        let jwt_service = JwtService::new(&config, cache);
        let user_id = "user-456".to_string();
        let email = "user@example.com";

        // Act
        let (_, claims) = jwt_service.generate_access_token(&user_id, email).unwrap();

        // Assert
        let now = Utc::now().timestamp();
        let expected_exp = now + (config.jwt_expiry_hours * 3600);
        assert!(
            (claims.exp - expected_exp).abs() < 5,
            "Expiration should be approximately 1 hour from now"
        );
    }

    #[test]
    fn test_generate_refresh_token() {
        // Arrange
        let config = create_test_config();
        let cache = MockCacheService::new();
        let jwt_service = JwtService::new(&config, cache);

        // Act
        let result1 = jwt_service.generate_refresh_token();
        let result2 = jwt_service.generate_refresh_token();

        // Assert
        assert!(result1.is_ok(), "generate_refresh_token should succeed");
        assert!(result2.is_ok(), "generate_refresh_token should succeed");

        let token1 = result1.unwrap();
        let token2 = result2.unwrap();

        assert!(!token1.is_empty(), "Token should not be empty");
        assert!(!token2.is_empty(), "Token should not be empty");
        assert_ne!(token1, token2, "Each refresh token should be unique");
        // Base64 URL-safe tokens should only contain alphanumeric, -, and _
        assert!(
            token1
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_'),
            "Token should be base64 URL-safe encoded"
        );
    }

    #[test]
    fn test_validate_token_success() {
        // Arrange
        let config = create_test_config();
        let cache = MockCacheService::new();
        let jwt_service = JwtService::new(&config, cache);
        let user_id = "user-789".to_string();
        let email = "validate@example.com";

        // Generate a valid token
        let (token, original_claims) = jwt_service.generate_access_token(&user_id, email).unwrap();

        // Act
        let result = jwt_service.validate_token(&token);

        // Assert
        assert!(
            result.is_ok(),
            "validate_token should succeed with valid token"
        );
        let claims = result.unwrap();

        assert_eq!(claims.sub, original_claims.sub, "Subject should match");
        assert_eq!(claims.email, original_claims.email, "Email should match");
        assert_eq!(claims.jti, original_claims.jti, "JTI should match");
        assert_eq!(
            claims.token_type, original_claims.token_type,
            "Token type should match"
        );
    }

    #[test]
    fn test_validate_token_invalid_signature() {
        // Arrange
        let config = create_test_config();
        let cache = MockCacheService::new();
        let jwt_service = JwtService::new(&config, cache);
        let user_id = "user-999".to_string();

        // Generate a valid token, then tamper with it
        let (mut token, _) = jwt_service
            .generate_access_token(&user_id, "test@example.com")
            .unwrap();

        // Tamper with the token by changing the signature
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() == 3 {
            token = format!("{}{}invalid", parts[0], parts[1]);
        }

        // Act
        let result = jwt_service.validate_token(&token);

        // Assert
        assert!(
            result.is_err(),
            "validate_token should fail with tampered token"
        );
        match result {
            Err(AppError::JwtError(_)) => {}
            _ => panic!("Expected JwtError for invalid token"),
        }
    }

    #[test]
    fn test_validate_token_malformed() {
        // Arrange
        let config = create_test_config();
        let cache = MockCacheService::new();
        let jwt_service = JwtService::new(&config, cache);
        let invalid_token = "not.a.valid.token.structure";

        // Act
        let result = jwt_service.validate_token(invalid_token);

        // Assert
        assert!(
            result.is_err(),
            "validate_token should fail with malformed token"
        );
        match result {
            Err(AppError::JwtError(_)) => {}
            _ => panic!("Expected JwtError for malformed token"),
        }
    }

    #[tokio::test]
    async fn test_save_refresh_token() {
        // Arrange
        let config = create_test_config();
        let cache = MockCacheService::new();
        let jwt_service = JwtService::new(&config, cache.clone());
        let jti = "token-jti-123";
        let refresh_token = "refresh-token-value";

        // Act
        let result = jwt_service.save_refresh_token(jti, refresh_token).await;

        // Assert
        assert!(result.is_ok(), "save_refresh_token should succeed");
        cache.assert_call("set");

        // Verify the token was actually stored
        let stored: Result<String, _> = cache.get(&format!("refresh_token:{}", jti)).await;
        assert!(stored.is_ok(), "Token should be stored in cache");
        assert_eq!(stored.unwrap(), refresh_token, "Stored token should match");
    }

    #[tokio::test]
    async fn test_save_refresh_token_calls_cache_set() {
        // Arrange
        let config = create_test_config();
        let cache = MockCacheService::new();
        let jwt_service = JwtService::new(&config, cache.clone());
        let jti = "token-jti-456";
        let refresh_token = "another-refresh-token";

        // Act
        let _result = jwt_service.save_refresh_token(jti, refresh_token).await;

        // Assert
        let call_log = cache.get_call_log();
        assert!(
            call_log.contains(&"set".to_string()),
            "Cache set method should be called"
        );
    }

    #[tokio::test]
    async fn test_get_refresh_token_success() {
        // Arrange
        let config = create_test_config();
        let cache = MockCacheService::new();
        let jwt_service = JwtService::new(&config, cache.clone());
        let jti = "token-jti-789";
        let refresh_token = "stored-refresh-token";

        // Save a refresh token first
        jwt_service
            .save_refresh_token(jti, refresh_token)
            .await
            .unwrap();

        // Act
        let result = jwt_service.get_refresh_token(jti).await;

        // Assert
        assert!(result.is_ok(), "get_refresh_token should succeed");
        let token = result.unwrap();
        assert!(token.is_some(), "Token should be found");
        assert_eq!(
            token.unwrap(),
            refresh_token,
            "Retrieved token should match stored token"
        );
    }

    #[tokio::test]
    async fn test_get_refresh_token_not_found() {
        // Arrange
        let config = create_test_config();
        let cache = MockCacheService::new();
        let jwt_service = JwtService::new(&config, cache);
        let non_existent_jti = "does-not-exist";

        // Act
        let result = jwt_service.get_refresh_token(non_existent_jti).await;

        // Assert
        assert!(
            result.is_ok(),
            "get_refresh_token should succeed even if key not found"
        );
        let token = result.unwrap();
        assert!(token.is_none(), "Token should be None when not found");
    }

    #[tokio::test]
    async fn test_get_refresh_token_calls_cache_get() {
        // Arrange
        let config = create_test_config();
        let cache = MockCacheService::new();
        let jwt_service = JwtService::new(&config, cache.clone());
        let jti = "token-jti-call-test";

        // Act
        let _result = jwt_service.get_refresh_token(jti).await;

        // Assert
        cache.assert_call("get");
    }

    #[tokio::test]
    async fn test_delete_refresh_token() {
        // Arrange
        let config = create_test_config();
        let cache = MockCacheService::new();
        let jwt_service = JwtService::new(&config, cache.clone());
        let jti = "token-jti-delete";
        let refresh_token = "token-to-delete";

        // Save a refresh token first
        jwt_service
            .save_refresh_token(jti, refresh_token)
            .await
            .unwrap();

        // Verify it exists
        let exists_before = jwt_service.get_refresh_token(jti).await.unwrap();
        assert!(
            exists_before.is_some(),
            "Token should exist before deletion"
        );

        // Act
        let result = jwt_service.delete_refresh_token(jti).await;

        // Assert
        assert!(result.is_ok(), "delete_refresh_token should succeed");

        // Verify it's deleted
        let exists_after = jwt_service.get_refresh_token(jti).await.unwrap();
        assert!(
            exists_after.is_none(),
            "Token should not exist after deletion"
        );
    }

    #[tokio::test]
    async fn test_delete_refresh_token_calls_cache_delete() {
        // Arrange
        let config = create_test_config();
        let cache = MockCacheService::new();
        let jwt_service = JwtService::new(&config, cache.clone());
        let jti = "token-jti-call-delete";

        // Act
        let _result = jwt_service.delete_refresh_token(jti).await;

        // Assert
        cache.assert_call("delete");
    }

    #[test]
    fn test_get_expiry_hours() {
        // Arrange
        let config = Config {
            server_host: "127.0.0.1".to_string(),
            server_port: 3000,
            database_url: "postgres://localhost/test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            jwt_secret: "test-secret".to_string(),
            jwt_expiry_hours: 24,
            refresh_token_expiry_days: 7,
        };
        let cache = MockCacheService::new();
        let jwt_service = JwtService::new(&config, cache);

        // Act
        let expiry = jwt_service.get_expiry_hours();

        // Assert
        assert_eq!(expiry, 24, "Expiry hours should match config value");
    }

    #[test]
    fn test_refresh_token_cache_key() {
        // Arrange
        let config = create_test_config();
        let cache = MockCacheService::new();
        let jwt_service = JwtService::new(&config, cache);
        let jti = "test-jti-123";

        // Act
        let key = jwt_service.refresh_token_cache_key(jti);

        // Assert
        assert_eq!(
            key,
            format!("refresh_token:{}", jti),
            "Cache key should be formatted correctly"
        );
    }
}
