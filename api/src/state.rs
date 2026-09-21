use std::sync::Arc;

use axum::extract::FromRequestParts;
use tactica_auth::AuthContext;
use tactica_db_model::TacticaStorage;

#[derive(Clone)]
pub struct ApiState {
    storage: Arc<dyn TacticaStorage>,
    auth: Arc<AuthContext>,
}

impl ApiState {
    pub fn new(
        storage: Arc<dyn TacticaStorage>,
        auth: Arc<AuthContext>,
    ) -> Self {
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
