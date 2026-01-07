pub mod mock;
pub mod redis_cache;

pub use redis_cache::RedisCacheService;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

/// Result type for cache operations
pub type CacheResult<T> = Result<T, CacheError>;

/// Errors that can occur during cache operations
#[derive(Debug, Clone)]
pub enum CacheError {
    /// Key was not found in cache
    KeyNotFound,
    /// Serialization error occurred
    SerializationError(String),
    /// Deserialization error occurred
    DeserializationError(String),
    /// Redis connection or operation error
    RedisError(String),
    /// Internal error occurred
    InternalError(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::KeyNotFound => write!(f, "Key not found in cache"),
            CacheError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            CacheError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            CacheError::RedisError(msg) => write!(f, "Redis error: {}", msg),
            CacheError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for CacheError {}

/// Cache service trait for cache operations
#[async_trait]
pub trait CacheService: Send + Sync + Clone {
    /// Set a value in the cache with optional expiration
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<()>;

    /// Get a value from the cache
    async fn get<T: DeserializeOwned>(&self, key: &str) -> CacheResult<T>;

    /// Delete a key from the cache
    async fn delete(&self, key: &str) -> CacheResult<()>;

    /// Check if a key exists in the cache
    async fn exists(&self, key: &str) -> CacheResult<bool>;

    /// Get all keys matching a pattern
    async fn keys(&self, pattern: &str) -> CacheResult<Vec<String>>;

    /// Clear all keys from the cache
    async fn clear(&self) -> CacheResult<()>;

    /// Set expiration on an existing key
    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<()>;

    /// Get the remaining time to live of a key
    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_error_display() {
        let err = CacheError::KeyNotFound;
        assert_eq!(err.to_string(), "Key not found in cache");

        let err = CacheError::SerializationError("test error".to_string());
        assert!(err.to_string().contains("Serialization error"));

        let err = CacheError::RedisError("connection failed".to_string());
        assert!(err.to_string().contains("Redis error"));
    }
}
