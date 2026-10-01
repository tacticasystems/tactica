use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tactica_uuid_kinds::{FileId, UnitId, UserId};

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct FileSummary {
    pub id: FileId,
    pub unit_id: UnitId,
    pub uploaded_by: UserId,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ListFilesResponse {
    pub files: Vec<FileSummary>,
}
