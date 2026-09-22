use argon2::{Argon2, PasswordHasher, password_hash::phc::SaltString};
use std::sync::Arc;
use tactica_db_model::TacticaStorage;
use tokio::sync::OnceCell;

use crate::jwt::JwtContext;

pub mod jwt;
pub mod principal;

pub(crate) static ARGON2: OnceCell<Argon2> = OnceCell::const_new();

pub(crate) async fn get_argon2() -> Argon2<'static> {
    ARGON2.get_or_init(async || Argon2::default()).await.clone()
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
        jwt: JwtContext,
        salt: &str,
    ) -> Result<Self, argon2::password_hash::phc::Error> {
        Ok(Self {
            storage,
            jwt,

            salt: SaltString::from_b64(salt)?,
        })
    }

    #[must_use]
    pub fn storage(&self) -> Arc<dyn TacticaStorage> {
        self.storage.clone()
    }

    #[must_use]
    pub fn jwt(&self) -> JwtContext {
        self.jwt.clone()
    }

    pub async fn hash_password(
        &self,
        password: &str,
    ) -> Result<String, argon2::password_hash::Error> {
        let argon2 = get_argon2().await;
        let password_hash =
            argon2.hash_password_with_salt(password.as_bytes(), self.salt.as_bytes())?;
        Ok(password_hash.to_string())
    }
}
