use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use tactica_db_model::{NewRefreshSession, StoreError};
use tactica_uuid_kinds::{SessionId, UserId};

use crate::{
    AuthContext,
    jwt::{ACCESS_TOKEN_LIFETIME_SECONDS, JwtError},
};

pub const REFRESH_SESSION_DAYS: i64 = 30;

#[derive(Debug, thiserror::Error)]
pub enum RefreshError {
    #[error("Invalid or expired refresh token")]
    InvalidToken,
    #[error("Failed to generate a refresh token: {0}")]
    Random(#[from] getrandom::Error),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Jwt(#[from] JwtError),
}

// Deliberately omit Debug: these values are credentials.
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

fn generate_refresh_token() -> Result<(String, String), RefreshError> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes)?;
    let token = format!("rt_{}", hex::encode(bytes));
    let hash = hex::encode(Sha256::digest(token.as_bytes()));
    Ok((token, hash))
}

fn token_hash(token: &str) -> Option<String> {
    let secret = token.strip_prefix("rt_")?;
    if secret.len() != 64
        || !secret
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return None;
    }

    Some(hex::encode(Sha256::digest(token.as_bytes())))
}

impl AuthContext {
    pub async fn issue_tokens(&self, user_id: UserId) -> Result<TokenPair, RefreshError> {
        let (refresh_token, token_hash) = generate_refresh_token()?;
        let access_token = self.jwt.generate_jwt_for_user(user_id)?;

        self.storage
            .create_refresh_session(NewRefreshSession {
                id: SessionId::new(),
                user_id,
                token_hash,
                expires_at: Utc::now() + Duration::days(REFRESH_SESSION_DAYS),
            })
            .await?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: ACCESS_TOKEN_LIFETIME_SECONDS,
        })
    }

    pub async fn refresh_tokens(&self, token: &str) -> Result<TokenPair, RefreshError> {
        let hash = token_hash(token).ok_or(RefreshError::InvalidToken)?;

        let (refresh_token, replacement_hash) = generate_refresh_token()?;
        let user_id = self
            .storage
            .rotate_refresh_token(hash, replacement_hash)
            .await?
            .ok_or(RefreshError::InvalidToken)?;

        let access_token = self.jwt.generate_jwt_for_user(user_id)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: ACCESS_TOKEN_LIFETIME_SECONDS,
        })
    }

    pub async fn revoke_refresh_token(&self, token: &str) -> Result<(), RefreshError> {
        if let Some(hash) = token_hash(token) {
            self.storage.revoke_refresh_session(hash).await?;
        }

        Ok(())
    }
}
