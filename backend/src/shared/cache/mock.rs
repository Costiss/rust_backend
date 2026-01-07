use core::time;
use std::sync::{Arc, Mutex};

use axum::async_trait;

use crate::shared::{CacheError, CacheService};

#[derive(Clone)]
pub struct MockCacheService {
    store: Arc<Mutex<std::collections::HashMap<String, String>>>,
    call_log: Arc<Mutex<Vec<String>>>,
}

impl Default for MockCacheService {
    fn default() -> Self {
        Self::new()
    }
}

impl MockCacheService {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(std::collections::HashMap::new())),
            call_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_call_log(&self) -> Vec<String> {
        self.call_log.lock().unwrap().clone()
    }

    pub fn assert_call(&self, method_name: &str) {
        let log = self.call_log.lock().unwrap();
        assert!(
            log.contains(&method_name.to_string()),
            "Expected call to {} not found in call log: {:?}",
            method_name,
            log
        );
    }
}

#[async_trait]
impl CacheService for MockCacheService {
    async fn set<T: serde::Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        _ttl: Option<time::Duration>,
    ) -> Result<(), CacheError> {
        self.call_log.lock().unwrap().push("set".to_string());
        let serialized = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;
        self.store
            .lock()
            .unwrap()
            .insert(key.to_string(), serialized);
        Ok(())
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T, CacheError> {
        self.call_log.lock().unwrap().push("get".to_string());
        self.store
            .lock()
            .unwrap()
            .get(key)
            .ok_or(CacheError::KeyNotFound)
            .and_then(|value| {
                serde_json::from_str(value)
                    .map_err(|e| CacheError::DeserializationError(e.to_string()))
            })
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        self.call_log.lock().unwrap().push("delete".to_string());
        self.store.lock().unwrap().remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        Ok(self.store.lock().unwrap().contains_key(key))
    }

    async fn keys(&self, _pattern: &str) -> Result<Vec<String>, CacheError> {
        Ok(self.store.lock().unwrap().keys().cloned().collect())
    }

    async fn clear(&self) -> Result<(), CacheError> {
        self.store.lock().unwrap().clear();
        Ok(())
    }

    async fn expire(&self, _key: &str, _ttl: time::Duration) -> Result<(), CacheError> {
        Ok(())
    }

    async fn ttl(&self, key: &str) -> Result<Option<time::Duration>, CacheError> {
        Ok(if self.store.lock().unwrap().contains_key(key) {
            Some(time::Duration::from_secs(3600))
        } else {
            None
        })
    }
}
