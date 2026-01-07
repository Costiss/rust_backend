/// Main entry point for Prontua Backend
use axum::Router;
use prontua_backend::modules::auth::auth_handler::auth_routes;
use prontua_backend::shared::app_state::AppState;
use prontua_backend::shared::openapi::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
    tracing::info!("OpenAPI Spec: http://{}/api-docs/openapi.json", server_addr);
    tracing::info!("Swagger UI: http://{}/swagger-ui/", server_addr);

    // Build router with OpenAPI documentation and Swagger UI
    let app = Router::new()
        .merge(auth_routes(&app_state))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(app_state);

    // Run server
    let listener = tokio::net::TcpListener::bind(&server_addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!("Server running on {}", server_addr);

    axum::serve(listener, app).await.expect("Server error");

    Ok(())
}
