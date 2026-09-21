use axum::{Json, Router, response::IntoResponse, routing::post};
use tactica_api_types::v1;
use tactica_db_model::UserStore;
use anyhow::anyhow;

use crate::error::{Error, Result};
use crate::state::{ApiState, AuthCtx, Storage};

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
    AuthCtx(auth_ctx): AuthCtx,
    Json(body): Json<v1::auth::LoginRequest>,
) -> Result<impl IntoResponse> {
    let user = UserStore::get_by_username(stg.as_ref(), &body.username).await?;

    if user.is_none() {
        return Err(Error::Unauthorized(
            "invalid username or password".to_string(),
        ));
    }

    let user = user.unwrap();

    if user.password_hash.is_none() {
        println!("User {} has no password hash", user.username);
        return Err(Error::Unauthorized(
            "invalid username or password".to_string(),
        ));
    }

    let password_hash = user.password_hash.unwrap();

    if !auth_ctx
        .hash_password(&body.password)
        .await
        .map_err(|e| {
            println!("Failed to hash password: {}", e);
            Error::Other(anyhow!("failed to hash password: {}", e))
        })?
        .eq(&password_hash)
    {
        println!("Password hash does not match for user {}", user.username);
        return Err(Error::Unauthorized(
            "invalid username or password".to_string(),
        ));
    }

    let token = auth_ctx.jwt().generate_jwt_for_user(user.id)
        .map_err(|e| {
            println!("Failed to generate JWT: {}", e);
            Error::Other(anyhow!("failed to generate JWT: {}", e))
        })?;

    Ok(Json(v1::auth::LoginResponse {
        token_type: "Bearer".to_string(),
        access_token: token,
        refresh_token: "".to_string(),
        expires_in: 3600,
    }))
}

async fn register() -> &'static str {
    "register"
}
