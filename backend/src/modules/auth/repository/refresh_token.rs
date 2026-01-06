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

    async fn get_latest_refresh_token_by_user_id(
        &self,
        user_id: &str,
    ) -> AppResult<Option<(String,)>>;
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

    async fn get_latest_refresh_token_by_user_id(
        &self,
        user_id: &str,
    ) -> AppResult<Option<(String,)>> {
        Ok(sqlx::query_as::<_, (String,)>(
            "SELECT token_hash FROM refresh_tokens WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(self)
        .await?)
    }
}
