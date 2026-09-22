use serde::{Deserialize, Serialize};
use tactica_uuid_kinds::{RoleId, UnitId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct RoleSummary {
    pub id: RoleId,
    pub unit_id: UnitId,
    pub display_name: String,
    pub description: Option<String>,
    pub permissions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ListRolesResponse {
    pub roles: Vec<RoleSummary>,
}
