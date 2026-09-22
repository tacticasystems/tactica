use axum::{
    Json, Router,
    extract::{Path, Query},
    routing::get,
};
use tactica_api_types::v1::ranks::{ListRanksResponse, RankSummary};
use tactica_db_model::{ListPagination, UnitRankFilter, UnitRankStore};
use tactica_uuid_kinds::UnitId;

use super::common::{require_unit_member, validate_pagination};
use crate::{
    error::Result,
    state::{ApiState, Principal, Storage},
};

pub fn router() -> Router<ApiState> {
    Router::new().route("/api/v1/units/{unit_id}/ranks", get(list_ranks))
}

#[utoipa::path(
    get,
    path = "/api/v1/units/{unit_id}/ranks",
    params(
        ("unit_id" = UnitId, Path, description = "Unit ID"),
        ("offset" = Option<i64>, Query, description = "Non-negative offset; defaults to 0"),
        ("limit" = Option<i64>, Query, description = "Page size from 1 to 100; defaults to 10"),
    ),
    responses(
        (status = 200, description = "Unit ranks", body = ListRanksResponse),
        (status = 400, description = "Invalid pagination"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "Unit membership required"),
        (status = 404, description = "Unit not found"),
    ),
)]
async fn list_ranks(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path(unit_id): Path<UnitId>,
    Query(pagination): Query<ListPagination>,
) -> Result<Json<ListRanksResponse>> {
    validate_pagination(&pagination)?;
    require_unit_member(storage.as_ref(), principal, unit_id).await?;
    let ranks = UnitRankStore::list(
        storage.as_ref(),
        UnitRankFilter::default().unit_id(vec![unit_id]),
        &pagination,
    )
        .await?
        .into_iter()
        .map(|item| RankSummary {
            id: item.id,
            unit_id: item.unit_id,
            slug: item.slug,
            display_name: item.display_name,
            icon_url: item.icon_url,
            description: item.description,
        })
        .collect();

    Ok(Json(ListRanksResponse { ranks }))
}
