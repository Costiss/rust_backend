/// Main entry point for Prontua Backend
use axum::{
    routing::post,
    Router,
};
use prontua_backend::features::auth::handlers::{self, AuthState};
use prontua_backend::features::auth::services::JwtService;
use prontua_backend::infrastructure::config::Config;
use prontua_backend::infrastructure::database;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Load configuration
    dotenv::dotenv().ok();
    let config = Config::from_env().expect("Failed to load configuration");

    tracing::info!("Starting Prontua Backend");
    tracing::info!("Server will listen on {}", config.server_addr());

    // Initialize database connection pool
    let pool = database::create_pool(&database::DatabaseConfig::new(
        config.database_url.clone(),
        5,
    ))
    .await
    .expect("Failed to create database pool");

    // Run migrations (for now, you need to create them manually with sqlx migrate add)
    // sqlx::migrate!("./migrations")
    //     .run(&pool)
    //     .await
    //     .expect("Failed to run migrations");

    tracing::info!("Database connection established");

    // Initialize JWT service
    let jwt_service = JwtService::new(config.jwt_secret.clone(), config.jwt_expiry_hours);

    // Create application state
    let auth_state = Arc::new(AuthState {
        pool: pool.clone(),
        jwt_service,
    });

    // Build router
    let app = Router::new()
        .route("/api/auth/sign-up", post(handlers::sign_up))
        .route("/api/auth/sign-in", post(handlers::sign_in))
        .route("/api/auth/refresh", post(handlers::refresh))
        .with_state(auth_state);

    // Run server
    let listener = tokio::net::TcpListener::bind(&config.server_addr())
        .await
        .expect("Failed to bind to address");

    tracing::info!("Server running on {}", config.server_addr());

    axum::serve(listener, app)
        .await
        .expect("Server error");

    Ok(())
}
