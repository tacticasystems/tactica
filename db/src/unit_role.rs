use async_trait::async_trait;
use diesel::{ExpressionMethods, QueryDsl, delete, dsl::insert_into, update};
use diesel_async::RunQueryDsl;
use tactica_db_model::{
    ListPagination, NewUnitRole, StoreError, UnitRole, UnitRoleFilter, UnitRoleStore,
};
use tactica_db_schema::schema::unit_roles;
use tactica_uuid_kinds::{RoleId, UnitId};

use crate::PgConnection;

#[async_trait]
impl UnitRoleStore for PgConnection {
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
            .offset(pagination.offset)
            .limit(pagination.limit)
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
        insert_into(unit_roles::table)
            .values((
                unit_roles::id.eq(role.id.as_uuid()),
                unit_roles::unit_id.eq(role.unit_id.as_uuid()),
                unit_roles::display_name.eq(role.display_name),
                unit_roles::description.eq(role.description),
                unit_roles::permissions.eq(role.permissions),
            ))
            .get_result(&mut self.conn().await?)
            .await
            .map_err(Into::into)
    }

    async fn update(&self, role: UnitRole) -> Result<UnitRole, StoreError> {
        update(unit_roles::table.filter(unit_roles::id.eq(role.id.as_uuid())))
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
        delete(unit_roles::table.filter(unit_roles::id.eq(id.as_uuid())))
            .execute(&mut self.conn().await?)
            .await?;
        Ok(())
    }
}
