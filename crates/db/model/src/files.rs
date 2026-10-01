use crate::{ListPagination, RoleWriteError, StoreError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::{Insertable, Queryable, Selectable};
use tactica_db_schema::schema::files;
use tactica_uuid_kinds::{FileId, UnitId, UserId};

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = files)]
pub struct File {
    pub id: FileId,
    pub unit_id: UnitId,
    pub uploaded_by: UserId,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub is_icon: bool,
    pub created_at: DateTime<Utc>,
    pub is_banner: bool,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = files)]
pub struct NewFile {
    pub id: FileId,
    pub unit_id: UnitId,
    pub uploaded_by: UserId,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub is_icon: bool,
    pub is_banner: bool,
}

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait FileStore {
    async fn authorize_file_upload(
        &self,
        actor: UserId,
        unit: UnitId,
        is_artwork: bool,
    ) -> Result<(), RoleWriteError>;
    /// Authorizes membership (`ManageUnit` for profile artwork), inserts metadata and, for
    /// artwork, changes the corresponding unit URL in one transaction under the unit lock.
    async fn create_file(&self, file: NewFile) -> Result<File, RoleWriteError>;
    async fn get_file(&self, unit_id: UnitId, id: FileId) -> Result<Option<File>, StoreError>;
    async fn list_files(
        &self,
        unit_id: UnitId,
        pagination: &ListPagination,
    ) -> Result<Vec<File>, StoreError>;
    /// An attachment's uploader or a unit manager may delete it. Profile artwork requires
    /// `ManageUnit`. Returns the removed metadata for object cleanup.
    async fn delete_file(
        &self,
        actor: UserId,
        unit_id: UnitId,
        id: FileId,
    ) -> Result<File, RoleWriteError>;
}
