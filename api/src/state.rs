use std::sync::Arc;

use axum::extract::FromRequestParts;
use tactica_db_model::TacticaStorage;

#[derive(Clone)]
pub struct ApiState {
    storage: Arc<dyn TacticaStorage>,
}

impl ApiState {
    pub fn new(storage: Arc<dyn TacticaStorage>) -> Self {
        Self { storage }
    }

    pub(crate) fn storage(&self) -> Arc<dyn TacticaStorage> {
        self.storage.clone()
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
