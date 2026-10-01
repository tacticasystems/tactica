use std::collections::HashSet;

use async_trait::async_trait;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, delete, insert_into,
    update,
};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use tactica_db_model::{
    NewUnitRole, RoleWriteError, StoreError, UnitRole, UnitRoleManagementStore, UnitRolePatch,
    role_kind,
};
use tactica_db_schema::schema::{unit_member_roles, unit_memberships, unit_roles, units, users};
use tactica_permissions::{Permission, Permissions};
use tactica_uuid_kinds::{MemberId, RoleId, UnitId, UserId};

use crate::{PgConnection, unit_role::insert_at_bottom};

enum Mutation {
    Create(NewUnitRole),
    Patch(RoleId, UnitRolePatch),
    Delete(RoleId),
    Assign(MemberId, RoleId),
    Remove(MemberId, RoleId),
}

fn permissions(bits: i64) -> Result<Permissions, RoleWriteError> {
    Permissions::from_bits(bits).ok_or(RoleWriteError::InvalidPermissions)
}

pub struct Actor {
    permissions: Permissions,
    highest_position: Option<i64>,
    is_owner: bool,
}

impl Actor {
    pub(crate) fn require_below(&self, position: i64) -> Result<(), RoleWriteError> {
        if self.is_owner
            || self
                .highest_position
                .is_some_and(|highest| position < highest)
        {
            Ok(())
        } else {
            Err(RoleWriteError::Forbidden)
        }
    }

    fn require_grant(&self, mask: i64) -> Result<(), RoleWriteError> {
        if self.is_owner || self.permissions.covers(permissions(mask)?) {
            Ok(())
        } else {
            Err(RoleWriteError::Forbidden)
        }
    }
}

pub async fn authorize(
    conn: &mut AsyncPgConnection,
    actor_id: UserId,
    unit_id: UnitId,
    required: Permission,
) -> Result<Actor, RoleWriteError> {
    let active = users::table
        .find(actor_id.as_uuid())
        .select(users::is_active)
        .first::<bool>(conn)
        .await
        .optional()?
        .unwrap_or(false);
    if !active {
        return Err(RoleWriteError::Unauthorized);
    }
    // Every API role and rank mutation uses this lock, including changes to the actor's
    // roles. Permission checks and writes therefore cannot race each other.
    let owner_id = units::table
        .find(unit_id.as_uuid())
        .for_update()
        .select(units::owner_id)
        .first::<UserId>(conn)
        .await
        .optional()?
        .ok_or(RoleWriteError::NotFound)?;
    let member_id = unit_memberships::table
        .filter(unit_memberships::unit_id.eq(unit_id.as_uuid()))
        .filter(unit_memberships::user_id.eq(actor_id.as_uuid()))
        .select(unit_memberships::id)
        .first::<MemberId>(conn)
        .await
        .optional()?
        .ok_or(RoleWriteError::Forbidden)?;
    if owner_id == actor_id {
        return Ok(Actor {
            permissions: Permissions::ADMINISTRATOR,
            highest_position: None,
            is_owner: true,
        });
    }
    let role_ids = unit_member_roles::table
        .filter(unit_member_roles::unit_id.eq(unit_id.as_uuid()))
        .filter(unit_member_roles::member_id.eq(member_id.as_uuid()))
        .select(unit_member_roles::role_id)
        .load::<RoleId>(conn)
        .await?;
    // Intentionally unpaginated: every assigned role contributes permissions.
    let masks = unit_roles::table
        .filter(unit_roles::unit_id.eq(unit_id.as_uuid()))
        .filter(
            unit_roles::kind
                .eq(role_kind::EVERYONE)
                .or(unit_roles::id.eq_any(role_ids.iter().map(RoleId::as_uuid))),
        )
        .select((unit_roles::permissions, unit_roles::position))
        .load::<(i64, i64)>(conn)
        .await?;
    let mut effective = Permissions::default();
    let mut highest_position = None;
    for (mask, position) in masks {
        effective = effective.union(permissions(mask)?);
        highest_position =
            Some(highest_position.map_or(position, |highest: i64| highest.max(position)));
    }
    if !effective.allows(required) {
        return Err(RoleWriteError::Forbidden);
    }
    Ok(Actor {
        permissions: effective,
        highest_position,
        is_owner: false,
    })
}

