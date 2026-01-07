use crate::modules::auth::dto::{AuthResponse, RefreshTokenRequest, SignInRequest, SignUpRequest};
use utoipa::{Modify, OpenApi};

/// OpenAPI spec for the Prontua backend API
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Prontua API",
        description = "Authentication and user management API for Prontua",
        version = "0.1.0",
    ),
    servers(
        (url = "http://127.0.0.1:3000", description = "Local loopback server"),
    ),
    paths(
        crate::modules::auth::auth_handler::sign_up,
        crate::modules::auth::auth_handler::sign_in,
        crate::modules::auth::auth_handler::refresh,
    ),
    components(
        schemas(SignUpRequest, SignInRequest, RefreshTokenRequest, AuthResponse)
    ),
    tags(
        (name = "Authentication", description = "User authentication endpoints")
    ),
    modifiers(&SecurityAddon),
)]
pub struct ApiDoc;

/// Modifier to add security information to OpenAPI spec
pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("JWT token obtained from sign-up or sign-in"))
                        .build(),
                ),
            );
        }
    }
}
