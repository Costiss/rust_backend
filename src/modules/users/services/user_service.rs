use sqlx::Row;
use ulid::Ulid;

use crate::domain::value_objects::Email;
use crate::modules::users::user_model::User;
use crate::{AppError, AppResult};
use chrono::Utc;

pub struct UserService {
    pool: sqlx::PgPool,
}

impl UserService {
    pub fn new(pool: &sqlx::PgPool) -> Self {
        UserService { pool: pool.clone() }
    }

    pub async fn create_user(&self, email: &str, password_hash: &str) -> AppResult<User> {
        let id = Ulid::new();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(id.to_string())
        .bind(email)
        .bind(password_hash)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let email_obj = Email::new(email).map_err(AppError::ValidationError)?;

        Ok(User::with_id(
            id,
            email_obj,
            password_hash.to_string(),
            now,
            now,
        ))
    }

    pub async fn get_user_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let row = sqlx::query(
            "SELECT id, email, password_hash, created_at, updated_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
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
