/// Database configuration and connection pool
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::str::FromStr;

pub type Database = sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl DatabaseConfig {
    pub fn new(url: String, max_connections: u32) -> Self {
        Self {
            url,
            max_connections,
        }
    }
}

/// Initialize database connection pool
pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    let connect_options = PgConnectOptions::from_str(&config.url)?;

    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect_with(connect_options)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config() {
        let config = DatabaseConfig::new("postgres://user:pass@localhost/db".to_string(), 5);
        assert_eq!(config.max_connections, 5);
    }
}
