use crate::RoleWriteError;
use async_trait::async_trait;
use tactica_uuid_kinds::{MemberId, RankId, RoleId, UnitId, UserId};

#[derive(Debug, Default)]
pub struct MemberPatch {
    pub display_name: Option<Option<String>>,
    pub rank_id: Option<RankId>,
    pub role_ids: Option<Vec<RoleId>>,
}

/// Atomically authorize and save member profile, rank, and role changes.
#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UnitMemberManagementStore {
    async fn patch_managed_member(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        member_id: MemberId,
        patch: MemberPatch,
    ) -> Result<(), RoleWriteError>;
}
