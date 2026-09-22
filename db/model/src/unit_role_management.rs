use async_trait::async_trait;
use tactica_uuid_kinds::{MemberId, RoleId, UnitId, UserId};

use crate::{NewUnitRole, StoreError, UnitRole, UnitRolePatch};

#[cfg(feature = "mock")]
use mockall::automock;

#[derive(Debug, thiserror::Error)]
pub enum RoleWriteError {
    #[error("User is missing or inactive")]
    Unauthorized,
    #[error("Insufficient unit permissions")]
    Forbidden,
    #[error("Unit, role, or member not found")]
    NotFound,
    #[error("Permissions contain undefined bits")]
    InvalidPermissions,
    #[error("Role order must contain every unit role exactly once")]
    InvalidOrder,
    #[error("This operation is not allowed on a built-in role")]
    ProtectedRole,
    #[error(transparent)]
    Store(#[from] StoreError),
}

impl From<diesel::result::Error> for RoleWriteError {
    fn from(error: diesel::result::Error) -> Self {
        Self::Store(error.into())
    }
}

/// Role writes for untrusted callers. Each operation atomically authorizes and
/// mutates under the same unit lock; API handlers must use these operations
/// rather than the raw role/mapping stores.
#[async_trait]
#[cfg_attr(feature = "mock", automock)]
pub trait UnitRoleManagementStore {
    /// Reorders the complete list from lowest to highest; roles at or above the
    /// caller's highest role must remain fixed unless the caller is the owner.
    async fn reorder_managed_roles(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        role_ids: Vec<RoleId>,
    ) -> Result<Vec<UnitRole>, RoleWriteError>;
    async fn create_managed_role(
        &self,
        actor_id: UserId,
        role: NewUnitRole,
    ) -> Result<UnitRole, RoleWriteError>;
    async fn patch_managed_role(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        role_id: RoleId,
        patch: UnitRolePatch,
    ) -> Result<UnitRole, RoleWriteError>;
    async fn delete_managed_role(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        role_id: RoleId,
    ) -> Result<(), RoleWriteError>;
    async fn assign_managed_role(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        member_id: MemberId,
        role_id: RoleId,
    ) -> Result<(), RoleWriteError>;
    async fn remove_managed_role(
        &self,
        actor_id: UserId,
        unit_id: UnitId,
        member_id: MemberId,
        role_id: RoleId,
    ) -> Result<(), RoleWriteError>;
}
