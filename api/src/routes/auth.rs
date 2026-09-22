use anyhow::anyhow;
use axum::routing::get;
use axum::{Json, Router, response::IntoResponse, routing::post};
use tactica_api_types::v1;
use tactica_db_model::UserStore;

use crate::error::{Error, Result};
use crate::state::{ApiState, AuthCtx, Principal, Storage};

pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/api/v1/auth/me", get(me))
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
    let user = UserStore::get_by_username(stg.as_ref(), &body.username)
        .await?
        .ok_or_else(|| Error::Unauthorized("invalid username or password".to_string()))?;

    let password_hash = user
        .password_hash
        .ok_or_else(|| Error::Unauthorized("invalid username or password".to_string()))?;

    if !auth_ctx
        .hash_password(&body.password)
        .await
        .map_err(|err| {
            tracing::error!(?err, "Failed to hash password");
            Error::Other(anyhow!("failed to hash password: {err}"))
        })?
        .eq(&password_hash)
    {
        return Err(Error::Unauthorized(
            "invalid username or password".to_string(),
        ));
    }

    let token = auth_ctx
        .jwt()
        .generate_jwt_for_user(user.id)
        .map_err(|err| {
            tracing::error!(?err, "Failed to generate JWT");
            Error::Other(anyhow!("failed to generate JWT: {err}"))
        })?;

    Ok(Json(v1::auth::LoginResponse {
        token_type: "Bearer".to_string(),
        access_token: token,
        refresh_token: String::new(),
        expires_in: 3600,
    }))
}

async fn register() -> &'static str {
    "register"
}

async fn me(Storage(stg): Storage, Principal(principal): Principal) -> Result<impl IntoResponse> {
    match principal {
        tactica_auth::principal::Principal::User(user_id) => {
            let user = UserStore::get(stg.as_ref(), user_id).await?;
            if let Some(user) = user {
                Ok(Json(v1::auth::MeResponse {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    display_name: user.display_name,
                    icon_url: user.icon_url,
                    banner_url: user.banner_url,
                    biography: user.biography,
                    is_active: user.is_active,
                    is_superuser: user.is_superuser,
                    created_at: user.created_at,
                    updated_at: user.updated_at,
                }))
            } else {
                Err(Error::Unauthorized("User not found".to_string()))
            }
        } // _ => Err(Error::Unauthorized("Invalid principal type".to_string())),
    }
}