async fn target_role(
    conn: &mut AsyncPgConnection,
    unit_id: UnitId,
    role_id: RoleId,
    actor: &Actor,
) -> Result<UnitRole, RoleWriteError> {
    let role = unit_roles::table
        .find(role_id.as_uuid())
        .filter(unit_roles::unit_id.eq(unit_id.as_uuid()))
        .first::<UnitRole>(conn)
        .await
        .optional()?
        .ok_or(RoleWriteError::NotFound)?;
    actor.require_below(role.position)?;
    Ok(role)
}

async fn mutate(
    conn: &mut AsyncPgConnection,
    actor_id: UserId,
    unit_id: UnitId,
    mutation: Mutation,
) -> Result<Option<UnitRole>, RoleWriteError> {
    let required = match &mutation {
        Mutation::Assign(..) | Mutation::Remove(..) => Permission::AssignRoles,
        Mutation::Create(..) | Mutation::Patch(..) | Mutation::Delete(..) => {
            Permission::ManageRoles
        }
    };
    let actor = authorize(conn, actor_id, unit_id, required).await?;
    match mutation {
        Mutation::Create(role) => {
            permissions(role.permissions)?;
            actor.require_grant(role.permissions)?;
            // Everyone is pinned at zero; a member with only Everyone cannot
            // create a role above their own ceiling, even with ManageRoles.
            actor.require_below(0)?;
            // Insert just above Everyone, below every explicitly assigned role.
            let role = insert_at_bottom(conn, role).await?;
            Ok(Some(role))
        }
        Mutation::Patch(role_id, patch) => {
            let old = target_role(conn, unit_id, role_id, &actor).await?;
            if old.kind == role_kind::ADMINISTRATOR
                || (old.kind == role_kind::EVERYONE
                    && (patch.display_name.is_some() || patch.description.is_some()))
            {
                return Err(RoleWriteError::ProtectedRole);
            }
            if let Some(mask) = patch.permissions {
                permissions(mask)?;
                actor.require_grant(mask & !old.permissions)?;
            }
            let role = update(
                unit_roles::table
                    .find(role_id.as_uuid())
                    .filter(unit_roles::unit_id.eq(unit_id.as_uuid())),
            )
            .set((&patch, unit_roles::updated_at.eq(chrono::Utc::now())))
            .get_result(conn)
            .await?;
            Ok(Some(role))
        }
        Mutation::Delete(role_id) => {
            let role = target_role(conn, unit_id, role_id, &actor).await?;
            if role.kind != role_kind::CUSTOM {
                return Err(RoleWriteError::ProtectedRole);
            }
            delete(
                unit_roles::table
                    .find(role_id.as_uuid())
                    .filter(unit_roles::unit_id.eq(unit_id.as_uuid())),
            )
            .execute(conn)
            .await?;
            Ok(None)
        }
        Mutation::Assign(member_id, role_id) | Mutation::Remove(member_id, role_id) => {
            let exists = unit_memberships::table
                .find(member_id.as_uuid())
                .filter(unit_memberships::unit_id.eq(unit_id.as_uuid()))
                .select(unit_memberships::id)
                .first::<MemberId>(conn)
                .await
                .optional()?;
            if exists.is_none() {
                return Err(RoleWriteError::NotFound);
            }
            let role = target_role(conn, unit_id, role_id, &actor).await?;
            if role.kind == role_kind::EVERYONE {
                return Err(RoleWriteError::ProtectedRole);
            }
            if matches!(mutation, Mutation::Assign(..)) {
                insert_into(unit_member_roles::table)
                    .values((
                        unit_member_roles::member_id.eq(member_id.as_uuid()),
                        unit_member_roles::role_id.eq(role_id.as_uuid()),
                        unit_member_roles::unit_id.eq(unit_id.as_uuid()),
                    ))
                    .on_conflict((unit_member_roles::member_id, unit_member_roles::role_id))
                    .do_nothing()
                    .execute(conn)
                    .await?;
            } else {
                delete(
                    unit_member_roles::table
                        .filter(unit_member_roles::unit_id.eq(unit_id.as_uuid()))
                        .filter(unit_member_roles::member_id.eq(member_id.as_uuid()))
                        .filter(unit_member_roles::role_id.eq(role_id.as_uuid())),
                )
                .execute(conn)
                .await?;
            }
            Ok(None)
        }
    }
}

