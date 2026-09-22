use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::{Insertable, Queryable, Selectable};
use partial_struct::partial;
use tactica_db_schema::schema::unit_roles;
use tactica_uuid_kinds::{RoleId, UnitId};

#[cfg(feature = "mock")]
use mockall::automock;

use crate::{ListPagination, StoreError};

/// Stable role-kind identifiers. Built-ins are recognized by kind, never by name.
pub mod role_kind {
    pub const CUSTOM: &str = "custom";
    pub const ADMINISTRATOR: &str = "administrator";
    pub const EVERYONE: &str = "everyone";
}

#[derive(Debug, Default, diesel::AsChangeset)]
#[diesel(table_name = unit_roles)]
pub struct UnitRolePatch {
    pub display_name: Option<String>,
    pub description: Option<Option<String>>,
    pub permissions: Option<i64>,
}

#[derive(Queryable, Insertable, Selectable, Debug)]
#[diesel(table_name = unit_roles)]
#[partial(NewUnitRole)]
pub struct UnitRole {
    pub id: RoleId,

    pub unit_id: UnitId,
    pub display_name: String,
    pub description: Option<String>,

    pub permissions: i64,

    /// Higher positions outrank lower ones. New roles start at the bottom.
    #[partial(NewUnitRole(skip))]
    pub position: i64,

    #[partial(NewUnitRole(skip))]
    pub kind: String,

    #[partial(NewUnitRole(skip))]
    pub created_at: DateTime<Utc>,
    #[partial(NewUnitRole(skip))]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub struct UnitRoleFilter {
    pub id: Option<Vec<RoleId>>,
    pub unit_id: Option<Vec<UnitId>>,
    pub display_name: Option<Vec<String>>,
}

impl UnitRoleFilter {
    #[must_use]
    pub fn id(mut self, id: Vec<RoleId>) -> Self {
        self.id = Some(id);
        self
    }

    #[must_use]
    pub fn unit_id(mut self, unit_id: Vec<UnitId>) -> Self {
        self.unit_id = Some(unit_id);
        self
    }

    #[must_use]
    pub fn display_name(mut self, display_name: Vec<String>) -> Self {
        self.display_name = Some(display_name);
        self
    }
}

#[async_trait]
#[cfg_attr(feature = "mock", automock)]
pub trait UnitRoleStore {
    /// Lists roles applying to a current member, including the implicit Everyone role.
    /// Returns no roles when the member does not belong to the unit.
    async fn list_for_member(
        &self,
        unit_id: UnitId,
        member_id: tactica_uuid_kinds::MemberId,
        pagination: &ListPagination,
    ) -> Result<Vec<UnitRole>, StoreError>;

    /// Updates only supplied fields, scoped to the unit. None means no matching role.
    async fn patch_in_unit(
        &self,
        unit_id: UnitId,
        id: RoleId,
        patch: UnitRolePatch,
    ) -> Result<Option<UnitRole>, StoreError>;

    /// Deletes a role in the unit, returning whether it existed.
    async fn delete_in_unit(&self, unit_id: UnitId, id: RoleId) -> Result<bool, StoreError>;

    /// Lists unit roles in the database, filtered and paginated by the given arguments.
    async fn list(
        &self,
        filter: UnitRoleFilter,
        pagination: &ListPagination,
    ) -> Result<Vec<UnitRole>, StoreError>;

    /// Gets a unit role by its ID.
    async fn get(&self, id: RoleId) -> Result<Option<UnitRole>, StoreError>;

    /// Gets a unit role by its unit and display name.
    async fn get_by_unit_and_display_name(
        &self,
        unit_id: UnitId,
        display_name: &str,
    ) -> Result<Option<UnitRole>, StoreError>;

    /// Creates a new unit role in the database.
    async fn create(&self, role: NewUnitRole) -> Result<UnitRole, StoreError>;

    /// Updates an existing unit role using its ID.
    async fn update(&self, role: UnitRole) -> Result<UnitRole, StoreError>;

    /// Deletes a unit role by its ID.
    async fn delete(&self, id: RoleId) -> Result<(), StoreError>;
}
