use async_trait::async_trait;
use diesel::{ExpressionMethods, QueryDsl, delete, dsl::insert_into, update};
use diesel_async::RunQueryDsl;
use tactica_db_model::{
    ListPagination, NewUnitRank, StoreError, UnitRank, UnitRankFilter, UnitRankStore,
};
use tactica_db_schema::schema::unit_ranks;
use tactica_uuid_kinds::{RankId, UnitId};

use crate::PgConnection;

#[async_trait]
impl UnitRankStore for PgConnection {
    async fn list(
        &self,
        filter: UnitRankFilter,
        pagination: &ListPagination,
    ) -> Result<Vec<UnitRank>, StoreError> {
        let mut query = unit_ranks::table.into_boxed();
        let UnitRankFilter { id, unit_id, slug } = filter;

        if let Some(id) = id {
            query = query.filter(unit_ranks::id.eq_any(id.into_iter().map(|id| *id.as_uuid())));
        }
        if let Some(unit_id) = unit_id {
            query = query
                .filter(unit_ranks::unit_id.eq_any(unit_id.into_iter().map(|id| *id.as_uuid())));
        }
        if let Some(slug) = slug {
            query = query.filter(unit_ranks::slug.eq_any(slug));
        }

        Ok(query
            .offset(pagination.offset)
            .limit(pagination.limit)
            .get_results(&mut self.conn().await?)
            .await?)
    }

    async fn get(&self, id: RankId) -> Result<Option<UnitRank>, StoreError> {
        Ok(Self::list(
            self,
            UnitRankFilter::default().id(vec![id]),
            &ListPagination::latest(),
        )
        .await?
        .into_iter()
        .next())
    }

    async fn get_by_unit_and_slug(
        &self,
        unit_id: UnitId,
        slug: &str,
    ) -> Result<Option<UnitRank>, StoreError> {
        Ok(Self::list(
            self,
            UnitRankFilter::default()
                .unit_id(vec![unit_id])
                .slug(vec![slug.to_owned()]),
            &ListPagination::latest(),
        )
        .await?
        .into_iter()
        .next())
    }

    async fn create(&self, rank: NewUnitRank) -> Result<UnitRank, StoreError> {
        insert_into(unit_ranks::table)
            .values((
                unit_ranks::id.eq(rank.id.as_uuid()),
                unit_ranks::unit_id.eq(rank.unit_id.as_uuid()),
                unit_ranks::slug.eq(rank.slug),
                unit_ranks::display_name.eq(rank.display_name),
                unit_ranks::icon_url.eq(rank.icon_url),
                unit_ranks::description.eq(rank.description),
            ))
            .get_result(&mut self.conn().await?)
            .await
            .map_err(Into::into)
    }

    async fn update(&self, rank: UnitRank) -> Result<UnitRank, StoreError> {
        update(unit_ranks::table.filter(unit_ranks::id.eq(rank.id.as_uuid())))
            .set((
                unit_ranks::unit_id.eq(rank.unit_id.as_uuid()),
                unit_ranks::slug.eq(rank.slug),
                unit_ranks::display_name.eq(rank.display_name),
                unit_ranks::icon_url.eq(rank.icon_url),
                unit_ranks::description.eq(rank.description),
            ))
            .get_result(&mut self.conn().await?)
            .await
            .map_err(Into::into)
    }

    async fn delete(&self, id: RankId) -> Result<(), StoreError> {
        delete(unit_ranks::table.filter(unit_ranks::id.eq(id.as_uuid())))
            .execute(&mut self.conn().await?)
            .await?;
        Ok(())
    }
}
