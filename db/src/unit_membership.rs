use async_trait::async_trait;
use diesel::{ExpressionMethods, QueryDsl, delete, dsl::insert_into, update};
use diesel_async::RunQueryDsl;
use tactica_db_model::{
    ListPagination, NewUnitMembership, StoreError, UnitMembership, UnitMembershipFilter,
    UnitMembershipStore,
};
use tactica_db_schema::schema::unit_memberships;
use tactica_uuid_kinds::{MemberId, UnitId, UserId};

use crate::PgConnection;

#[async_trait]
impl UnitMembershipStore for PgConnection {
    async fn list(
        &self,
        filter: UnitMembershipFilter,
        pagination: &ListPagination,
    ) -> Result<Vec<UnitMembership>, StoreError> {
        let mut query = unit_memberships::table.into_boxed();
        let UnitMembershipFilter {
            id,
            user_id,
            unit_id,
            rank_id,
        } = filter;

        if let Some(id) = id {
            query =
                query.filter(unit_memberships::id.eq_any(id.into_iter().map(|id| *id.as_uuid())));
        }
        if let Some(user_id) = user_id {
            query = query.filter(
                unit_memberships::user_id.eq_any(user_id.into_iter().map(|id| *id.as_uuid())),
            );
        }
        if let Some(unit_id) = unit_id {
            query = query.filter(
                unit_memberships::unit_id.eq_any(unit_id.into_iter().map(|id| *id.as_uuid())),
            );
        }
        if let Some(rank_id) = rank_id {
            query = query.filter(
                unit_memberships::rank_id.eq_any(rank_id.into_iter().map(|id| *id.as_uuid())),
            );
        }

        Ok(query
            .offset(pagination.offset.0)
            .limit(pagination.limit.0)
            .get_results(&mut self.conn().await?)
            .await?)
    }

    async fn get(&self, id: MemberId) -> Result<Option<UnitMembership>, StoreError> {
        Ok(Self::list(
            self,
            UnitMembershipFilter::default().id(vec![id]),
            &ListPagination::latest(),
        )
        .await?
        .into_iter()
        .next())
    }

    async fn get_by_user_and_unit(
        &self,
        user_id: UserId,
        unit_id: UnitId,
    ) -> Result<Option<UnitMembership>, StoreError> {
        Ok(Self::list(
            self,
            UnitMembershipFilter::default()
                .user_id(vec![user_id])
                .unit_id(vec![unit_id]),
            &ListPagination::latest(),
        )
        .await?
        .into_iter()
        .next())
    }

    async fn create(&self, membership: NewUnitMembership) -> Result<UnitMembership, StoreError> {
        insert_into(unit_memberships::table)
            .values((
                unit_memberships::id.eq(membership.id.as_uuid()),
                unit_memberships::user_id.eq(membership.user_id.as_uuid()),
                unit_memberships::unit_id.eq(membership.unit_id.as_uuid()),
                unit_memberships::rank_id.eq(membership.rank_id.as_uuid()),
            ))
            .get_result(&mut self.conn().await?)
            .await
            .map_err(Into::into)
    }

    async fn update(&self, membership: UnitMembership) -> Result<UnitMembership, StoreError> {
        update(unit_memberships::table.filter(unit_memberships::id.eq(membership.id.as_uuid())))
            .set((
                unit_memberships::user_id.eq(membership.user_id.as_uuid()),
                unit_memberships::unit_id.eq(membership.unit_id.as_uuid()),
                unit_memberships::rank_id.eq(membership.rank_id.as_uuid()),
            ))
            .get_result(&mut self.conn().await?)
            .await
            .map_err(Into::into)
    }

    async fn delete(&self, id: MemberId) -> Result<(), StoreError> {
        delete(unit_memberships::table.filter(unit_memberships::id.eq(id.as_uuid())))
            .execute(&mut self.conn().await?)
            .await?;
        Ok(())
    }
}
