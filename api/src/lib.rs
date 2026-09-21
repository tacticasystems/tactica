use std::time::Duration;

use axum::{Router, extract::DefaultBodyLimit, http::StatusCode};
use tower_http::{limit::RequestBodyLimitLayer, timeout::TimeoutLayer};

use crate::state::ApiState;

mod error;
mod routes;
pub mod state;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_BODY_BYTES: usize = 1024 * 1024;

pub fn router(state: ApiState) -> Router {
    let router = routes(state);

    router
}

fn routes(state: ApiState) -> Router {
    let ordinary = Router::new()
        .merge(routes::auth::router().with_state(state.clone()))
        .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
        .with_state(state);

    ordinary
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ))
        .layer(DefaultBodyLimit::disable())
}
