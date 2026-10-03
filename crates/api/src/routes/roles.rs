use axum::{
    Json, Router,
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, patch, put},
};
use tactica_api_types::v1::roles::{
    CreateRoleRequest, ListRoleMembersResponse, ListRolesResponse, ReorderRolesRequest,
    RoleSummary, UpdateRoleRequest,
};
use tactica_db_model::{
    ListPagination, NewUnitRole, UnitMemberRoleFilter, UnitMemberRoleStore, UnitMembershipFilter,
    UnitMembershipStore, UnitRole, UnitRoleFilter, UnitRoleManagementStore, UnitRolePatch,
    UnitRoleStore, role_kind,
};
use tactica_permissions::Permissions;
use tactica_uuid_kinds::{MemberId, RoleId, UnitId};

use super::common::{require_unit_member, require_user, validate_pagination};
use crate::{
    error::{Error, Result},
    state::{ApiState, Principal, Storage},
};

pub fn router() -> Router<ApiState> {
    Router::new()
        .route(
            "/api/v1/units/{unit_id}/roles",
            get(list_roles).post(create_role),
        )
        .route("/api/v1/units/{unit_id}/roles/order", patch(reorder_roles))
        .route(
            "/api/v1/units/{unit_id}/roles/{role_id}/members",
            get(list_role_members),
        )
        .route(
            "/api/v1/units/{unit_id}/roles/{role_id}",
            patch(update_role).delete(delete_role),
        )
        .route(
            "/api/v1/units/{unit_id}/members/{member_id}/roles/{role_id}",
            put(assign_role).delete(remove_role),
        )
}

#[utoipa::path(get, path = "/api/v1/units/{unit_id}/roles/{role_id}/members",
    params(("unit_id" = UnitId, Path), ("role_id" = RoleId, Path),
        ("offset" = Option<i64>, Query, description = "Non-negative offset; defaults to 0"),
        ("limit" = Option<i64>, Query, description = "Page size from 1 to 100; defaults to 10")),
    responses((status = 200, body = ListRoleMembersResponse, description = "Assigned membership IDs, sorted ascending"),
        (status = 400, description = "Invalid pagination"), (status = 401, description = "Authentication required"),
        (status = 403, description = "Unit membership required"), (status = 404, description = "Unit or role not found")))]
async fn list_role_members(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path((unit_id, role_id)): Path<(UnitId, RoleId)>,
    Query(pagination): Query<ListPagination>,
) -> Result<Json<ListRoleMembersResponse>> {
    validate_pagination(&pagination)?;
    require_unit_member(storage.as_ref(), principal, unit_id).await?;
    let role = UnitRoleStore::get(storage.as_ref(), role_id)
        .await?
        .filter(|role| role.unit_id == unit_id)
        .ok_or(Error::NotFound)?;
    let member_ids = if role.kind == role_kind::EVERYONE {
        UnitMembershipStore::list(
            storage.as_ref(),
            UnitMembershipFilter::default().unit_id(vec![unit_id]),
            &pagination,
        )
        .await?
        .into_iter()
        .map(|member| member.id)
        .collect()
    } else {
        UnitMemberRoleStore::list(
            storage.as_ref(),
            UnitMemberRoleFilter::default()
                .unit_id(vec![unit_id])
                .role_id(vec![role_id]),
            &pagination,
        )
        .await?
        .into_iter()
        .map(|binding| binding.member_id)
        .collect()
    };
    Ok(Json(ListRoleMembersResponse { member_ids }))
}

#[utoipa::path(
    get,
    path = "/api/v1/units/{unit_id}/roles",
    params(
        ("unit_id" = UnitId, Path, description = "Unit ID"),
        ("offset" = Option<i64>, Query, description = "Non-negative offset; defaults to 0"),
        ("limit" = Option<i64>, Query, description = "Page size from 1 to 100; defaults to 10"),
    ),
    responses(
        (status = 200, description = "Unit roles", body = ListRolesResponse),
        (status = 400, description = "Invalid pagination"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Unit membership required"),
        (status = 404, description = "Unit not found"),
    ),
)]
async fn list_roles(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path(unit_id): Path<UnitId>,
    Query(pagination): Query<ListPagination>,
) -> Result<Json<ListRolesResponse>> {
    validate_pagination(&pagination)?;
    require_unit_member(storage.as_ref(), principal, unit_id).await?;
    let roles = UnitRoleStore::list(
        storage.as_ref(),
        UnitRoleFilter::default().unit_id(vec![unit_id]),
        &pagination,
    )
    .await?
    .into_iter()
    .map(role_summary)
    .collect();
    Ok(Json(ListRolesResponse { roles }))
}

fn role_summary(role: UnitRole) -> RoleSummary {
    RoleSummary {
        id: role.id,
        unit_id: role.unit_id,
        display_name: role.display_name,
        description: role.description,
        permissions: role.permissions,
        position: role.position,
        kind: role.kind,
    }
}

fn validate_role(
    name: Option<&str>,
    description: Option<&str>,
    permissions: Option<i64>,
) -> Result<()> {
    if name.is_some_and(|name| name.trim().is_empty() || name.chars().count() > 100) {
        return Err(Error::Validation(
            "Role display name must contain 1–100 characters".to_owned(),
        ));
    }
    if description.is_some_and(|description| description.chars().count() > 2000) {
        return Err(Error::Validation(
            "Role description must not exceed 2000 characters".to_owned(),
        ));
    }
    if permissions.is_some_and(|permissions| Permissions::from_bits(permissions).is_none()) {
        return Err(Error::Validation(
            "Permissions contain undefined bits".to_owned(),
        ));
    }
    Ok(())
}

