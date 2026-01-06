/// Application configuration
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,

    pub database_url: String,

    pub redis_url: String,

    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env::<Config>()
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_addr() {
        let config = Config {
            server_host: "127.0.0.1".to_string(),
            server_port: 3000,

            database_url: "postgres://localhost/db".to_string(),

            redis_url: "redis://localhost:6379".to_string(),

            jwt_secret: "secret".to_string(),
            jwt_expiry_hours: 24,
        };
        assert_eq!(config.server_addr(), "127.0.0.1:3000");
    }
}
