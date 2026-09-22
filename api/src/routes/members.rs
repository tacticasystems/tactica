use axum::{
    Json, Router,
    extract::{Path, Query},
    routing::get,
};
use tactica_api_types::v1::members::{ListMemberRolesResponse, ListMembersResponse, MemberSummary};
use tactica_db_model::{
    ListPagination, UnitMemberRoleFilter, UnitMemberRoleStore, UnitMembershipFilter,
    UnitMembershipStore,
};
use tactica_uuid_kinds::{MemberId, UnitId};

use super::common::{require_unit_member, validate_pagination};
use crate::{
    error::{Error, Result},
    state::{ApiState, Principal, Storage},
};

pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/api/v1/units/{unit_id}/members", get(list_members))
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
    let members = UnitMembershipStore::list(
        storage.as_ref(),
        UnitMembershipFilter::default().unit_id(vec![unit_id]),
        &pagination,
    )
    .await?
    .into_iter()
    .map(|member| MemberSummary {
        id: member.id,
        user_id: member.user_id,
        unit_id: member.unit_id,
        rank_id: member.rank_id,
    })
    .collect();
    Ok(Json(ListMembersResponse { members }))
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
        (status = 200, description = "Assigned role IDs", body = ListMemberRolesResponse),
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
    let assignments = UnitMemberRoleStore::list(
        storage.as_ref(),
        UnitMemberRoleFilter::default()
            .unit_id(vec![unit_id])
            .member_id(vec![member_id]),
        &pagination,
    )
    .await?;
    Ok(Json(ListMemberRolesResponse {
        role_ids: assignments
            .into_iter()
            .map(|assignment| assignment.role_id)
            .collect(),
    }))
}
