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
        Ok(Storage(state.storage()))
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
        Ok(AuthCtx(state.auth()))
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
                        return Some(token.to_string());
                    } else {
                        println!("Invalid authorization scheme: {}", scheme);
                        return None;
                    }
                } else {
                    println!("Invalid authorization header format");
                    return None;
                }
            });

        if bearer_token.is_none() {
            println!("No bearer token found in request");
            return Err(Error::Unauthorized(
                "No bearer token found in request".to_string(),
            ));
        }
        let bearer_token = bearer_token.unwrap();

        let claims = state.auth.jwt().validate_jwt(&bearer_token).map_err(|e| {
            println!("Failed to validate JWT: {}", e);
            Error::Unauthorized("Invalid or expired token".to_string())
        })?;

        Ok(Principal(
            tactica_auth::principal::Principal::from_jwt_claims(&claims)
                .await
                .map_err(|e| {
                    println!("Failed to extract principal from claims: {}", e);
                    Error::Unauthorized("Invalid token claims".to_string())
                })?,
        ))
    }
}
