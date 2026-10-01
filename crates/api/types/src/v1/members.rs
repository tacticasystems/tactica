use serde::{Deserialize, Serialize};
use tactica_uuid_kinds::{MemberId, RankId, RoleId, UnitId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct MemberSummary {
    pub id: MemberId,
    pub user_id: UserId,
    pub unit_id: UnitId,
    pub rank_id: RankId,
    pub username: String,
    pub display_name: Option<String>,
    /// Unit-specific name override; null inherits the account display name.
    pub unit_display_name: Option<String>,
    pub icon_url: Option<String>,
    /// Roles applying to this member, highest first, including implicit Everyone.
    pub role_ids: Vec<RoleId>,
}

/// Current member's UI capabilities; role writes remain authoritative.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct UnitAccessResponse {
    pub member_id: MemberId,
    pub is_owner: bool,
    /// Effective permissions, including expansion of Administrator.
    pub permissions: i64,
    pub highest_role_position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ListMembersResponse {
    pub members: Vec<MemberSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ListMemberRolesResponse {
    pub role_ids: Vec<RoleId>,
}

/// Replace a member's rank with another rank from the same unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct SetMemberRankRequest {
    pub rank_id: RankId,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct UpdateMemberRequest {
    #[serde(
        default,
        deserialize_with = "name_patch",
        skip_serializing_if = "Option::is_none"
    )]
    pub display_name: Option<Option<String>>,
    pub rank_id: Option<RankId>,
    /// Complete explicit role list; excludes implicit Everyone.
    pub role_ids: Option<Vec<RoleId>>,
}

#[expect(
    clippy::option_option,
    reason = "Distinguish omitted name from an explicit null override"
)]
fn name_patch<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error> {
    Option::<String>::deserialize(deserializer).map(Some)
}
