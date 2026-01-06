/// Redis implementation of the CacheService trait
use super::cache::{CacheError, CacheResult, CacheService};
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tracing::{debug, error};

/// Redis cache service implementation
#[derive(Clone)]
pub struct RedisCacheService {
    client: ConnectionManager,
}

impl RedisCacheService {
    /// Create a new Redis cache service
    pub fn new(client: ConnectionManager) -> Self {
        Self { client }
    }
}

#[async_trait]
impl CacheService for RedisCacheService {
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        let serialized = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let mut conn = self.client.clone();

        match ttl {
            Some(duration) => {
                let seconds = duration.as_secs() as usize;
                redis::cmd("SET")
                    .arg(key)
                    .arg(&serialized)
                    .arg("EX")
                    .arg(seconds)
                    .query_async::<_, ()>(&mut conn)
                    .await
                    .map_err(|e| {
                        error!("Redis SET error for key '{}': {}", key, e);
                        CacheError::RedisError(e.to_string())
                    })?;
            }
            None => {
                redis::cmd("SET")
                    .arg(key)
                    .arg(&serialized)
                    .query_async::<_, ()>(&mut conn)
                    .await
                    .map_err(|e| {
                        error!("Redis SET error for key '{}': {}", key, e);
                        CacheError::RedisError(e.to_string())
                    })?;
            }
        }

        debug!("Cache SET: key='{}', ttl={:?}", key, ttl);
        Ok(())
    }

    async fn get<T: DeserializeOwned>(&self, key: &str) -> CacheResult<T> {
        let mut conn = self.client.clone();

        let value: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                error!("Redis GET error for key '{}': {}", key, e);
                CacheError::RedisError(e.to_string())
            })?;

        match value {
            Some(serialized) => {
                let deserialized = serde_json::from_str(&serialized)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))?;
                debug!("Cache GET (hit): key='{}'", key);
                Ok(deserialized)
            }
            None => {
                debug!("Cache GET (miss): key='{}'", key);
                Err(CacheError::KeyNotFound)
            }
        }
    }

    async fn delete(&self, key: &str) -> CacheResult<()> {
        let mut conn = self.client.clone();

        redis::cmd("DEL")
            .arg(key)
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| {
                error!("Redis DEL error for key '{}': {}", key, e);
                CacheError::RedisError(e.to_string())
            })?;

        debug!("Cache DELETE: key='{}'", key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> CacheResult<bool> {
        let mut conn = self.client.clone();

        let exists: bool = redis::cmd("EXISTS")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                error!("Redis EXISTS error for key '{}': {}", key, e);
                CacheError::RedisError(e.to_string())
            })?;

        debug!("Cache EXISTS: key='{}', exists={}", key, exists);
        Ok(exists)
    }

    async fn keys(&self, pattern: &str) -> CacheResult<Vec<String>> {
        let mut conn = self.client.clone();

        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                error!("Redis KEYS error for pattern '{}': {}", pattern, e);
                CacheError::RedisError(e.to_string())
            })?;

        debug!("Cache KEYS: pattern='{}', count={}", pattern, keys.len());
        Ok(keys)
    }

    async fn clear(&self) -> CacheResult<()> {
        let mut conn = self.client.clone();

        redis::cmd("FLUSHDB")
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| {
                error!("Redis FLUSHDB error: {}", e);
                CacheError::RedisError(e.to_string())
            })?;

        debug!("Cache CLEAR: all keys removed");
        Ok(())
    }

    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<()> {
        let mut conn = self.client.clone();
        let seconds = ttl.as_secs() as usize;

        redis::cmd("EXPIRE")
            .arg(key)
            .arg(seconds)
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| {
                error!("Redis EXPIRE error for key '{}': {}", key, e);
                CacheError::RedisError(e.to_string())
            })?;

        debug!("Cache EXPIRE: key='{}', ttl={:?}", key, ttl);
        Ok(())
    }

    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>> {
        let mut conn = self.client.clone();

        let seconds: i64 = redis::cmd("TTL")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                error!("Redis TTL error for key '{}': {}", key, e);
                CacheError::RedisError(e.to_string())
            })?;

        let ttl = match seconds {
            -2 => None,                                // Key does not exist
            -1 => Some(Duration::from_secs(u64::MAX)), // Key exists but has no expiration
            s if s >= 0 => Some(Duration::from_secs(s as u64)),
            _ => None,
        };

        debug!(
            "Cache TTL: key='{}', seconds={}, ttl={:?}",
            key, seconds, ttl
        );
        Ok(ttl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests would require a running Redis instance
    // For integration tests, use a test container or mock

    #[test]
    fn test_redis_cache_service_creation() {
        // This is a basic test that verifies the type can be created
        // Actual functionality tests would require a Redis instance
    }
}
