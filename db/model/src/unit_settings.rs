use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::{Insertable, Queryable, Selectable};
use partial_struct::partial;
use tactica_db_schema::schema::unit_settings;
use tactica_uuid_kinds::{RankId, UnitId, UserId};

#[cfg(feature = "mock")]
use mockall::automock;

use crate::{ListPagination, StoreError};

#[derive(Queryable, Insertable, Selectable, Debug)]
#[diesel(table_name = unit_settings)]
#[partial(NewUnitSettings)]
pub struct UnitSettings {
    pub unit_id: UnitId,

    pub discord_guild_id: Option<String>,
    pub discord_guild_joined_at: Option<NaiveDateTime>,

    pub initial_rank_id: RankId,

    #[partial(NewUnitSettings(skip))]
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<UserId>,
}

#[derive(Debug, Default)]
pub struct UnitSettingsFilter {
    pub unit_id: Option<Vec<UnitId>>,
    pub discord_guild_id: Option<Vec<String>>,
    pub initial_rank_id: Option<Vec<RankId>>,
    pub updated_by: Option<Vec<UserId>>,
}

impl UnitSettingsFilter {
    #[must_use]
    pub fn unit_id(mut self, unit_id: Vec<UnitId>) -> Self {
        self.unit_id = Some(unit_id);
        self
    }

    #[must_use]
    pub fn discord_guild_id(mut self, discord_guild_id: Vec<String>) -> Self {
        self.discord_guild_id = Some(discord_guild_id);
        self
    }

    #[must_use]
    pub fn initial_rank_id(mut self, initial_rank_id: Vec<RankId>) -> Self {
        self.initial_rank_id = Some(initial_rank_id);
        self
    }

    #[must_use]
    pub fn updated_by(mut self, updated_by: Vec<UserId>) -> Self {
        self.updated_by = Some(updated_by);
        self
    }
}

#[async_trait]
#[cfg_attr(feature = "mock", automock)]
pub trait UnitSettingsStore {
    /// Lists unit settings in the database, filtered and paginated by the given arguments.
    async fn list(
        &self,
        filter: UnitSettingsFilter,
        pagination: &ListPagination,
    ) -> Result<Vec<UnitSettings>, StoreError>;

    /// Gets settings by their unit ID.
    async fn get(&self, unit_id: UnitId) -> Result<Option<UnitSettings>, StoreError>;

    /// Gets settings by their Discord guild ID.
    async fn get_by_discord_guild_id(
        &self,
        discord_guild_id: &str,
    ) -> Result<Option<UnitSettings>, StoreError>;

    /// Creates unit settings in the database.
    async fn create(&self, settings: NewUnitSettings) -> Result<UnitSettings, StoreError>;

    /// Updates existing unit settings using their unit ID.
    async fn update(&self, settings: UnitSettings) -> Result<UnitSettings, StoreError>;

    /// Deletes unit settings by their unit ID.
    async fn delete(&self, unit_id: UnitId) -> Result<(), StoreError>;
}
