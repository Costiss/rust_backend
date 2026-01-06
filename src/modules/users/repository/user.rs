use crate::{
    infrastructure::database::Database, modules::users::user_model::User,
    shared::objects::email::Email, AppResult,
};
use sqlx::Row;
use ulid::Ulid;

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn insert_user(
        &self,
        id: &str,
        email: &str,
        password_hash: &str,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()>;

    async fn get_user_by_email(&self, email: &str) -> AppResult<Option<User>>;
}

#[async_trait::async_trait]
impl UserRepository for Database {
    async fn insert_user(
        &self,
        id: &str,
        email: &str,
        password_hash: &str,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(id)
        .bind(email)
        .bind(password_hash)
        .bind(created_at)
        .bind(updated_at)
        .execute(self)
        .await?;

        Ok(())
    }

    async fn get_user_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let row = sqlx::query(
            "SELECT id, email, password_hash, created_at, updated_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(self)
        .await?;

        if let Some(row) = row {
            let id_str: String = row.get("id");
            let id = Ulid::from_string(&id_str).unwrap();
            let email_str: String = row.get("email");
            let password_hash: String = row.get("password_hash");
            let created_at = row.get("created_at");
            let updated_at = row.get("updated_at");

            let email_obj = Email::new(&email_str).unwrap();

            Ok(Some(User::with_id(
                id,
                email_obj,
                password_hash,
                created_at,
                updated_at,
            )))
        } else {
            Ok(None)
        }
    }
}
