use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, insert_into, update};
use diesel_async::{AsyncConnection, RunQueryDsl};
use tactica_db_model::{NewRefreshSession, RefreshSessionStore, StoreError};
use tactica_db_schema::schema::{refresh_sessions, refresh_tokens, users};
use tactica_uuid_kinds::{SessionId, UserId};

use crate::PgConnection;

#[async_trait]
impl RefreshSessionStore for PgConnection {
    async fn create_refresh_session(&self, session: NewRefreshSession) -> Result<(), StoreError> {
        self.conn()
            .await?
            .transaction::<_, StoreError, _>(async move |conn| {
                insert_into(refresh_sessions::table)
                    .values((
                        refresh_sessions::id.eq(session.id.as_uuid()),
                        refresh_sessions::user_id.eq(session.user_id.as_uuid()),
                        refresh_sessions::expires_at.eq(session.expires_at),
                    ))
                    .execute(conn)
                    .await?;
                insert_into(refresh_tokens::table)
                    .values((
                        refresh_tokens::token_hash.eq(session.token_hash),
                        refresh_tokens::session_id.eq(session.id.as_uuid()),
                    ))
                    .execute(conn)
                    .await?;
                Ok(())
            })
            .await
    }

    async fn rotate_refresh_token(
        &self,
        token_hash: String,
        replacement_hash: String,
    ) -> Result<Option<UserId>, StoreError> {
        self.conn()
            .await?
            .transaction::<_, StoreError, _>(async move |conn| {
                let session_id = refresh_tokens::table
                    .find(&token_hash)
                    .select(refresh_tokens::session_id)
                    .first::<SessionId>(conn)
                    .await
                    .optional()?;
                let Some(session_id) = session_id else {
                    return Ok(None);
                };

                // Serialize every rotation and revocation for a login session. Locking
                // only the presented token would race with replay of its predecessor.
                let session = refresh_sessions::table
                    .find(session_id.as_uuid())
                    .for_update()
                    .select((
                        refresh_sessions::user_id,
                        refresh_sessions::expires_at,
                        refresh_sessions::revoked_at,
                    ))
                    .first::<(UserId, DateTime<Utc>, Option<DateTime<Utc>>)>(conn)
                    .await
                    .optional()?;
                let Some((user_id, expires_at, revoked_at)) = session else {
                    return Ok(None);
                };
                let now = Utc::now();
                if revoked_at.is_some() || expires_at <= now {
                    return Ok(None);
                }

                let consumed_at = refresh_tokens::table
                    .find(&token_hash)
                    .select(refresh_tokens::consumed_at)
                    .first::<Option<DateTime<Utc>>>(conn)
                    .await?;
                let active = users::table
                    .find(user_id.as_uuid())
                    .select(users::is_active)
                    .first::<bool>(conn)
                    .await
                    .optional()?
                    .unwrap_or(false);
                if consumed_at.is_some() || !active {
                    update(refresh_sessions::table.find(session_id.as_uuid()))
                        .set(refresh_sessions::revoked_at.eq(now))
                        .execute(conn)
                        .await?;
                    // Return success with no user so the revocation commits.
                    return Ok(None);
                }
                update(refresh_tokens::table.find(&token_hash))
                    .set(refresh_tokens::consumed_at.eq(now))
                    .execute(conn)
                    .await?;
                insert_into(refresh_tokens::table)
                    .values((
                        refresh_tokens::token_hash.eq(replacement_hash),
                        refresh_tokens::session_id.eq(session_id.as_uuid()),
                    ))
                    .execute(conn)
                    .await?;
                Ok(Some(user_id))
            })
            .await
    }

    async fn revoke_refresh_session(&self, token_hash: String) -> Result<(), StoreError> {
        let mut conn = self.conn().await?;
        let session_id = refresh_tokens::table
            .find(token_hash)
            .select(refresh_tokens::session_id)
            .first::<SessionId>(&mut conn)
            .await
            .optional()?;
        if let Some(session_id) = session_id {
            update(
                refresh_sessions::table
                    .find(session_id.as_uuid())
                    .filter(refresh_sessions::revoked_at.is_null()),
            )
            .set(refresh_sessions::revoked_at.eq(Utc::now()))
            .execute(&mut conn)
            .await?;
        }
        Ok(())
    }
}
