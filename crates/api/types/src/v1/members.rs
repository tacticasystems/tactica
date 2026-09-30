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
    pub icon_url: Option<String>,
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
