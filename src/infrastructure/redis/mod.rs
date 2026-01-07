use crate::infrastructure::config::Config;
use redis::{aio::ConnectionManager, Client};
use tracing::{info, warn};

/// Create a Redis connection manager pool
pub async fn create_redis_client(config: &Config) -> Result<ConnectionManager, anyhow::Error> {
    info!("Connecting to Redis at {}", config.redis_url);

    let client = Client::open(config.redis_url.as_str())?;

    let connection_manager = ConnectionManager::new(client).await?;

    verify_redis_connection(&connection_manager).await?;

    info!("Redis connection established successfully");

    Ok(connection_manager)
}

/// Verify Redis connection
pub async fn verify_redis_connection(
    connection_manager: &ConnectionManager,
) -> Result<(), anyhow::Error> {
    let mut conn = connection_manager.clone();
    let pong: String = redis::cmd("PING").query_async(&mut conn).await?;

    if pong == "PONG" {
        info!("Redis PING successful");
        Ok(())
    } else {
        warn!("Unexpected PING response: {}", pong);
        Err(anyhow::anyhow!("Unexpected PING response from Redis"))
    }
}
