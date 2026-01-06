use axum::{middleware, Router};

use crate::{modules::auth::middleware::jwt_validation_middleware, shared::app_state::AppContext};

pub fn public_router() -> Router<AppContext> {
    Router::new()
}
pub fn protected_route(app_state: AppContext) -> Router<AppContext> {
    Router::new().layer(middleware::from_fn_with_state(
        app_state,
        jwt_validation_middleware,
    ))
}
