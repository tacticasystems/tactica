use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tactica_uuid_kinds::{SessionId, UserId};

#[cfg(feature = "mock")]
use mockall::automock;

use crate::StoreError;

pub struct NewRefreshSession {
    pub id: SessionId,
    pub user_id: UserId,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

#[async_trait]
#[cfg_attr(feature = "mock", automock)]
pub trait RefreshSessionStore {
    /// Atomically creates a session and its initial hashed refresh token.
    async fn create_refresh_session(&self, session: NewRefreshSession) -> Result<(), StoreError>;

    /// Consumes the current token and inserts its replacement atomically.
    /// Returns None for unknown, expired, revoked, replayed, or inactive-user tokens.
    /// Replay revokes the entire session, including any replacement tokens.
    async fn rotate_refresh_token(
        &self,
        token_hash: String,
        replacement_hash: String,
    ) -> Result<Option<UserId>, StoreError>;

    /// Revokes the session identified by any of its token hashes, including consumed tokens.
    /// Unknown tokens are a no-op.
    async fn revoke_refresh_session(&self, token_hash: String) -> Result<(), StoreError>;
}
