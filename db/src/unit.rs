use async_trait::async_trait;
use diesel::{ExpressionMethods, QueryDsl, delete, dsl::insert_into, update};
use diesel_async::RunQueryDsl;
use tactica_db_model::{ListPagination, NewUnit, StoreError, Unit, UnitFilter, UnitStore};
use tactica_db_schema::schema::units;
use tactica_uuid_kinds::UnitId;

use crate::PgConnection;

#[async_trait]
impl UnitStore for PgConnection {
    async fn list(
        &self,
        filter: UnitFilter,
        pagination: &ListPagination,
    ) -> Result<Vec<Unit>, StoreError> {
        let mut query = units::table.into_boxed();
        let UnitFilter { id, slug } = filter;

        if let Some(id) = id {
            query = query.filter(units::id.eq_any(id.into_iter().map(|id| *id.as_uuid())));
        }
        if let Some(slug) = slug {
            query = query.filter(units::slug.eq_any(slug));
        }

        Ok(query
            .offset(pagination.offset)
            .limit(pagination.limit)
            .get_results(&mut self.conn().await?)
            .await?)
    }

    async fn get(&self, id: UnitId) -> Result<Option<Unit>, StoreError> {
        Ok(Self::list(
            self,
            UnitFilter::default().id(vec![id]),
            &ListPagination::latest(),
        )
        .await?
        .into_iter()
        .next())
    }

    async fn get_by_slug(&self, slug: &str) -> Result<Option<Unit>, StoreError> {
        Ok(Self::list(
            self,
            UnitFilter::default().slug(vec![slug.to_owned()]),
            &ListPagination::latest(),
        )
        .await?
        .into_iter()
        .next())
    }

    async fn create(&self, unit: NewUnit) -> Result<Unit, StoreError> {
        insert_into(units::table)
            .values((
                units::id.eq(unit.id.as_uuid()),
                units::slug.eq(unit.slug),
                units::display_name.eq(unit.display_name),
                units::icon_url.eq(unit.icon_url),
                units::banner_url.eq(unit.banner_url),
                units::biography.eq(unit.biography),
            ))
            .get_result(&mut self.conn().await?)
            .await
            .map_err(Into::into)
    }

    async fn update(&self, unit: Unit) -> Result<Unit, StoreError> {
        update(units::table.filter(units::id.eq(unit.id.as_uuid())))
            .set((
                units::slug.eq(unit.slug),
                units::display_name.eq(unit.display_name),
                units::icon_url.eq(unit.icon_url),
                units::banner_url.eq(unit.banner_url),
                units::biography.eq(unit.biography),
            ))
            .get_result(&mut self.conn().await?)
            .await
            .map_err(Into::into)
    }

    async fn delete(&self, id: UnitId) -> Result<(), StoreError> {
        delete(units::table.filter(units::id.eq(id.as_uuid())))
            .execute(&mut self.conn().await?)
            .await?;
        Ok(())
    }
}
