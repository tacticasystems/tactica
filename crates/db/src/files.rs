use crate::{PgConnection, unit_role_management::require_unit_permission};
use async_trait::async_trait;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use tactica_db_model::{File, FileStore, ListPagination, NewFile, RoleWriteError, StoreError};
use tactica_db_schema::schema::{files, unit_memberships, units, users};
use tactica_permissions::Permission;
use tactica_uuid_kinds::{FileId, UnitId, UserId};

async fn member(
    conn: &mut AsyncPgConnection,
    actor: UserId,
    unit: UnitId,
) -> Result<(), RoleWriteError> {
    let active = users::table
        .find(actor.as_uuid())
        .select(users::is_active)
        .first::<bool>(conn)
        .await
        .optional()?
        .unwrap_or(false);
    if !active {
        return Err(RoleWriteError::Unauthorized);
    }
    units::table
        .find(unit.as_uuid())
        .for_update()
        .select(units::id)
        .first::<UnitId>(conn)
        .await
        .optional()?
        .ok_or(RoleWriteError::NotFound)?;
    let exists = unit_memberships::table
        .filter(unit_memberships::unit_id.eq(unit.as_uuid()))
        .filter(unit_memberships::user_id.eq(actor.as_uuid()))
        .count()
        .get_result::<i64>(conn)
        .await?;
    if exists == 0 {
        return Err(RoleWriteError::Forbidden);
    }
    Ok(())
}

#[async_trait]
impl FileStore for PgConnection {
    async fn authorize_file_upload(
        &self,
        actor: UserId,
        unit: UnitId,
        is_artwork: bool,
    ) -> Result<(), RoleWriteError> {
        self.conn()
            .await
            .map_err(StoreError::from)?
            .transaction::<_, RoleWriteError, _>(async move |conn| {
                member(conn, actor, unit).await?;
                if is_artwork {
                    require_unit_permission(conn, actor, unit, Permission::ManageUnit).await?;
                }
                Ok(())
            })
            .await
    }
    async fn create_file(&self, file: NewFile) -> Result<File, RoleWriteError> {
        self.conn()
            .await
            .map_err(StoreError::from)?
            .transaction::<_, RoleWriteError, _>(async move |conn| {
                member(conn, file.uploaded_by, file.unit_id).await?;
                if file.is_icon || file.is_banner {
                    require_unit_permission(
                        conn,
                        file.uploaded_by,
                        file.unit_id,
                        Permission::ManageUnit,
                    )
                    .await?;
                }
                let file = diesel::insert_into(files::table)
                    .values(file)
                    .get_result::<File>(conn)
                    .await?;
                if file.is_icon {
                    let url = format!("/api/v1/units/{}/icon/{}", file.unit_id, file.id);
                    diesel::update(units::table.find(file.unit_id.as_uuid()))
                        .set((
                            units::icon_url.eq(Some(url)),
                            units::updated_at.eq(chrono::Utc::now()),
                        ))
                        .execute(conn)
                        .await?;
                }
                if file.is_banner {
                    let url = format!("/api/v1/units/{}/banner/{}", file.unit_id, file.id);
                    diesel::update(units::table.find(file.unit_id.as_uuid()))
                        .set((
                            units::banner_url.eq(Some(url)),
                            units::updated_at.eq(chrono::Utc::now()),
                        ))
                        .execute(conn)
                        .await?;
                }
                Ok(file)
            })
            .await
    }
    async fn get_file(&self, unit_id: UnitId, id: FileId) -> Result<Option<File>, StoreError> {
        Ok(files::table
            .find(id.as_uuid())
            .filter(files::unit_id.eq(unit_id.as_uuid()))
            .first(&mut self.conn().await?)
            .await
            .optional()?)
    }
    async fn list_files(
        &self,
        unit_id: UnitId,
        pagination: &ListPagination,
    ) -> Result<Vec<File>, StoreError> {
        Ok(files::table
            .filter(files::unit_id.eq(unit_id.as_uuid()))
            .filter(files::is_icon.eq(false))
            .filter(files::is_banner.eq(false))
            .order(files::id.desc())
            .offset(pagination.offset.0)
            .limit(pagination.limit.0)
            .load(&mut self.conn().await?)
            .await?)
    }
    async fn delete_file(
        &self,
        actor: UserId,
        unit_id: UnitId,
        id: FileId,
    ) -> Result<File, RoleWriteError> {
        self.conn()
            .await
            .map_err(StoreError::from)?
            .transaction::<_, RoleWriteError, _>(async move |conn| {
                member(conn, actor, unit_id).await?;
                let file = files::table
                    .find(id.as_uuid())
                    .filter(files::unit_id.eq(unit_id.as_uuid()))
                    .first::<File>(conn)
                    .await
                    .optional()?
                    .ok_or(RoleWriteError::NotFound)?;
                if file.is_icon || file.is_banner || file.uploaded_by != Some(actor) {
                    require_unit_permission(conn, actor, unit_id, Permission::ManageUnit).await?;
                }
                if file.is_icon {
                    let url = format!("/api/v1/units/{unit_id}/icon/{id}");
                    diesel::update(
                        units::table
                            .find(unit_id.as_uuid())
                            .filter(units::icon_url.eq(Some(url))),
                    )
                    .set((
                        units::icon_url.eq(None::<String>),
                        units::updated_at.eq(chrono::Utc::now()),
                    ))
                    .execute(conn)
                    .await?;
                }
                if file.is_banner {
                    let url = format!("/api/v1/units/{unit_id}/banner/{id}");
                    diesel::update(
                        units::table
                            .find(unit_id.as_uuid())
                            .filter(units::banner_url.eq(Some(url))),
                    )
                    .set((
                        units::banner_url.eq(None::<String>),
                        units::updated_at.eq(chrono::Utc::now()),
                    ))
                    .execute(conn)
                    .await?;
                }
                diesel::delete(files::table.find(id.as_uuid()))
                    .execute(conn)
                    .await?;
                Ok(file)
            })
            .await
    }
}
