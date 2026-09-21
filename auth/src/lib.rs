use std::sync::Arc;
use tokio::sync::OnceCell;
use argon2::{Argon2, PasswordHasher, password_hash::phc::SaltString};
use tactica_db_model::TacticaStorage;

use crate::jwt::JwtContext;

pub mod jwt;
pub mod utils;

pub(crate) const ARGON2: OnceCell<Argon2> = OnceCell::const_new();

pub(crate) async fn get_argon2() -> Argon2<'static> {
    ARGON2.get_or_init(async || {Argon2::default()})
        .await
        .clone()
}

#[derive(Clone)]
pub struct AuthContext {
    storage: Arc<dyn TacticaStorage>,
    jwt: JwtContext,

    salt: SaltString,
}

impl AuthContext {
    pub fn new(
        storage: Arc<dyn TacticaStorage>,
        jwt: JwtContext
    ) -> Self {
        Self {
            storage,
            jwt,

            salt: SaltString::generate(),
        }
    }

    pub fn storage(&self) -> Arc<dyn TacticaStorage> {
        self.storage.clone()
    }

    pub fn jwt(&self) -> JwtContext {
        self.jwt.clone()
    }

    pub async fn hash_password(&self, password: &str) -> Result<String, argon2::password_hash::Error> {
        let argon2 = get_argon2().await;
        let password_hash = argon2.hash_password_with_salt(password.as_bytes(), &self.salt.as_bytes())?;
        Ok(password_hash.to_string())
    }
}
