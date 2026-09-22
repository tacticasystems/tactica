use async_trait::async_trait;
use diesel::{ExpressionMethods, QueryDsl, delete, dsl::insert_into, update};
use diesel_async::{AsyncConnection, RunQueryDsl};
use tactica_db_model::{
    CreateUnit, CreatedUnit, ListPagination, NewUnit, StoreError, Unit, UnitFilter, UnitRank,
    UnitStore,
};
use tactica_db_schema::schema::{unit_memberships, unit_ranks, unit_settings, units};
use tactica_uuid_kinds::{MemberId, RankId, UnitId};

use crate::PgConnection;

#[async_trait]
impl UnitStore for PgConnection {
    async fn create_with_defaults(&self, input: CreateUnit) -> Result<CreatedUnit, StoreError> {
        let mut conn = self.conn().await?;
        conn.transaction::<_, StoreError, _>(async move |conn| {
            let unit = insert_into(units::table)
                .values((
                    units::id.eq(input.unit.id.as_uuid()),
                    units::slug.eq(input.unit.slug),
                    units::display_name.eq(input.unit.display_name),
                    units::icon_url.eq(input.unit.icon_url),
                    units::banner_url.eq(input.unit.banner_url),
                    units::biography.eq(input.unit.biography),
                ))
                .get_result::<Unit>(conn)
                .await?;

            let owner_rank_id = RankId::new();
            let owner_rank = insert_into(unit_ranks::table)
                .values((
                    unit_ranks::id.eq(owner_rank_id.as_uuid()),
                    unit_ranks::unit_id.eq(unit.id.as_uuid()),
                    unit_ranks::slug.eq("Maj."),
                    unit_ranks::display_name.eq(Some("Major".to_owned())),
                    unit_ranks::description.eq(Some("The owner of the unit".to_owned())),
                ))
                .get_result::<UnitRank>(conn)
                .await?;

            let join_rank_id = RankId::new();
            let join_rank = insert_into(unit_ranks::table)
                .values((
                    unit_ranks::id.eq(join_rank_id.as_uuid()),
                    unit_ranks::unit_id.eq(unit.id.as_uuid()),
                    unit_ranks::slug.eq("Pvt."),
                    unit_ranks::display_name.eq(Some("Private".to_owned())),
                    unit_ranks::description.eq(Some("The enlisted members".to_owned())),
                ))
                .get_result::<UnitRank>(conn)
                .await?;

            insert_into(unit_settings::table)
                .values((
                    unit_settings::unit_id.eq(unit.id.as_uuid()),
                    unit_settings::initial_rank_id.eq(join_rank.id.as_uuid()),
                    unit_settings::updated_by.eq(Some(input.owner_id.as_uuid())),
                ))
                .execute(conn)
                .await?;

            let owner_membership_id = MemberId::new();
            insert_into(unit_memberships::table)
                .values((
                    unit_memberships::id.eq(owner_membership_id.as_uuid()),
                    unit_memberships::rank_id.eq(owner_rank.id.as_uuid()),
                    unit_memberships::unit_id.eq(unit.id.as_uuid()),
                    unit_memberships::user_id.eq(input.owner_id.as_uuid()),
                ))
                .execute(conn)
                .await?;

            Ok(CreatedUnit {
                unit,
                owner_rank_id: owner_rank.id,
                owner_membership_id,
            })
        })
        .await
    }

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
            .offset(pagination.offset.0)
            .limit(pagination.limit.0)
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