impl PgConnection {
    async fn mutate_role(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        mutation: Mutation,
    ) -> Result<Option<UnitRole>, RoleWriteError> {
        self.conn()
            .await
            .map_err(StoreError::from)?
            .transaction::<_, RoleWriteError, _>(async move |conn| {
                mutate(conn, actor_id, unit_id, mutation).await
            })
            .await
    }
}

fn returned_role(role: Option<UnitRole>) -> Result<UnitRole, RoleWriteError> {
    role.ok_or_else(|| {
        StoreError::InvariantFailed("Role mutation did not return a role".to_owned()).into()
    })
}

#[async_trait]
impl UnitRoleManagementStore for PgConnection {
    async fn reorder_managed_roles(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        role_ids: Vec<RoleId>,
    ) -> Result<Vec<UnitRole>, RoleWriteError> {
        self.conn()
            .await
            .map_err(StoreError::from)?
            .transaction::<_, RoleWriteError, _>(async move |conn| {
                let actor = authorize(conn, actor_id, unit_id, Permission::ManageRoles).await?;
                let roles = unit_roles::table
                    .filter(unit_roles::unit_id.eq(unit_id.as_uuid()))
                    .order(unit_roles::position)
                    .load::<UnitRole>(conn)
                    .await?;
                let expected: HashSet<_> = roles.iter().map(|role| role.id).collect();
                let actual: HashSet<_> = role_ids.iter().copied().collect();
                if role_ids.len() != roles.len() || expected != actual {
                    return Err(RoleWriteError::InvalidOrder);
                }
                if roles
                    .iter()
                    .find(|role| role.kind == role_kind::EVERYONE)
                    .map(|role| &role.id)
                    != role_ids.as_slice().first()
                    || roles
                        .iter()
                        .find(|role| role.kind == role_kind::ADMINISTRATOR)
                        .map(|role| &role.id)
                        != role_ids.last()
                {
                    return Err(RoleWriteError::ProtectedRole);
                }
                // The protected suffix cannot move. This also prevents a lower
                // role from crossing above the caller's highest assigned role.
                for (role, requested) in roles.iter().zip(&role_ids) {
                    if actor.require_below(role.position).is_err() && role.id != *requested {
                        return Err(RoleWriteError::Forbidden);
                    }
                }
                for (index, role_id) in role_ids.into_iter().enumerate() {
                    let position =
                        i64::try_from(index).map_err(|_overflow| RoleWriteError::InvalidOrder)?;
                    update(
                        unit_roles::table
                            .find(role_id.as_uuid())
                            .filter(unit_roles::unit_id.eq(unit_id.as_uuid())),
                    )
                    .set((
                        unit_roles::position.eq(position),
                        unit_roles::updated_at.eq(chrono::Utc::now()),
                    ))
                    .execute(conn)
                    .await?;
                }
                Ok(unit_roles::table
                    .filter(unit_roles::unit_id.eq(unit_id.as_uuid()))
                    .order(unit_roles::position.desc())
                    .load(conn)
                    .await?)
            })
            .await
    }
    async fn create_managed_role(
        &self,
        actor_id: UserId,
        role: NewUnitRole,
    ) -> Result<UnitRole, RoleWriteError> {
        returned_role(
            self.mutate_role(actor_id, role.unit_id, Mutation::Create(role))
                .await?,
        )
    }
    async fn patch_managed_role(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        role_id: RoleId,
        patch: UnitRolePatch,
    ) -> Result<UnitRole, RoleWriteError> {
        returned_role(
            self.mutate_role(actor_id, unit_id, Mutation::Patch(role_id, patch))
                .await?,
        )
    }
    async fn delete_managed_role(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        role_id: RoleId,
    ) -> Result<(), RoleWriteError> {
        self.mutate_role(actor_id, unit_id, Mutation::Delete(role_id))
            .await?;
        Ok(())
    }
    async fn assign_managed_role(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        member_id: MemberId,
        role_id: RoleId,
    ) -> Result<(), RoleWriteError> {
        self.mutate_role(actor_id, unit_id, Mutation::Assign(member_id, role_id))
            .await?;
        Ok(())
    }
    async fn remove_managed_role(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        member_id: MemberId,
        role_id: RoleId,
    ) -> Result<(), RoleWriteError> {
        self.mutate_role(actor_id, unit_id, Mutation::Remove(member_id, role_id))
            .await?;
        Ok(())
    }
}
