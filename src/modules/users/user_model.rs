use crate::shared::{kernel::traits::HasId, objects::email::Email};
use chrono::{DateTime, Utc};
use ulid::Ulid;

#[derive(Debug, Clone)]
pub struct User {
    id: Ulid,
    email: Email,
    password_hash: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    /// Create a new user with a hashed password
    pub fn new(email: Email, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            email,
            password_hash,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a user with specific ID (for database reconstructions)
    pub fn with_id(
        id: Ulid,
        email: Email,
        password_hash: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            email,
            password_hash,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> &Ulid {
        &self.id
    }

    pub fn id_str(&self) -> String {
        self.id.to_string()
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Verify that a plain text password matches the stored hash
    pub fn verify_password(&self, plain_password: &str) -> Result<bool, bcrypt::BcryptError> {
        bcrypt::verify(plain_password, &self.password_hash)
    }

    /// Update the password hash (called after rehashing)
    pub fn set_password_hash(&mut self, new_hash: String) {
        self.password_hash = new_hash;
        self.updated_at = Utc::now();
    }
}

impl HasId for User {
    type Id = Ulid;

    fn id(&self) -> Self::Id {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let email = Email::new("test@example.com").unwrap();
        let user = User::new(email.clone(), "hashed_password".to_string());

        assert_eq!(user.email(), &email);
        assert_eq!(user.password_hash(), "hashed_password");
        assert!(user.created_at <= Utc::now());
    }

    #[test]
    fn test_user_password_hash_update() {
        let email = Email::new("test@example.com").unwrap();
        let mut user = User::new(email, "old_hash".to_string());
        let old_updated_at = user.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(100));

        user.set_password_hash("new_hash".to_string());

        assert_eq!(user.password_hash(), "new_hash");
        assert!(user.updated_at > old_updated_at);
    }

    #[test]
    fn test_user_has_id() {
        let email = Email::new("test@example.com").unwrap();
        let user = User::new(email, "hashed_password".to_string());
        assert_eq!(user.id(), user.id);
    }
}
