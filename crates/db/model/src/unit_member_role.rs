use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::{Insertable, Queryable, Selectable};
use partial_struct::partial;
use tactica_db_schema::schema::unit_member_roles;
use tactica_uuid_kinds::{MemberId, RoleId, UnitId};

#[cfg(feature = "mock")]
use mockall::automock;

use crate::{ListPagination, StoreError};

/// A role assigned to a membership in the same unit.
#[derive(Queryable, Insertable, Selectable, Debug)]
#[diesel(table_name = unit_member_roles)]
#[partial(NewUnitMemberRole)]
pub struct UnitMemberRole {
    pub member_id: MemberId,
    pub role_id: RoleId,
    pub unit_id: UnitId,
    #[partial(NewUnitMemberRole(skip))]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub struct UnitMemberRoleFilter {
    pub member_id: Option<Vec<MemberId>>,
    pub role_id: Option<Vec<RoleId>>,
    pub unit_id: Option<Vec<UnitId>>,
}

impl UnitMemberRoleFilter {
    #[must_use]
    pub fn member_id(mut self, member_id: Vec<MemberId>) -> Self {
        self.member_id = Some(member_id);
        self
    }

    #[must_use]
    pub fn role_id(mut self, role_id: Vec<RoleId>) -> Self {
        self.role_id = Some(role_id);
        self
    }

    #[must_use]
    pub fn unit_id(mut self, unit_id: Vec<UnitId>) -> Self {
        self.unit_id = Some(unit_id);
        self
    }
}

#[async_trait]
#[cfg_attr(feature = "mock", automock)]
pub trait UnitMemberRoleStore {
    /// Assigns a role idempotently, preserving an existing assignment.
    async fn assign(&self, assignment: NewUnitMemberRole) -> Result<(), StoreError>;

    /// Lists assignments, optionally filtered by member, role, or unit.
    async fn list(
        &self,
        filter: UnitMemberRoleFilter,
        pagination: &ListPagination,
    ) -> Result<Vec<UnitMemberRole>, StoreError>;

    /// Assigns a role to a member. Duplicate assignments return a conflict.
    /// Both the member and role must belong to the supplied unit.
    async fn create(&self, assignment: NewUnitMemberRole) -> Result<UnitMemberRole, StoreError>;

    /// Removes an assignment. An absent assignment is a no-op.
    async fn delete(&self, member_id: MemberId, role_id: RoleId) -> Result<(), StoreError>;
}
