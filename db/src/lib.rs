//! This module contains the database connection and storage trait
//! implementations.

use diesel_async::{
    AsyncPgConnection,
    pooled_connection::{
        AsyncDieselConnectionManager, PoolError,
        bb8::{Pool, PooledConnection, RunError},
    },
};

mod unit;
mod unit_membership;
mod unit_rank;
mod unit_role;
mod unit_settings;
mod user;

#[derive(Debug, Clone)]
pub struct PgConnection(Pool<AsyncPgConnection>);

impl PgConnection {
    pub async fn new(database_url: &str) -> Result<Self, PoolError> {
        let config =
            AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
        let pool: Pool<AsyncPgConnection> = Pool::builder().build(config).await?;
        Ok(Self(pool))
    }

    pub async fn conn(&self) -> Result<PooledConnection<'_, AsyncPgConnection>, RunError> {
        self.0.get().await
    }

    #[must_use]
    pub fn pool(&self) -> Pool<AsyncPgConnection> {
        self.0.clone()
    }
}