#[utoipa::path(post, path = "/api/v1/units/{unit_id}/roles",
    params(("unit_id" = UnitId, Path)), request_body = CreateRoleRequest,
    responses((status = 201, body = RoleSummary, description = "Role created"),
        (status = 400, description = "Invalid role"), (status = 401, description = "Authentication required"),
        (status = 403, description = "Required permission or role hierarchy missing"), (status = 404, description = "Unit not found"),
        (status = 409, description = "Role name already exists")))]
async fn create_role(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path(unit_id): Path<UnitId>,
    Json(body): Json<CreateRoleRequest>,
) -> Result<(StatusCode, Json<RoleSummary>)> {
    let actor_id = require_user(&principal)?;
    let name = body.display_name.trim();
    validate_role(
        Some(name),
        body.description.as_deref(),
        Some(body.permissions),
    )?;
    let role = UnitRoleManagementStore::create_managed_role(
        storage.as_ref(),
        actor_id,
        NewUnitRole {
            id: RoleId::new(),
            unit_id,
            display_name: name.to_owned(),
            description: body.description,
            permissions: body.permissions,
        },
    )
    .await?;
    Ok((StatusCode::CREATED, Json(role_summary(role))))
}

#[utoipa::path(patch, path = "/api/v1/units/{unit_id}/roles/{role_id}",
    params(("unit_id" = UnitId, Path), ("role_id" = RoleId, Path)), request_body = UpdateRoleRequest,
    responses((status = 200, body = RoleSummary, description = "Role updated"),
        (status = 400, description = "Invalid role"), (status = 401, description = "Authentication required"),
        (status = 403, description = "Required permission or role hierarchy missing"), (status = 404, description = "Unit or role not found"),
        (status = 409, description = "Role name already exists")))]
async fn update_role(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path((unit_id, role_id)): Path<(UnitId, RoleId)>,
    Json(body): Json<UpdateRoleRequest>,
) -> Result<Json<RoleSummary>> {
    let actor_id = require_user(&principal)?;
    let name = body.display_name.map(|name| name.trim().to_owned());
    validate_role(
        name.as_deref(),
        body.description.as_ref().and_then(|value| value.as_deref()),
        body.permissions,
    )?;
    let role = UnitRoleManagementStore::patch_managed_role(
        storage.as_ref(),
        actor_id,
        unit_id,
        role_id,
        UnitRolePatch {
            display_name: name,
            description: body.description,
            permissions: body.permissions,
        },
    )
    .await?;
    Ok(Json(role_summary(role)))
}

#[utoipa::path(delete, path = "/api/v1/units/{unit_id}/roles/{role_id}",
    params(("unit_id" = UnitId, Path), ("role_id" = RoleId, Path)),
    responses((status = 204, description = "Role deleted"), (status = 401, description = "Authentication required"),
        (status = 403, description = "Required permission or role hierarchy missing"), (status = 404, description = "Unit or role not found")))]
async fn delete_role(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path((unit_id, role_id)): Path<(UnitId, RoleId)>,
) -> Result<StatusCode> {
    let actor_id = require_user(&principal)?;
    UnitRoleManagementStore::delete_managed_role(storage.as_ref(), actor_id, unit_id, role_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(put, path = "/api/v1/units/{unit_id}/members/{member_id}/roles/{role_id}",
    params(("unit_id" = UnitId, Path), ("member_id" = MemberId, Path), ("role_id" = RoleId, Path)),
    responses((status = 204, description = "Role assigned or already assigned"), (status = 401, description = "Authentication required"),
        (status = 403, description = "Required permission or role hierarchy missing"), (status = 404, description = "Unit, member, or role not found")))]
async fn assign_role(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path((unit_id, member_id, role_id)): Path<(UnitId, MemberId, RoleId)>,
) -> Result<StatusCode> {
    let actor_id = require_user(&principal)?;
    UnitRoleManagementStore::assign_managed_role(
        storage.as_ref(),
        actor_id,
        unit_id,
        member_id,
        role_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(delete, path = "/api/v1/units/{unit_id}/members/{member_id}/roles/{role_id}",
    params(("unit_id" = UnitId, Path), ("member_id" = MemberId, Path), ("role_id" = RoleId, Path)),
    responses((status = 204, description = "Assignment removed or already absent"), (status = 401, description = "Authentication required"),
        (status = 403, description = "Required permission or role hierarchy missing"), (status = 404, description = "Unit, member, or role not found")))]
async fn remove_role(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path((unit_id, member_id, role_id)): Path<(UnitId, MemberId, RoleId)>,
) -> Result<StatusCode> {
    let actor_id = require_user(&principal)?;
    UnitRoleManagementStore::remove_managed_role(
        storage.as_ref(),
        actor_id,
        unit_id,
        member_id,
        role_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(patch, path = "/api/v1/units/{unit_id}/roles/order",
    params(("unit_id" = UnitId, Path)), request_body = ReorderRolesRequest,
    responses((status = 200, body = ListRolesResponse, description = "Updated roles, highest first"),
        (status = 400, description = "Invalid or incomplete order"), (status = 401, description = "Authentication required"),
        (status = 403, description = "ManageRoles required or protected role moved"), (status = 404, description = "Unit not found")))]
async fn reorder_roles(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path(unit_id): Path<UnitId>,
    Json(body): Json<ReorderRolesRequest>,
) -> Result<Json<ListRolesResponse>> {
    let actor_id = require_user(&principal)?;
    let roles = UnitRoleManagementStore::reorder_managed_roles(
        storage.as_ref(),
        actor_id,
        unit_id,
        body.role_ids,
    )
    .await?;
    Ok(Json(ListRolesResponse {
        roles: roles.into_iter().map(role_summary).collect(),
    }))
}
