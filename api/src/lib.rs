use std::time::Duration;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tower_http::{limit::RequestBodyLimitLayer, timeout::TimeoutLayer};

use crate::{error::Error, state::ApiState};

mod error;
mod routes;
pub mod state;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_BODY_BYTES: usize = 1024 * 1024;

pub fn router(state: ApiState) -> Router {
    routes(state)
}

fn routes(state: ApiState) -> Router {
    let ordinary = Router::new()
        .merge(routes::auth::router().with_state(state.clone()))
        .merge(routes::users::router().with_state(state.clone()))
        .merge(routes::units::router().with_state(state.clone()))
        .merge(routes::unit_settings::router().with_state(state.clone()))
        .merge(routes::ranks::router().with_state(state.clone()))
        .merge(routes::roles::router().with_state(state.clone()))
        .merge(routes::members::router().with_state(state.clone()))
        .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
        .fallback(async || -> Response { Error::NotFound.into_response() })
        .with_state(state);

    ordinary
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ))
        .layer(DefaultBodyLimit::disable())
}
