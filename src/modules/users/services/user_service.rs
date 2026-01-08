use crate::modules::users::objects::birthdate::BirthDate;
use crate::modules::users::repository::user::UserRepository;
use crate::modules::users::user_model::User;
use crate::shared::objects::email::Email;
use crate::{AppError, AppResult};
use chrono::Utc;
use ulid::Ulid;

pub struct UserService {
    pool: sqlx::PgPool,
}

impl UserService {
    pub fn new(pool: &sqlx::PgPool) -> Self {
        UserService { pool: pool.clone() }
    }

    pub async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        birthdate: BirthDate,
    ) -> AppResult<User> {
        let id = Ulid::new();
        let now = Utc::now();

        let email_obj = Email::new(email).map_err(AppError::validation)?;
        self.pool
            .insert_user(
                &id.to_string(),
                email,
                password_hash,
                birthdate.as_date(),
                now,
                now,
            )
            .await?;

        Ok(User::with_id(
            id,
            email_obj,
            password_hash.to_string(),
            now,
            now,
        ))
    }

    pub async fn get_user_by_email(&self, email: &str) -> AppResult<Option<User>> {
        self.pool.get_user_by_email(email).await
    }

    pub async fn get_user_by_id(&self, id: &str) -> AppResult<Option<User>> {
        self.pool.get_user_by_id(id).await
    }
}
