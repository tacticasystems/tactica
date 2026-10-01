use axum::{
    Json, Router,
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, put},
};
use tactica_api_types::v1::members::{
    ListMemberRolesResponse, ListMembersResponse, MemberSummary, SetMemberRankRequest,
    UnitAccessResponse, UpdateMemberRequest,
};
use tactica_db_model::{
    ListPagination, MemberPatch, UnitMemberManagementStore, UnitMemberRoleFilter,
    UnitMemberRoleStore, UnitMembershipFilter, UnitMembershipStore, UnitRankManagementStore,
    UnitRoleFilter, UnitRoleStore, UserFilter, UserStore, role_kind,
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
        .route(
            "/api/v1/units/{unit_id}/members/{member_id}",
            get(get_member).patch(update_member),
        )
        .route("/api/v1/units/{unit_id}/access", get(unit_access))
        .route(
            "/api/v1/units/{unit_id}/members/{member_id}/rank",
            put(set_member_rank),
        )
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
    let roles = UnitRoleStore::list(
        storage.as_ref(),
        UnitRoleFilter::default().unit_id(vec![unit_id]),
        &ListPagination::unlimited(),
    )
    .await?;
    // Load all assignments for this roster page in one batch, including members
    // whose roles exceed the normal collection page size.
    let assignments = if memberships.is_empty() {
        Vec::new()
    } else {
        UnitMemberRoleStore::list(
            storage.as_ref(),
            UnitMemberRoleFilter::default()
                .unit_id(vec![unit_id])
                .member_id(memberships.iter().map(|member| member.id).collect()),
            &ListPagination::unlimited(),
        )
        .await?
    };
    let mut assigned = std::collections::HashMap::<_, std::collections::HashSet<_>>::new();
    for assignment in assignments {
        assigned
            .entry(assignment.member_id)
            .or_default()
            .insert(assignment.role_id);
    }
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
                display_name: member
                    .display_name
                    .clone()
                    .or_else(|| user.display_name.clone()),
                unit_display_name: member.display_name.clone(),
                icon_url: user.icon_url.clone(),
                role_ids: roles
                    .iter()
                    .filter(|role| {
                        role.kind == role_kind::EVERYONE
                            || assigned
                                .get(&member.id)
                                .is_some_and(|ids| ids.contains(&role.id))
                    })
                    .map(|role| role.id)
                    .collect(),
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

#[utoipa::path(put, path = "/api/v1/units/{unit_id}/members/{member_id}/rank",
    params(("unit_id" = UnitId, Path), ("member_id" = MemberId, Path)),
    request_body = SetMemberRankRequest,
    responses((status = 204, description = "Member rank set"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Assign ranks permission required"),
        (status = 404, description = "Unit, member, or rank not found in this unit")))]
async fn set_member_rank(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path((unit_id, member_id)): Path<(UnitId, MemberId)>,
    Json(input): Json<SetMemberRankRequest>,
) -> Result<StatusCode> {
    let actor_id = super::common::require_user(principal)?;
    UnitRankManagementStore::set_member_rank(
        storage.as_ref(),
        actor_id,
        unit_id,
        member_id,
        input.rank_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get, path = "/api/v1/units/{unit_id}/members/{member_id}",
    params(("unit_id" = UnitId, Path), ("member_id" = MemberId, Path)),
    responses((status = 200, description = "Member details", body = MemberSummary),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Unit membership required"),
        (status = 404, description = "Unit or member not found")))]
async fn get_member(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path((unit_id, member_id)): Path<(UnitId, MemberId)>,
) -> Result<Json<MemberSummary>> {
    require_unit_member(storage.as_ref(), principal, unit_id).await?;
    let member = UnitMembershipStore::get(storage.as_ref(), member_id)
        .await?
        .filter(|member| member.unit_id == unit_id)
        .ok_or(Error::NotFound)?;
    let user = UserStore::get(storage.as_ref(), member.user_id)
        .await?
        .ok_or(Error::NotFound)?;
    let mut roles = UnitRoleStore::list_for_member(
        storage.as_ref(),
        unit_id,
        member_id,
        &ListPagination::unlimited(),
    )
    .await?;
    roles.sort_by_key(|role| std::cmp::Reverse(role.position));
    Ok(Json(MemberSummary {
        id: member.id,
        user_id: member.user_id,
        unit_id: member.unit_id,
        rank_id: member.rank_id,
        username: user.username,
        display_name: member.display_name.clone().or(user.display_name),
        unit_display_name: member.display_name,
        icon_url: user.icon_url,
        role_ids: roles.into_iter().map(|role| role.id).collect(),
    }))
}

#[utoipa::path(patch, path = "/api/v1/units/{unit_id}/members/{member_id}",
    params(("unit_id" = UnitId, Path), ("member_id" = MemberId, Path)),
    request_body = UpdateMemberRequest,
    responses((status = 204, description = "Member changes saved atomically"),
        (status = 400, description = "Invalid name, role list, or empty patch"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Required permission or role hierarchy missing"),
        (status = 404, description = "Unit, member, rank, or role not found")))]
async fn update_member(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path((unit_id, member_id)): Path<(UnitId, MemberId)>,
    Json(mut input): Json<UpdateMemberRequest>,
) -> Result<StatusCode> {
    let actor_id = super::common::require_user(principal)?;
    if input.display_name.is_none() && input.rank_id.is_none() && input.role_ids.is_none() {
        return Err(Error::Validation(
            "Provide at least one member field".to_owned(),
        ));
    }
    if let Some(name) = input.display_name.as_mut() {
        *name = name
            .as_ref()
            .map(|name| name.trim().to_owned())
            .filter(|name| !name.is_empty());
        if name.as_ref().is_some_and(|name| name.chars().count() > 100) {
            return Err(Error::Validation(
                "Display name must be at most 100 characters".to_owned(),
            ));
        }
    }
    if let Some(ids) = &input.role_ids {
        if ids.iter().collect::<std::collections::HashSet<_>>().len() != ids.len() {
            return Err(Error::Validation("Role IDs must be unique".to_owned()));
        }
    }
    UnitMemberManagementStore::patch_managed_member(
        storage.as_ref(),
        actor_id,
        unit_id,
        member_id,
        MemberPatch {
            display_name: input.display_name,
            rank_id: input.rank_id,
            role_ids: input.role_ids,
        },
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
