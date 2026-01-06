/// Main entry point for Prontua Backend
use axum::Router;
use prontua_backend::modules::auth::auth_handler::auth_routes;
use prontua_backend::shared::app_state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app_state = AppState::initialize().await;

    let server_addr = app_state.config.server_addr();

    tracing::info!("Starting Prontua Backend");
    tracing::info!("Server will listen on {}", server_addr);

    // Run migrations (for now, you need to create them manually with sqlx migrate add)
    // sqlx::migrate!("./migrations")
    //     .run(&pool)
    //     .await
    //     .expect("Failed to run migrations");

    // Build router
    let app = Router::new().merge(auth_routes()).with_state(app_state);

    // Run server
    let listener = tokio::net::TcpListener::bind(&server_addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!("Server running on {}", server_addr);

    axum::serve(listener, app).await.expect("Server error");

    Ok(())
}
