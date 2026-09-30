use std::sync::Arc;

use axum::extract::FromRequestParts;
use tactica_auth::AuthContext;
use tactica_db_model::TacticaStorage;

use crate::error::Error;

#[derive(Clone)]
pub struct ApiState {
    storage: Arc<dyn TacticaStorage>,
    auth: Arc<AuthContext>,
}

impl ApiState {
    pub fn new(storage: Arc<dyn TacticaStorage>, auth: Arc<AuthContext>) -> Self {
        Self { storage, auth }
    }

    pub(crate) fn storage(&self) -> Arc<dyn TacticaStorage> {
        self.storage.clone()
    }

    pub(crate) fn auth(&self) -> Arc<AuthContext> {
        self.auth.clone()
    }
}

#[derive(Clone)]
pub struct Storage(pub Arc<dyn TacticaStorage>);

impl FromRequestParts<ApiState> for Storage {
    type Rejection = ();

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &ApiState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(state.storage()))
    }
}

#[derive(Clone)]
pub struct AuthCtx(pub Arc<AuthContext>);

impl FromRequestParts<ApiState> for AuthCtx {
    type Rejection = ();

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &ApiState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(state.auth()))
    }
}

#[derive(Clone)]
pub struct Principal(pub tactica_auth::principal::Principal);

impl FromRequestParts<ApiState> for Principal {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &ApiState,
    ) -> Result<Self, Self::Rejection> {
        let bearer_token = parts
            .headers
            .get("Authorization")
            .and_then(|header| header.to_str().ok())
            .and_then(|header| {
                let parts = header.split_once(' ');
                if let Some((scheme, token)) = parts {
                    if scheme == "Bearer" {
                        Some(token.to_string())
                    } else {
                        tracing::debug!(?scheme, "Invalid authorization scheme");
                        None
                    }
                } else {
                    tracing::debug!("Invalid authorization header format");
                    None
                }
            })
            .ok_or_else(|| {
                tracing::debug!("No authorization header found");
                Error::Unauthorized("No bearer token found in request".to_string())
            })?;

        let claims = state
            .auth
            .jwt()
            .validate_jwt(&bearer_token)
            .map_err(|err| {
                tracing::debug!(?err, "Failed to validate JWT");
                Error::Unauthorized("Invalid or expired token".to_string())
            })?;

        Ok(Self(
            tactica_auth::principal::Principal::from_jwt_claims(&claims).map_err(|err| {
                tracing::debug!(?err, "Failed to extract principal from claims");
                Error::Unauthorized("Invalid token claims".to_string())
            })?,
        ))
    }
}
