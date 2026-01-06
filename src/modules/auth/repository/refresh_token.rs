use chrono::Utc;

use crate::{infrastructure::database::Database, AppResult};

#[async_trait::async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    async fn save_refresh_token(
        &self,
        id: &str,
        user_id: &str,
        token_hash: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()>;
}

/// Default implementation using the database pool
#[async_trait::async_trait]
impl RefreshTokenRepository for Database {
    async fn save_refresh_token(
        &self,
        id: &str,
        user_id: &str,
        token_hash: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()> {
        let now = Utc::now();
        sqlx::query(
            "INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, created_at) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(id)
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .bind(now)
        .execute(self)
        .await?;

        Ok(())
    }
}
