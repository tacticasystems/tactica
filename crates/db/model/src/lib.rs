//! Database models and storage traits for Tactica.

mod refresh_session;
mod unit;
mod unit_member_management;
mod unit_member_role;
mod unit_membership;
mod unit_rank;
mod unit_rank_management;
mod unit_role;
mod unit_role_management;
mod unit_settings;
mod user;

use diesel_async::pooled_connection::{PoolError, bb8::RunError};
pub use refresh_session::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;
pub use unit::*;
pub use unit_member_management::*;
pub use unit_member_role::*;
pub use unit_membership::*;
pub use unit_rank::*;
pub use unit_rank_management::*;
pub use unit_role::*;
pub use unit_role_management::*;
pub use unit_settings::*;
pub use user::*;

/// A trait that combines all the storage traits into one for convenience.
pub trait TacticaStorage:
    UserStore
    + RefreshSessionStore
    + UnitStore
    + UnitMembershipStore
    + UnitMemberManagementStore
    + UnitMemberRoleStore
    + UnitRankStore
    + UnitRankManagementStore
    + UnitRoleStore
    + UnitRoleManagementStore
    + UnitSettingsStore
    + Send
    + Sync
    + 'static
{
}

impl<T> TacticaStorage for T where
    T: UserStore
        + RefreshSessionStore
        + UnitStore
        + UnitMembershipStore
        + UnitMemberManagementStore
        + UnitMemberRoleStore
        + UnitRankStore
        + UnitRankManagementStore
        + UnitRoleStore
        + UnitRoleManagementStore
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

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub struct ListPagination {
    #[serde(default)]
    pub offset: Offset,
    #[serde(default)]
    pub limit: Limit,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub struct Offset(pub i64);
impl Default for Offset {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub struct Limit(pub i64);
impl Default for Limit {
    fn default() -> Self {
        Self(10)
    }
}

impl Default for ListPagination {
    fn default() -> Self {
        Self {
            offset: Offset(0),
            limit: Limit(10),
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
    #[must_use]
    pub fn latest() -> Self {
        Self::default().limit(1)
    }

    #[must_use]
    pub fn unlimited() -> Self {
        Self::default().limit(9_999_999_999)
    }

    #[must_use]
    pub const fn offset(mut self, offset: i64) -> Self {
        self.offset = Offset(offset);
        self
    }

    #[must_use]
    pub const fn limit(mut self, limit: i64) -> Self {
        self.limit = Limit(limit);
        self
    }
}
