use crate::{PgConnection, unit_role_management::authorize};
use async_trait::async_trait;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, delete, insert_into, update};
use diesel_async::{AsyncConnection, RunQueryDsl};
use std::collections::HashSet;
use tactica_db_model::{
    MemberPatch, RoleWriteError, StoreError, UnitMemberManagementStore, UnitMembership, UnitRole,
    role_kind,
};
use tactica_db_schema::schema::{unit_member_roles, unit_memberships, unit_ranks, unit_roles};
use tactica_permissions::Permission;
use tactica_uuid_kinds::{MemberId, RankId, RoleId, UnitId, UserId};

#[async_trait]
impl UnitMemberManagementStore for PgConnection {
    #[expect(
        clippy::too_many_lines,
        reason = "Validate all fields before mutating the member in one transaction"
    )]
    async fn patch_managed_member(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        member_id: MemberId,
        patch: MemberPatch,
    ) -> Result<(), RoleWriteError> {
        if patch.display_name.is_none() && patch.rank_id.is_none() && patch.role_ids.is_none() {
            return Err(RoleWriteError::Forbidden);
        }
        self.conn()
            .await
            .map_err(StoreError::from)?
            .transaction::<_, RoleWriteError, _>(async move |conn| {
                // Every permission is checked before role changes can affect the actor's own access.
                if patch.display_name.is_some() {
                    authorize(conn, actor_id, unit_id, Permission::ManageMembers).await?;
                }
                if patch.rank_id.is_some() {
                    authorize(conn, actor_id, unit_id, Permission::AssignRanks).await?;
                }
                let role_actor = if patch.role_ids.is_some() {
                    Some(authorize(conn, actor_id, unit_id, Permission::AssignRoles).await?)
                } else {
                    None
                };
                let member = unit_memberships::table
                    .find(member_id.as_uuid())
                    .filter(unit_memberships::unit_id.eq(unit_id.as_uuid()))
                    .first::<UnitMembership>(conn)
                    .await
                    .optional()?
                    .ok_or(RoleWriteError::NotFound)?;
                if let Some(rank_id) = patch.rank_id {
                    unit_ranks::table
                        .find(rank_id.as_uuid())
                        .filter(unit_ranks::unit_id.eq(unit_id.as_uuid()))
                        .select(unit_ranks::id)
                        .first::<RankId>(conn)
                        .await
                        .optional()?
                        .ok_or(RoleWriteError::NotFound)?;
                }
                let mut added = HashSet::new();
                let mut removed = HashSet::new();
                if let (Some(ids), Some(actor)) = (&patch.role_ids, &role_actor) {
                    let wanted: HashSet<_> = ids.iter().copied().collect();
                    let roles = unit_roles::table
                        .filter(unit_roles::unit_id.eq(unit_id.as_uuid()))
                        .load::<UnitRole>(conn)
                        .await?;
                    for id in &wanted {
                        let role = roles
                            .iter()
                            .find(|role| role.id == *id)
                            .ok_or(RoleWriteError::NotFound)?;
                        if role.kind == role_kind::EVERYONE {
                            return Err(RoleWriteError::ProtectedRole);
                        }
                    }
                    let current: HashSet<_> = unit_member_roles::table
                        .filter(unit_member_roles::unit_id.eq(unit_id.as_uuid()))
                        .filter(unit_member_roles::member_id.eq(member_id.as_uuid()))
                        .select(unit_member_roles::role_id)
                        .load::<RoleId>(conn)
                        .await?
                        .into_iter()
                        .collect();
                    added = wanted.difference(&current).copied().collect();
                    removed = current.difference(&wanted).copied().collect();
                    for id in added.union(&removed) {
                        let role = roles
                            .iter()
                            .find(|role| role.id == *id)
                            .ok_or(RoleWriteError::NotFound)?;
                        if role.kind == role_kind::EVERYONE {
                            return Err(RoleWriteError::ProtectedRole);
                        }
                        actor.require_below(role.position)?;
                    }
                }
                update(unit_memberships::table.find(member_id.as_uuid()))
                    .set((
                        unit_memberships::display_name
                            .eq(patch.display_name.unwrap_or(member.display_name)),
                        unit_memberships::rank_id
                            .eq(patch.rank_id.unwrap_or(member.rank_id).as_uuid()),
                        unit_memberships::updated_at.eq(chrono::Utc::now()),
                    ))
                    .execute(conn)
                    .await?;
                for role_id in removed {
                    delete(
                        unit_member_roles::table
                            .filter(unit_member_roles::unit_id.eq(unit_id.as_uuid()))
                            .filter(unit_member_roles::member_id.eq(member_id.as_uuid()))
                            .filter(unit_member_roles::role_id.eq(role_id.as_uuid())),
                    )
                    .execute(conn)
                    .await?;
                }
                for role_id in added {
                    insert_into(unit_member_roles::table)
                        .values((
                            unit_member_roles::unit_id.eq(unit_id.as_uuid()),
                            unit_member_roles::member_id.eq(member_id.as_uuid()),
                            unit_member_roles::role_id.eq(role_id.as_uuid()),
                        ))
                        .on_conflict_do_nothing()
                        .execute(conn)
                        .await?;
                }
                Ok(())
            })
            .await
    }
}
