use serde::{Deserialize, Serialize};
use tactica_uuid_kinds::{MemberId, RankId, RoleId, UnitId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct MemberSummary {
    pub id: MemberId,
    pub user_id: UserId,
    pub unit_id: UnitId,
    pub rank_id: RankId,
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
