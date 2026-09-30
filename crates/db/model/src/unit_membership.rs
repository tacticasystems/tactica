use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::{Insertable, Queryable, Selectable};
use partial_struct::partial;
use tactica_db_schema::schema::unit_memberships;
use tactica_uuid_kinds::{MemberId, RankId, UnitId, UserId};

#[cfg(feature = "mock")]
use mockall::automock;

use crate::{ListPagination, StoreError};

#[derive(Queryable, Insertable, Selectable, Debug)]
#[diesel(table_name = unit_memberships)]
#[partial(NewUnitMembership)]
pub struct UnitMembership {
    pub id: MemberId,

    pub user_id: UserId,
    pub unit_id: UnitId,
    pub rank_id: RankId,

    #[partial(NewUnitMembership(skip))]
    pub created_at: DateTime<Utc>,
    #[partial(NewUnitMembership(skip))]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub struct UnitMembershipFilter {
    pub id: Option<Vec<MemberId>>,
    pub user_id: Option<Vec<UserId>>,
    pub unit_id: Option<Vec<UnitId>>,
    pub rank_id: Option<Vec<RankId>>,
}

impl UnitMembershipFilter {
    #[must_use]
    pub fn id(mut self, id: Vec<MemberId>) -> Self {
        self.id = Some(id);
        self
    }

    #[must_use]
    pub fn user_id(mut self, user_id: Vec<UserId>) -> Self {
        self.user_id = Some(user_id);
        self
    }

    #[must_use]
    pub fn unit_id(mut self, unit_id: Vec<UnitId>) -> Self {
        self.unit_id = Some(unit_id);
        self
    }

    #[must_use]
    pub fn rank_id(mut self, rank_id: Vec<RankId>) -> Self {
        self.rank_id = Some(rank_id);
        self
    }
}

#[async_trait]
#[cfg_attr(feature = "mock", automock)]
pub trait UnitMembershipStore {
    /// Counts memberships for each requested unit. Units without members are omitted.
    async fn count_by_unit(
        &self,
        unit_ids: Vec<UnitId>,
    ) -> Result<HashMap<UnitId, i64>, StoreError>;

    /// Lists unit memberships in the database, filtered and paginated by the given arguments.
    async fn list(
        &self,
        filter: UnitMembershipFilter,
        pagination: &ListPagination,
    ) -> Result<Vec<UnitMembership>, StoreError>;

    /// Gets a unit membership by its ID.
    async fn get(&self, id: MemberId) -> Result<Option<UnitMembership>, StoreError>;

    /// Gets a user's membership in a unit.
    async fn get_by_user_and_unit(
        &self,
        user_id: UserId,
        unit_id: UnitId,
    ) -> Result<Option<UnitMembership>, StoreError>;

    /// Creates a new unit membership in the database.
    async fn create(&self, membership: NewUnitMembership) -> Result<UnitMembership, StoreError>;

    /// Updates an existing unit membership using its ID.
    async fn update(&self, membership: UnitMembership) -> Result<UnitMembership, StoreError>;

    /// Deletes a unit membership by its ID.
    async fn delete(&self, id: MemberId) -> Result<(), StoreError>;
}
