use axum::Router;

use crate::state::ApiState;

pub fn router() -> Router<ApiState> {
    Router::new()
}
