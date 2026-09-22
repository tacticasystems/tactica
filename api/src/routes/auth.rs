use anyhow::anyhow;
use axum::http::{StatusCode, header};
use axum::routing::get;
use axum::{Json, Router, response::IntoResponse, routing::post};
use tactica_api_types::v1;
use tactica_auth::refresh::{RefreshError, TokenPair};
use tactica_db_model::{NewUser, UserStore};
use tactica_uuid_kinds::UserId;

use crate::error::{Error, Result};
use crate::state::{ApiState, AuthCtx, Principal, Storage};

pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/api/v1/auth/me", get(me))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/register", post(register))
        .route("/api/v1/auth/refresh", post(refresh))
        .route("/api/v1/auth/logout", post(logout))
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
        .filter(|user| user.is_active)
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

    let tokens = auth_ctx
        .issue_tokens(user.id)
        .await
        .map_err(refresh_error)?;

    Ok(token_response(tokens))
}

fn refresh_error(error: RefreshError) -> Error {
    match error {
        RefreshError::InvalidToken => {
            Error::Unauthorized("Invalid or expired refresh token".to_owned())
        }
        RefreshError::Store(error) => Error::Store(error),
        other => Error::Other(anyhow::Error::new(other)),
    }
}

fn token_response(tokens: TokenPair) -> impl IntoResponse {
    (
        [
            (header::CACHE_CONTROL, "no-store"),
            (header::PRAGMA, "no-cache"),
        ],
        Json(v1::auth::LoginResponse {
            token_type: "Bearer".to_owned(),
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in: tokens.expires_in,
        }),
    )
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = v1::auth::RefreshRequest,
    responses(
        (status = 200, description = "Replacement access and refresh tokens", body = v1::auth::LoginResponse),
        (status = 401, description = "Invalid, expired, revoked, or reused refresh token"),
    ),
)]
async fn refresh(
    AuthCtx(auth): AuthCtx,
    Json(body): Json<v1::auth::RefreshRequest>,
) -> Result<impl IntoResponse> {
    let tokens = auth
        .refresh_tokens(&body.refresh_token)
        .await
        .map_err(refresh_error)?;
    Ok(token_response(tokens))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    request_body = v1::auth::LogoutRequest,
    responses((status = 204, description = "Refresh session revoked")),
)]
async fn logout(
    AuthCtx(auth): AuthCtx,
    Json(body): Json<v1::auth::LogoutRequest>,
) -> Result<StatusCode> {
    auth.revoke_refresh_token(&body.refresh_token)
        .await
        .map_err(refresh_error)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Register a new user.
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = v1::auth::RegisterRequest,
    responses(
        (status = 200, description = "Registration successful", body = v1::auth::RegisterResponse),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Username or email already exists"),
    ),
)]
async fn register(
    Storage(stg): Storage,
    AuthCtx(auth_ctx): AuthCtx,
    Json(body): Json<v1::auth::RegisterRequest>,
) -> Result<impl IntoResponse> {
    if body.password != body.password_confirm {
        return Err(Error::Validation(
            "Password and password confirmation do not match".to_string(),
        ));
    }

    let password_hash = auth_ctx
        .hash_password(&body.password)
        .await
        .map_err(|err| {
            tracing::error!(?err, "Failed to hash password");
            Error::Other(anyhow!("failed to hash password: {err}"))
        })?;

    let new_user = NewUser {
        id: UserId::new(),
        username: body.username.clone(),
        email: body.email.clone(),
        password_hash: Some(password_hash.clone()),
        display_name: body.display_name.clone(),
        is_active: true,
        is_superuser: false,
        icon_url: None,
        banner_url: None,
        biography: None,
        totp_secret: None,
    };

    let user = UserStore::create(stg.as_ref(), new_user)
        .await
        .inspect_err(|err| {
            tracing::error!(?err, "Failed to create user");
        })?;

    Ok(Json(v1::auth::RegisterResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        display_name: user.display_name,
        is_active: user.is_active,
        created_at: user.created_at,
    }))
}

/// Get the currently authenticated user's information.
#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    responses(
        (status = 200, description = "User information retrieved successfully", body = v1::auth::MeResponse),
        (status = 401, description = "Unauthorized"),
    ),
)]
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
        }

        _ => Err(Error::Unauthorized("Invalid principal type".to_string())),
    }
}
