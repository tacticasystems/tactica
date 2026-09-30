use async_trait::async_trait;
use diesel::{ExpressionMethods, QueryDsl, delete, dsl::insert_into, update};
use diesel_async::RunQueryDsl;
use tactica_db_model::{
    ListPagination, NewUnitSettings, StoreError, UnitSettings, UnitSettingsFilter,
    UnitSettingsStore,
};
use tactica_db_schema::schema::unit_settings;
use tactica_uuid_kinds::UnitId;

use crate::PgConnection;

#[async_trait]
impl UnitSettingsStore for PgConnection {
    async fn list(
        &self,
        filter: UnitSettingsFilter,
        pagination: &ListPagination,
    ) -> Result<Vec<UnitSettings>, StoreError> {
        let mut query = unit_settings::table.into_boxed();
        let UnitSettingsFilter {
            unit_id,
            discord_guild_id,
            initial_rank_id,
            updated_by,
        } = filter;

        if let Some(unit_id) = unit_id {
            query = query
                .filter(unit_settings::unit_id.eq_any(unit_id.into_iter().map(|id| *id.as_uuid())));
        }
        if let Some(discord_guild_id) = discord_guild_id {
            query = query.filter(unit_settings::discord_guild_id.eq_any(discord_guild_id));
        }
        if let Some(initial_rank_id) = initial_rank_id {
            query = query.filter(
                unit_settings::initial_rank_id
                    .eq_any(initial_rank_id.into_iter().map(|id| *id.as_uuid())),
            );
        }
        if let Some(updated_by) = updated_by {
            query = query.filter(
                unit_settings::updated_by.eq_any(updated_by.into_iter().map(|id| *id.as_uuid())),
            );
        }

        Ok(query
            .offset(pagination.offset.0)
            .limit(pagination.limit.0)
            .get_results(&mut self.conn().await?)
            .await?)
    }

    async fn get(&self, unit_id: UnitId) -> Result<Option<UnitSettings>, StoreError> {
        Ok(Self::list(
            self,
            UnitSettingsFilter::default().unit_id(vec![unit_id]),
            &ListPagination::latest(),
        )
        .await?
        .into_iter()
        .next())
    }

    async fn get_by_discord_guild_id(
        &self,
        discord_guild_id: &str,
    ) -> Result<Option<UnitSettings>, StoreError> {
        Ok(Self::list(
            self,
            UnitSettingsFilter::default().discord_guild_id(vec![discord_guild_id.to_owned()]),
            &ListPagination::latest(),
        )
        .await?
        .into_iter()
        .next())
    }

    async fn create(&self, settings: NewUnitSettings) -> Result<UnitSettings, StoreError> {
        insert_into(unit_settings::table)
            .values((
                unit_settings::unit_id.eq(settings.unit_id.as_uuid()),
                unit_settings::discord_guild_id.eq(settings.discord_guild_id),
                unit_settings::discord_guild_joined_at.eq(settings.discord_guild_joined_at),
                unit_settings::initial_rank_id.eq(settings.initial_rank_id.as_uuid()),
                unit_settings::updated_by.eq(settings.updated_by.map(|id| *id.as_uuid())),
            ))
            .get_result(&mut self.conn().await?)
            .await
            .map_err(Into::into)
    }

    async fn update(&self, settings: UnitSettings) -> Result<UnitSettings, StoreError> {
        update(unit_settings::table.filter(unit_settings::unit_id.eq(settings.unit_id.as_uuid())))
            .set((
                unit_settings::discord_guild_id.eq(settings.discord_guild_id),
                unit_settings::discord_guild_joined_at.eq(settings.discord_guild_joined_at),
                unit_settings::initial_rank_id.eq(settings.initial_rank_id.as_uuid()),
                unit_settings::updated_by.eq(settings.updated_by.map(|id| *id.as_uuid())),
            ))
            .get_result(&mut self.conn().await?)
            .await
            .map_err(Into::into)
    }

    async fn delete(&self, unit_id: UnitId) -> Result<(), StoreError> {
        delete(unit_settings::table.filter(unit_settings::unit_id.eq(unit_id.as_uuid())))
            .execute(&mut self.conn().await?)
            .await?;
        Ok(())
    }
}
