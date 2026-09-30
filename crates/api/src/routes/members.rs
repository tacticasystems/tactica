use axum::{
    Json, Router,
    extract::{Path, Query},
    routing::get,
};
use tactica_api_types::v1::members::{
    ListMemberRolesResponse, ListMembersResponse, MemberSummary, UnitAccessResponse,
};
use tactica_db_model::{
    ListPagination, UnitMembershipFilter, UnitMembershipStore, UnitRoleStore, UserFilter, UserStore,
};
use tactica_permissions::{Permission, Permissions};
use tactica_uuid_kinds::{MemberId, UnitId};

use super::common::{require_unit_member, validate_pagination};
use crate::{
    error::{Error, Result},
    state::{ApiState, Principal, Storage},
};

pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/api/v1/units/{unit_id}/members", get(list_members))
        .route("/api/v1/units/{unit_id}/access", get(unit_access))
        .route(
            "/api/v1/units/{unit_id}/members/{member_id}/roles",
            get(list_member_roles),
        )
}

#[utoipa::path(
    get,
    path = "/api/v1/units/{unit_id}/members",
    params(
        ("unit_id" = UnitId, Path, description = "Unit ID"),
        ("offset" = Option<i64>, Query, description = "Non-negative offset; defaults to 0"),
        ("limit" = Option<i64>, Query, description = "Page size from 1 to 100; defaults to 10"),
    ),
    responses(
        (status = 200, description = "Unit roster", body = ListMembersResponse),
        (status = 400, description = "Invalid pagination"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Unit membership required"),
        (status = 404, description = "Unit not found"),
    ),
)]
async fn list_members(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path(unit_id): Path<UnitId>,
    Query(pagination): Query<ListPagination>,
) -> Result<Json<ListMembersResponse>> {
    validate_pagination(&pagination)?;
    require_unit_member(storage.as_ref(), principal, unit_id).await?;
    let memberships = UnitMembershipStore::list(
        storage.as_ref(),
        UnitMembershipFilter::default().unit_id(vec![unit_id]),
        &pagination,
    )
    .await?;
    // The roster page is bounded to 100; fetch display fields in one batch.
    let users = if memberships.is_empty() {
        Vec::new()
    } else {
        UserStore::list(
            storage.as_ref(),
            UserFilter::default().id(memberships.iter().map(|member| member.user_id).collect()),
            &ListPagination::default().limit(100),
        )
        .await?
    };
    let members = memberships
        .into_iter()
        .map(|member| {
            let user = users
                .iter()
                .find(|user| user.id == member.user_id)
                .ok_or(Error::NotFound)?;
            Ok(MemberSummary {
                id: member.id,
                user_id: member.user_id,
                unit_id: member.unit_id,
                rank_id: member.rank_id,
                username: user.username.clone(),
                display_name: user.display_name.clone(),
                icon_url: user.icon_url.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Json(ListMembersResponse { members }))
}

#[utoipa::path(get, path = "/api/v1/units/{unit_id}/access",
    params(("unit_id" = UnitId, Path)),
    responses((status = 200, description = "Current member capabilities", body = UnitAccessResponse),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Unit membership required"),
        (status = 404, description = "Unit not found")))]
async fn unit_access(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path(unit_id): Path<UnitId>,
) -> Result<Json<UnitAccessResponse>> {
    let user_id = super::common::require_user(principal.clone())?;
    require_unit_member(storage.as_ref(), principal, unit_id).await?;
    let unit = super::common::require_unit(storage.as_ref(), unit_id).await?;
    let member = UnitMembershipStore::get_by_user_and_unit(storage.as_ref(), user_id, unit_id)
        .await?
        .ok_or_else(|| Error::Forbidden("Unit membership is required".to_owned()))?;
    let roles = UnitRoleStore::list_for_member(
        storage.as_ref(),
        unit_id,
        member.id,
        &ListPagination::unlimited(),
    )
    .await?;
    let bits = roles.iter().fold(0, |bits, role| bits | role.permissions);
    let is_owner = unit.owner_id == user_id;
    let permissions = if is_owner || bits & Permission::Administrator.bits() != 0 {
        Permissions::KNOWN_BITS
    } else {
        bits
    };
    Ok(Json(UnitAccessResponse {
        member_id: member.id,
        is_owner,
        permissions,
        highest_role_position: roles
            .iter()
            .map(|role| role.position)
            .max()
            .unwrap_or_default(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/units/{unit_id}/members/{member_id}/roles",
    params(
        ("unit_id" = UnitId, Path, description = "Unit ID"),
        ("member_id" = MemberId, Path, description = "Member ID"),
        ("offset" = Option<i64>, Query, description = "Non-negative offset; defaults to 0"),
        ("limit" = Option<i64>, Query, description = "Page size from 1 to 100; defaults to 10"),
    ),
    responses(
        (status = 200, description = "Assigned role IDs including Everyone", body = ListMemberRolesResponse),
        (status = 400, description = "Invalid pagination"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Unit membership required"),
        (status = 404, description = "Unit or member not found"),
    ),
)]
async fn list_member_roles(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path((unit_id, member_id)): Path<(UnitId, MemberId)>,
    Query(pagination): Query<ListPagination>,
) -> Result<Json<ListMemberRolesResponse>> {
    validate_pagination(&pagination)?;
    require_unit_member(storage.as_ref(), principal, unit_id).await?;
    UnitMembershipStore::get(storage.as_ref(), member_id)
        .await?
        .filter(|member| member.unit_id == unit_id)
        .ok_or(Error::NotFound)?;
    let roles =
        UnitRoleStore::list_for_member(storage.as_ref(), unit_id, member_id, &pagination).await?;

    Ok(Json(ListMemberRolesResponse {
        role_ids: roles.into_iter().map(|role| role.id).collect(),
    }))
}
