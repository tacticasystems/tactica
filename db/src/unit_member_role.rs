use async_trait::async_trait;
use diesel::{ExpressionMethods, QueryDsl, delete, dsl::insert_into};
use diesel_async::RunQueryDsl;
use tactica_db_model::{
    ListPagination, NewUnitMemberRole, StoreError, UnitMemberRole, UnitMemberRoleFilter,
    UnitMemberRoleStore,
};
use tactica_db_schema::schema::unit_member_roles;
use tactica_uuid_kinds::{MemberId, RoleId};

use crate::PgConnection;

#[async_trait]
impl UnitMemberRoleStore for PgConnection {
    async fn assign(&self, assignment: NewUnitMemberRole) -> Result<(), StoreError> {
        insert_into(unit_member_roles::table)
            .values((
                unit_member_roles::member_id.eq(assignment.member_id.as_uuid()),
                unit_member_roles::role_id.eq(assignment.role_id.as_uuid()),
                unit_member_roles::unit_id.eq(assignment.unit_id.as_uuid()),
            ))
            .on_conflict((unit_member_roles::member_id, unit_member_roles::role_id))
            .do_nothing()
            .execute(&mut self.conn().await?)
            .await?;
        Ok(())
    }

    async fn list(
        &self,
        filter: UnitMemberRoleFilter,
        pagination: &ListPagination,
    ) -> Result<Vec<UnitMemberRole>, StoreError> {
        let mut query = unit_member_roles::table.into_boxed();
        let UnitMemberRoleFilter {
            member_id,
            role_id,
            unit_id,
        } = filter;

        if let Some(member_id) = member_id {
            query = query.filter(
                unit_member_roles::member_id.eq_any(member_id.into_iter().map(|id| *id.as_uuid())),
            );
        }
        if let Some(role_id) = role_id {
            query = query.filter(
                unit_member_roles::role_id.eq_any(role_id.into_iter().map(|id| *id.as_uuid())),
            );
        }
        if let Some(unit_id) = unit_id {
            query = query.filter(
                unit_member_roles::unit_id.eq_any(unit_id.into_iter().map(|id| *id.as_uuid())),
            );
        }

        Ok(query
            .order((unit_member_roles::member_id, unit_member_roles::role_id))
            .offset(pagination.offset.0)
            .limit(pagination.limit.0)
            .get_results(&mut self.conn().await?)
            .await?)
    }

    async fn create(&self, assignment: NewUnitMemberRole) -> Result<UnitMemberRole, StoreError> {
        insert_into(unit_member_roles::table)
            .values((
                unit_member_roles::member_id.eq(assignment.member_id.as_uuid()),
                unit_member_roles::role_id.eq(assignment.role_id.as_uuid()),
                unit_member_roles::unit_id.eq(assignment.unit_id.as_uuid()),
            ))
            .get_result(&mut self.conn().await?)
            .await
            .map_err(Into::into)
    }

    async fn delete(&self, member_id: MemberId, role_id: RoleId) -> Result<(), StoreError> {
        delete(
            unit_member_roles::table
                .filter(unit_member_roles::member_id.eq(member_id.as_uuid()))
                .filter(unit_member_roles::role_id.eq(role_id.as_uuid())),
        )
        .execute(&mut self.conn().await?)
        .await?;
        Ok(())
    }
}
