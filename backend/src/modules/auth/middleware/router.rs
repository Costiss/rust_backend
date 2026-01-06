use axum::{middleware, Router};

use crate::{modules::auth::middleware::jwt_validation_middleware, shared::app_state::AppContext};

pub fn public_router() -> Router<AppContext> {
    Router::new()
}
pub fn protected_route(router: Router<AppContext>, ctx: &AppContext) -> Router<AppContext> {
    router.layer(middleware::from_fn_with_state(
        ctx.clone(),
        jwt_validation_middleware,
    ))
}
