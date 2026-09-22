use async_trait::async_trait;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, delete,
    dsl::insert_into, update,
};
use diesel_async::{AsyncConnection, RunQueryDsl};
use tactica_db_model::{
    ListPagination, NewUnitRole, StoreError, UnitRole, UnitRoleFilter, UnitRolePatch,
    UnitRoleStore, role_kind,
};
use tactica_db_schema::schema::{unit_member_roles, unit_memberships, unit_roles, units};
use tactica_uuid_kinds::{MemberId, RoleId, UnitId};

use crate::PgConnection;

#[async_trait]
impl UnitRoleStore for PgConnection {
    async fn list_for_member(
        &self,
        unit_id: UnitId,
        member_id: MemberId,
        pagination: &ListPagination,
    ) -> Result<Vec<UnitRole>, StoreError> {
        let mut conn = self.conn().await?;
        let member_exists = diesel::select(diesel::dsl::exists(
            unit_memberships::table
                .filter(unit_memberships::id.eq(member_id.as_uuid()))
                .filter(unit_memberships::unit_id.eq(unit_id.as_uuid())),
        ))
        .get_result::<bool>(&mut conn)
        .await?;
        if !member_exists {
            return Ok(Vec::new());
        }
        let ids = unit_member_roles::table
            .filter(unit_member_roles::member_id.eq(member_id.as_uuid()))
            .filter(unit_member_roles::unit_id.eq(unit_id.as_uuid()))
            .select(unit_member_roles::role_id)
            .load::<RoleId>(&mut conn)
            .await?;
        Ok(unit_roles::table
            .filter(unit_roles::unit_id.eq(unit_id.as_uuid()))
            .filter(
                unit_roles::kind
                    .eq(role_kind::EVERYONE)
                    .or(unit_roles::id.eq_any(ids.iter().map(RoleId::as_uuid))),
            )
            .order(unit_roles::id)
            .offset(pagination.offset.0)
            .limit(pagination.limit.0)
            .load(&mut conn)
            .await?)
    }

    async fn patch_in_unit(
        &self,
        unit_id: UnitId,
        id: RoleId,
        patch: UnitRolePatch,
    ) -> Result<Option<UnitRole>, StoreError> {
        update(
            unit_roles::table
                .filter(unit_roles::id.eq(id.as_uuid()))
                .filter(unit_roles::unit_id.eq(unit_id.as_uuid()))
                .filter(unit_roles::kind.ne(role_kind::ADMINISTRATOR)),
        )
        .set((&patch, unit_roles::updated_at.eq(chrono::Utc::now())))
        .get_result(&mut self.conn().await?)
        .await
        .optional()
        .map_err(Into::into)
    }

    async fn delete_in_unit(&self, unit_id: UnitId, id: RoleId) -> Result<bool, StoreError> {
        let deleted = delete(
            unit_roles::table
                .filter(unit_roles::id.eq(id.as_uuid()))
                .filter(unit_roles::unit_id.eq(unit_id.as_uuid()))
                .filter(unit_roles::kind.eq(role_kind::CUSTOM)),
        )
        .execute(&mut self.conn().await?)
        .await?;
        Ok(deleted != 0)
    }

    async fn list(
        &self,
        filter: UnitRoleFilter,
        pagination: &ListPagination,
    ) -> Result<Vec<UnitRole>, StoreError> {
        let mut query = unit_roles::table.into_boxed();
        let UnitRoleFilter {
            id,
            unit_id,
            display_name,
        } = filter;

        if let Some(id) = id {
            query = query.filter(unit_roles::id.eq_any(id.into_iter().map(|id| *id.as_uuid())));
        }
        if let Some(unit_id) = unit_id {
            query = query
                .filter(unit_roles::unit_id.eq_any(unit_id.into_iter().map(|id| *id.as_uuid())));
        }
        if let Some(display_name) = display_name {
            query = query.filter(unit_roles::display_name.eq_any(display_name));
        }

        Ok(query
            .order((
                unit_roles::unit_id,
                unit_roles::position.desc(),
                unit_roles::id,
            ))
            .offset(pagination.offset.0)
            .limit(pagination.limit.0)
            .get_results(&mut self.conn().await?)
            .await?)
    }

    async fn get(&self, id: RoleId) -> Result<Option<UnitRole>, StoreError> {
        Ok(Self::list(
            self,
            UnitRoleFilter::default().id(vec![id]),
            &ListPagination::latest(),
        )
        .await?
        .into_iter()
        .next())
    }

    async fn get_by_unit_and_display_name(
        &self,
        unit_id: UnitId,
        display_name: &str,
    ) -> Result<Option<UnitRole>, StoreError> {
        Ok(Self::list(
            self,
            UnitRoleFilter::default()
                .unit_id(vec![unit_id])
                .display_name(vec![display_name.to_owned()]),
            &ListPagination::latest(),
        )
        .await?
        .into_iter()
        .next())
    }

    async fn create(&self, role: NewUnitRole) -> Result<UnitRole, StoreError> {
        self.conn()
            .await?
            .transaction::<_, StoreError, _>(async move |conn| {
                // Raw creation shares the ordering lock with API role management.
                units::table
                    .find(role.unit_id.as_uuid())
                    .for_update()
                    .select(units::id)
                    .first::<UnitId>(conn)
                    .await?;
                insert_at_bottom(conn, role).await
            })
            .await
    }

    async fn update(&self, role: UnitRole) -> Result<UnitRole, StoreError> {
        update(
            unit_roles::table
                .filter(unit_roles::id.eq(role.id.as_uuid()))
                .filter(unit_roles::kind.ne(role_kind::ADMINISTRATOR)),
        )
        .set((
            unit_roles::unit_id.eq(role.unit_id.as_uuid()),
            unit_roles::display_name.eq(role.display_name),
            unit_roles::description.eq(role.description),
            unit_roles::permissions.eq(role.permissions),
        ))
        .get_result(&mut self.conn().await?)
        .await
        .map_err(Into::into)
    }

    async fn delete(&self, id: RoleId) -> Result<(), StoreError> {
        delete(
            unit_roles::table
                .filter(unit_roles::id.eq(id.as_uuid()))
                .filter(unit_roles::kind.eq(role_kind::CUSTOM)),
        )
        .execute(&mut self.conn().await?)
        .await?;
        Ok(())
    }
}

/// Caller must hold the unit row lock. Position uniqueness is deferred until commit.
pub async fn insert_at_bottom(
    conn: &mut diesel_async::AsyncPgConnection,
    role: NewUnitRole,
) -> Result<UnitRole, StoreError> {
    update(
        unit_roles::table
            .filter(unit_roles::unit_id.eq(role.unit_id.as_uuid()))
            .filter(unit_roles::kind.ne(role_kind::EVERYONE)),
    )
    .set(unit_roles::position.eq(unit_roles::position + 1_i64))
    .execute(conn)
    .await?;
    insert_into(unit_roles::table)
        .values((
            unit_roles::id.eq(role.id.as_uuid()),
            unit_roles::unit_id.eq(role.unit_id.as_uuid()),
            unit_roles::display_name.eq(role.display_name),
            unit_roles::description.eq(role.description),
            unit_roles::permissions.eq(role.permissions),
            unit_roles::position.eq(1_i64),
        ))
        .get_result(conn)
        .await
        .map_err(Into::into)
}
