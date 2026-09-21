use axum::{Json, Router, response::IntoResponse, routing::post};
use tactica_api_types::v1;
use tactica_db_model::UserStore;

use crate::state::{ApiState, Storage};

pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/register", post(register))
}

/// Login a user and return a JWT token if successful.
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = v1::auth::LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = v1::auth::LoginResponse),
        (status = 401, description = "Invalid credentials"),
    ),
)]
async fn login(
    Storage(stg): Storage,
    Json(body): Json<v1::auth::LoginRequest>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    let user = UserStore::get_by_username(
        stg.as_ref(),
        &body.username,
    )
        .await;

    if user.is_err() {
        return Err((axum::http::StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    if user.unwrap().is_none() {
        return Err((axum::http::StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    Ok(Json(v1::auth::LoginResponse {
        token_type: "Bearer".to_string(),
        access_token: "".to_string(),
        refresh_token: "".to_string(),
        expires_in: 3600,
    }))
}

async fn register() -> &'static str {
    "register"
}
