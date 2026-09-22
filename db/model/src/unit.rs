use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::{Insertable, Queryable, Selectable};
use partial_struct::partial;
use tactica_db_schema::schema::units;
use tactica_uuid_kinds::{MemberId, RankId, UnitId, UserId};

#[cfg(feature = "mock")]
use mockall::automock;

use crate::{ListPagination, StoreError};

#[derive(Queryable, Insertable, Selectable, Debug)]
#[diesel(table_name = units)]
#[partial(NewUnit)]
pub struct Unit {
    pub id: UnitId,

    pub slug: String,

    pub display_name: Option<String>,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub biography: Option<String>,

    #[partial(NewUnit(skip))]
    pub created_at: DateTime<Utc>,
    #[partial(NewUnit(skip))]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub struct UnitFilter {
    pub id: Option<Vec<UnitId>>,
    pub slug: Option<Vec<String>>,
}

/// Data needed to create a unit together with its initial ranks, settings,
/// and owner membership as one atomic operation.
pub struct CreateUnit {
    pub unit: NewUnit,
    pub owner_id: UserId,
}

/// The records returned after creating a unit and its defaults.
pub struct CreatedUnit {
    pub unit: Unit,
    pub owner_rank_id: RankId,
    pub owner_membership_id: MemberId,
}

impl UnitFilter {
    #[must_use]
    pub fn id(mut self, id: Vec<UnitId>) -> Self {
        self.id = Some(id);
        self
    }

    #[must_use]
    pub fn slug(mut self, slug: Vec<String>) -> Self {
        self.slug = Some(slug);
        self
    }
}

#[async_trait]
#[cfg_attr(feature = "mock", automock)]
pub trait UnitStore {
    /// Creates a unit, its default ranks and settings, and its owner's
    /// membership atomically.
    async fn create_with_defaults(&self, input: CreateUnit) -> Result<CreatedUnit, StoreError>;

    /// Lists units in the database, filtered and paginated by the given arguments.
    async fn list(
        &self,
        filter: UnitFilter,
        pagination: &ListPagination,
    ) -> Result<Vec<Unit>, StoreError>;

    /// Gets a unit by its ID.
    async fn get(&self, id: UnitId) -> Result<Option<Unit>, StoreError>;

    /// Gets a unit by its slug.
    async fn get_by_slug(&self, slug: &str) -> Result<Option<Unit>, StoreError>;

    /// Creates a new unit in the database.
    async fn create(&self, unit: NewUnit) -> Result<Unit, StoreError>;

    /// Updates an existing unit using its ID.
    async fn update(&self, unit: Unit) -> Result<Unit, StoreError>;

    /// Deletes a unit by its ID.
    async fn delete(&self, id: UnitId) -> Result<(), StoreError>;
}
