//! Database models and storage traits for Tactica.

mod unit;
mod unit_membership;
mod unit_rank;
mod unit_role;
mod unit_settings;
mod user;

use diesel_async::pooled_connection::{PoolError, bb8::RunError};
use thiserror::Error;
pub use unit::*;
pub use unit_membership::*;
pub use unit_rank::*;
pub use unit_role::*;
pub use unit_settings::*;
pub use user::*;

/// A trait that combines all the storage traits into one for convenience.
pub trait TacticaStorage:
    UserStore
    + UnitStore
    + UnitMembershipStore
    + UnitRankStore
    + UnitRoleStore
    + UnitSettingsStore
    + Send
    + Sync
    + 'static
{
}

impl<T> TacticaStorage for T where
    T: UserStore
        + UnitStore
        + UnitMembershipStore
        + UnitRankStore
        + UnitRoleStore
        + UnitSettingsStore
        + Send
        + Sync
        + 'static
{
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Connection failure: {0}")]
    Conn(#[from] RunError),
    #[error("Database failure due to uniqueness constraint")]
    Conflict,
    #[error("Database failure: {0}")]
    Db(diesel::result::Error),
    #[error("Database invariant failed to hold")]
    InvariantFailed(String),
    #[error("Connection pool failure: {0}")]
    Pool(#[from] PoolError),
    #[error("Unknown error")]
    Unknown,
}

impl From<diesel::result::Error> for StoreError {
    fn from(value: diesel::result::Error) -> Self {
        match value {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => Self::Conflict,
            _ => Self::Db(value),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ListPagination {
    pub offset: i64,
    pub limit: i64,
}

impl Default for ListPagination {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 10,
        }
    }
}

/// The direction that a list of records is sorted in
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SortDirection {
    /// Oldest records first
    #[default]
    Ascending,

    /// Newest records first
    Descending,
}

impl ListPagination {
    pub fn latest() -> Self {
        Self::default().limit(1)
    }

    pub fn unlimited() -> Self {
        ListPagination::default().limit(9999999999)
    }

    pub fn offset(mut self, offset: i64) -> Self {
        self.offset = offset;
        self
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = limit;
        self
    }
}
