use tactica_auth::principal::Principal;
use tactica_db_model::{
    ListPagination, TacticaStorage, Unit, UnitMembershipStore, UnitStore, UserStore,
};
use tactica_uuid_kinds::UnitId;

use crate::error::{Error, Result};

pub(super) async fn require_unit(storage: &dyn TacticaStorage, unit_id: UnitId) -> Result<Unit> {
    UnitStore::get(storage, unit_id)
        .await?
        .ok_or(Error::NotFound)
}

pub(super) async fn require_unit_member(
    storage: &dyn TacticaStorage,
    principal: Principal,
    unit_id: UnitId,
) -> Result<()> {
    let Principal::User(user_id) = principal else {
        return Err(Error::Forbidden("Unit membership is required".to_owned()));
    };
    let user = UserStore::get(storage, user_id)
        .await?
        .filter(|user| user.is_active)
        .ok_or_else(|| Error::Unauthorized("User is missing or inactive".to_owned()))?;
    require_unit(storage, unit_id).await?;
    if UnitMembershipStore::get_by_user_and_unit(storage, user.id, unit_id)
        .await?
        .is_none()
    {
        return Err(Error::Forbidden("Unit membership is required".to_owned()));
    }
    Ok(())
}

pub(super) fn validate_pagination(pagination: &ListPagination) -> Result<()> {
    if pagination.offset.0 < 0 || !(1..=100).contains(&pagination.limit.0) {
        return Err(Error::Validation(
            "offset must be non-negative and limit must be between 1 and 100".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn require_user(principal: Principal) -> Result<tactica_uuid_kinds::UserId> {
    match principal {
        Principal::User(user_id) => Ok(user_id),
        Principal::Service(_) => Err(Error::Forbidden("A user principal is required".to_owned())),
    }
}
