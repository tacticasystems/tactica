use crate::{NewUnitRank, RoleWriteError, StoreError, UnitRank, UnitRankPatch};
use async_trait::async_trait;
use tactica_uuid_kinds::{MemberId, RankId, UnitId, UserId};

#[derive(Debug, thiserror::Error)]
pub enum RankWriteError {
    #[error(transparent)]
    Authorization(#[from] RoleWriteError),
    #[error("Rank not found in this unit")]
    NotFound,
    #[error("Rank order must contain every unit rank exactly once")]
    InvalidOrder,
    #[error("Rank is assigned to a member or is the initial rank")]
    InUse,
    #[error(transparent)]
    Store(#[from] StoreError),
}

impl From<diesel::result::Error> for RankWriteError {
    fn from(error: diesel::result::Error) -> Self {
        // A concurrent trusted membership/settings write can add a reference
        // after the explicit in-use check; the foreign key remains authoritative.
        if matches!(
            &error,
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                _
            )
        ) {
            Self::InUse
        } else {
            Self::Store(error.into())
        }
    }
}

/// Authorize rank writes with live role permissions and mutate in one transaction
/// under the same unit lock used by role management. Rank position grants no authority.
#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait UnitRankManagementStore {
    /// Assign any rank in the unit using live `AssignRanks` permission, without a rank hierarchy check.
    async fn set_member_rank(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        member_id: MemberId,
        rank_id: RankId,
    ) -> Result<(), RankWriteError>;
    async fn create_managed_rank(
        &self,
        actor_id: UserId,
        rank: NewUnitRank,
    ) -> Result<UnitRank, RankWriteError>;
    async fn patch_managed_rank(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        rank_id: RankId,
        patch: UnitRankPatch,
    ) -> Result<UnitRank, RankWriteError>;
    async fn delete_managed_rank(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        rank_id: RankId,
    ) -> Result<(), RankWriteError>;
    /// Complete order, lowest first. Returns ranks highest first.
    async fn reorder_managed_ranks(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        rank_ids: Vec<RankId>,
    ) -> Result<Vec<UnitRank>, RankWriteError>;
}
