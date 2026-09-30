use serde::{Deserialize, Serialize};
use tactica_uuid_kinds::{MemberId, RoleId, UnitId};

pub use tactica_permissions::{Permission, Permissions};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct RoleSummary {
    pub id: RoleId,
    pub unit_id: UnitId,
    pub display_name: String,
    pub description: Option<String>,
    pub permissions: i64,
    pub position: i64,
    /// custom, administrator, or everyone; built-in identity is independent of the name.
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ListRolesResponse {
    pub roles: Vec<RoleSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ListRoleMembersResponse {
    /// Membership IDs; Everyone includes all current unit members.
    pub member_ids: Vec<MemberId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ReorderRolesRequest {
    /// Every role in the unit, once each, ordered from lowest to highest.
    pub role_ids: Vec<RoleId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CreateRoleRequest {
    pub display_name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub permissions: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct UpdateRoleRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Omitted leaves the description unchanged; null clears it.
    #[serde(
        default,
        deserialize_with = "description_patch",
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<i64>,
}

fn description_patch<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error> {
    Option::<String>::deserialize(deserializer).map(Some)
}
