use axum::{Router, routing::get};

use crate::state::ApiState;

pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/livez", get(livez))
}

async fn healthz() -> &'static str {
    "OK"
}

async fn readyz() -> &'static str {
    "OK"
}

async fn livez() -> &'static str {
    "OK"
}
