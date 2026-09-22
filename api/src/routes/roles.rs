use axum::{
    Json, Router,
    extract::{Path, Query},
    routing::get,
};
use tactica_api_types::v1::roles::{ListRolesResponse, RoleSummary};
use tactica_db_model::{ListPagination, UnitRoleFilter, UnitRoleStore};
use tactica_uuid_kinds::UnitId;

use super::common::{require_unit_member, validate_pagination};
use crate::{
    error::Result,
    state::{ApiState, Principal, Storage},
};

pub fn router() -> Router<ApiState> {
    Router::new().route("/api/v1/units/{unit_id}/roles", get(list_roles))
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
    .map(|item| RoleSummary {
        id: item.id,
        unit_id: item.unit_id,
        display_name: item.display_name,
        description: item.description,
        permissions: item.permissions,
    })
    .collect();
    Ok(Json(ListRolesResponse { roles }))
}
