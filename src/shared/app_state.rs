use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    infrastructure::{config::Config, database},
    modules::{
        auth::services::jwt_service::JwtService, users::services::user_service::UserService,
    },
};

pub struct AppState {
    pub config: Config,
    pub pool: PgPool,

    pub jwt_service: JwtService,

    pub user_service: UserService,
}

impl AppState {
    pub async fn initialize() -> Arc<Self> {
        dotenv::dotenv().ok();
        let config = Config::from_env().expect("Failed to load configuration");

        let pool = database::create_pool(&database::DatabaseConfig::new(
            config.database_url.clone(),
            5,
        ))
        .await
        .expect("Failed to create database pool");
        tracing::info!("Database connection established");

        let jwt_service =
            JwtService::new(config.jwt_secret.clone(), config.jwt_expiry_hours, &pool);

        let user_service = UserService::new(&pool);

        Arc::new(AppState {
            pool,
            jwt_service,
            config,
            user_service,
        })
    }
}
