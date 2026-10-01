use crate::{PgConnection, unit_rank::insert_at_bottom, unit_role_management::authorize};
use async_trait::async_trait;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, delete, update};
use diesel_async::{AsyncConnection, RunQueryDsl};
use std::collections::HashSet;
use tactica_db_model::{
    NewUnitRank, RankWriteError, StoreError, UnitRank, UnitRankManagementStore, UnitRankPatch,
};
use tactica_db_schema::schema::{unit_memberships, unit_ranks, unit_settings};
use tactica_permissions::Permission;
use tactica_uuid_kinds::{MemberId, RankId, UnitId, UserId};

#[async_trait]
impl UnitRankManagementStore for PgConnection {
    async fn set_member_rank(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        member_id: MemberId,
        rank_id: RankId,
    ) -> Result<(), RankWriteError> {
        self.conn()
            .await
            .map_err(StoreError::from)?
            .transaction::<_, RankWriteError, _>(async move |conn| {
                authorize(conn, actor_id, unit_id, Permission::AssignRanks).await?;
                unit_ranks::table
                    .find(rank_id.as_uuid())
                    .filter(unit_ranks::unit_id.eq(unit_id.as_uuid()))
                    .first::<UnitRank>(conn)
                    .await
                    .optional()?
                    .ok_or(RankWriteError::NotFound)?;
                let changed = update(
                    unit_memberships::table
                        .find(member_id.as_uuid())
                        .filter(unit_memberships::unit_id.eq(unit_id.as_uuid())),
                )
                .set((
                    unit_memberships::rank_id.eq(rank_id.as_uuid()),
                    unit_memberships::updated_at.eq(chrono::Utc::now()),
                ))
                .execute(conn)
                .await?;
                if changed == 0 {
                    return Err(RankWriteError::NotFound);
                }
                Ok(())
            })
            .await
    }

    async fn create_managed_rank(
        &self,
        actor_id: UserId,
        rank: NewUnitRank,
    ) -> Result<UnitRank, RankWriteError> {
        self.conn()
            .await
            .map_err(StoreError::from)?
            .transaction::<_, RankWriteError, _>(async move |conn| {
                authorize(conn, actor_id, rank.unit_id, Permission::ManageRanks).await?;
                Ok(insert_at_bottom(conn, rank).await?)
            })
            .await
    }

    async fn patch_managed_rank(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        rank_id: RankId,
        patch: UnitRankPatch,
    ) -> Result<UnitRank, RankWriteError> {
        self.conn()
            .await
            .map_err(StoreError::from)?
            .transaction::<_, RankWriteError, _>(async move |conn| {
                authorize(conn, actor_id, unit_id, Permission::ManageRanks).await?;
                update(
                    unit_ranks::table
                        .find(rank_id.as_uuid())
                        .filter(unit_ranks::unit_id.eq(unit_id.as_uuid())),
                )
                .set((&patch, unit_ranks::updated_at.eq(chrono::Utc::now())))
                .get_result(conn)
                .await
                .optional()?
                .ok_or(RankWriteError::NotFound)
            })
            .await
    }

    async fn delete_managed_rank(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        rank_id: RankId,
    ) -> Result<(), RankWriteError> {
        self.conn()
            .await
            .map_err(StoreError::from)?
            .transaction::<_, RankWriteError, _>(async move |conn| {
                authorize(conn, actor_id, unit_id, Permission::ManageRanks).await?;
                unit_ranks::table
                    .find(rank_id.as_uuid())
                    .filter(unit_ranks::unit_id.eq(unit_id.as_uuid()))
                    .first::<UnitRank>(conn)
                    .await
                    .optional()?
                    .ok_or(RankWriteError::NotFound)?;
                let assigned = unit_memberships::table
                    .filter(unit_memberships::rank_id.eq(rank_id.as_uuid()))
                    .count()
                    .get_result::<i64>(conn)
                    .await?;
                let initial = unit_settings::table
                    .filter(unit_settings::initial_rank_id.eq(rank_id.as_uuid()))
                    .count()
                    .get_result::<i64>(conn)
                    .await?;
                if assigned > 0 || initial > 0 {
                    return Err(RankWriteError::InUse);
                }
                delete(
                    unit_ranks::table
                        .find(rank_id.as_uuid())
                        .filter(unit_ranks::unit_id.eq(unit_id.as_uuid())),
                )
                .execute(conn)
                .await?;
                Ok(())
            })
            .await
    }

    async fn reorder_managed_ranks(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        rank_ids: Vec<RankId>,
    ) -> Result<Vec<UnitRank>, RankWriteError> {
        self.conn()
            .await
            .map_err(StoreError::from)?
            .transaction::<_, RankWriteError, _>(async move |conn| {
                authorize(conn, actor_id, unit_id, Permission::ManageRanks).await?;
                let ranks = unit_ranks::table
                    .filter(unit_ranks::unit_id.eq(unit_id.as_uuid()))
                    .load::<UnitRank>(conn)
                    .await?;
                let expected: HashSet<_> = ranks.iter().map(|rank| rank.id).collect();
                let actual: HashSet<_> = rank_ids.iter().copied().collect();
                if rank_ids.len() != ranks.len() || expected != actual {
                    return Err(RankWriteError::InvalidOrder);
                }
                for (index, id) in rank_ids.into_iter().enumerate() {
                    let position =
                        i64::try_from(index).map_err(|_overflow| RankWriteError::InvalidOrder)?;
                    update(
                        unit_ranks::table
                            .find(id.as_uuid())
                            .filter(unit_ranks::unit_id.eq(unit_id.as_uuid())),
                    )
                    .set((
                        unit_ranks::position.eq(position),
                        unit_ranks::updated_at.eq(chrono::Utc::now()),
                    ))
                    .execute(conn)
                    .await?;
                }
                Ok(unit_ranks::table
                    .filter(unit_ranks::unit_id.eq(unit_id.as_uuid()))
                    .order(unit_ranks::position.desc())
                    .load(conn)
                    .await?)
            })
            .await
    }
}
