use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    infrastructure::{config::Config, database, redis},
    modules::{
        auth::services::jwt_service::JwtService, users::services::user_service::UserService,
    },
    shared::RedisCacheService,
};

/**
    Atomic reference counted application state shared across the application.
*/
pub type AppContext = Arc<AppState>;

pub struct AppState {
    pub config: Config,
    pub pool: PgPool,

    pub jwt_service: JwtService,

    pub user_service: UserService,

    pub cache_service: RedisCacheService,
}

impl AppState {
    pub async fn initialize() -> AppContext {
        dotenv::dotenv().ok();
        let config = Config::from_env().expect("Failed to load configuration");

        let pool = database::create_pool(&database::DatabaseConfig::new(
            config.database_url.clone(),
            5,
        ))
        .await
        .expect("Failed to create database pool");
        tracing::info!("Database connection established");

        let redis_client = redis::create_redis_client(&config)
            .await
            .expect("Failed to create Redis client");
        let cache = RedisCacheService::new(redis_client.clone());

        let jwt_service = JwtService::new(&config, &cache);

        let user_service = UserService::new(&pool);

        let cache_service = RedisCacheService::new(redis_client);

        Arc::new(AppState {
            pool,
            jwt_service,
            config,
            user_service,
            cache_service,
        })
    }
}
