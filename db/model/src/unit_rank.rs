use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::{Insertable, Queryable, Selectable};
use partial_struct::partial;
use tactica_db_schema::schema::unit_ranks;
use tactica_uuid_kinds::{RankId, UnitId};

#[cfg(feature = "mock")]
use mockall::automock;

use crate::{ListPagination, StoreError};

#[derive(Queryable, Insertable, Selectable, Debug)]
#[diesel(table_name = unit_ranks)]
#[partial(NewUnitRank)]
pub struct UnitRank {
    pub id: RankId,

    pub unit_id: UnitId,

    pub slug: String,

    pub display_name: Option<String>,
    pub icon_url: Option<String>,
    pub description: Option<String>,

    #[partial(NewUnitRank(skip))]
    pub created_at: DateTime<Utc>,
    #[partial(NewUnitRank(skip))]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub struct UnitRankFilter {
    pub id: Option<Vec<RankId>>,
    pub unit_id: Option<Vec<UnitId>>,
    pub slug: Option<Vec<String>>,
}

impl UnitRankFilter {
    pub fn id(mut self, id: Vec<RankId>) -> Self {
        self.id = Some(id);
        self
    }

    pub fn unit_id(mut self, unit_id: Vec<UnitId>) -> Self {
        self.unit_id = Some(unit_id);
        self
    }

    pub fn slug(mut self, slug: Vec<String>) -> Self {
        self.slug = Some(slug);
        self
    }
}

#[async_trait]
#[cfg_attr(feature = "mock", automock)]
pub trait UnitRankStore {
    /// Lists unit ranks in the database, filtered and paginated by the given arguments.
    async fn list(
        &self,
        filter: UnitRankFilter,
        pagination: &ListPagination,
    ) -> Result<Vec<UnitRank>, StoreError>;

    /// Gets a unit rank by its ID.
    async fn get(&self, id: RankId) -> Result<Option<UnitRank>, StoreError>;

    /// Gets a unit rank by its unit and slug.
    async fn get_by_unit_and_slug(
        &self,
        unit_id: UnitId,
        slug: &str,
    ) -> Result<Option<UnitRank>, StoreError>;

    /// Creates a new unit rank in the database.
    async fn create(&self, rank: NewUnitRank) -> Result<UnitRank, StoreError>;

    /// Updates an existing unit rank using its ID.
    async fn update(&self, rank: UnitRank) -> Result<UnitRank, StoreError>;

    /// Deletes a unit rank by its ID.
    async fn delete(&self, id: RankId) -> Result<(), StoreError>;
}
