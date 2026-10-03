use axum::{
    Json, Router,
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, patch},
};
use tactica_api_types::v1::ranks::{
    CreateRankRequest, ListRanksResponse, RankSummary, ReorderRanksRequest, UpdateRankRequest,
};
use tactica_db_model::{
    ListPagination, NewUnitRank, UnitRank, UnitRankFilter, UnitRankManagementStore, UnitRankPatch,
    UnitRankStore,
};
use tactica_uuid_kinds::{RankId, UnitId};

use super::common::{require_unit_member, require_user, validate_pagination};
use crate::{
    error::{Error, Result},
    state::{ApiState, Principal, Storage},
};

pub fn router() -> Router<ApiState> {
    Router::new()
        .route(
            "/api/v1/units/{unit_id}/ranks",
            get(list_ranks).post(create_rank),
        )
        .route("/api/v1/units/{unit_id}/ranks/order", patch(reorder_ranks))
        .route(
            "/api/v1/units/{unit_id}/ranks/{rank_id}",
            patch(update_rank).delete(delete_rank),
        )
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
    .map(rank_summary)
    .collect();

    Ok(Json(ListRanksResponse { ranks }))
}

fn rank_summary(item: UnitRank) -> RankSummary {
    RankSummary {
        id: item.id,
        unit_id: item.unit_id,
        slug: item.slug,
        display_name: item.display_name,
        icon_url: item.icon_url,
        description: item.description,
        position: item.position,
    }
}

fn validate_rank(
    slug: Option<&str>,
    name: Option<&str>,
    icon_url: Option<&str>,
    description: Option<&str>,
) -> Result<()> {
    if slug.is_some_and(|value| value.trim().is_empty() || value.chars().count() > 100) {
        return Err(Error::Validation(
            "Rank abbreviation must contain 1–100 characters".to_owned(),
        ));
    }
    if name.is_some_and(|value| value.trim().is_empty() || value.chars().count() > 100) {
        return Err(Error::Validation(
            "Rank display name must contain 1–100 characters or be null".to_owned(),
        ));
    }
    if description.is_some_and(|value| value.chars().count() > 2000) {
        return Err(Error::Validation(
            "Rank description must not exceed 2000 characters".to_owned(),
        ));
    }
    if icon_url.is_some_and(|value| {
        value.len() > 2000
            || !(value.starts_with("https://") || value.starts_with("http://"))
            || value.chars().any(char::is_whitespace)
    }) {
        return Err(Error::Validation(
            "Rank icon must be an HTTP(S) URL of at most 2000 bytes or null".to_owned(),
        ));
    }
    Ok(())
}

#[utoipa::path(post, path = "/api/v1/units/{unit_id}/ranks",
    params(("unit_id" = UnitId, Path)), request_body = CreateRankRequest,
    responses((status = 201, body = RankSummary, description = "Rank created at lowest position"),
        (status = 400, description = "Invalid rank"), (status = 401, description = "Authentication required"),
        (status = 403, description = "ManageRanks required"), (status = 404, description = "Unit not found"),
        (status = 409, description = "Abbreviation already exists")))]
async fn create_rank(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path(unit_id): Path<UnitId>,
    Json(body): Json<CreateRankRequest>,
) -> Result<(StatusCode, Json<RankSummary>)> {
    let actor_id = require_user(&principal)?;
    let slug = body.slug.trim().to_owned();
    let display_name = body.display_name.map(|value| value.trim().to_owned());
    validate_rank(
        Some(&slug),
        display_name.as_deref(),
        body.icon_url.as_deref(),
        body.description.as_deref(),
    )?;
    let rank = UnitRankManagementStore::create_managed_rank(
        storage.as_ref(),
        actor_id,
        NewUnitRank {
            id: RankId::new(),
            unit_id,
            slug,
            display_name,
            icon_url: body.icon_url,
            description: body.description,
        },
    )
    .await?;
    Ok((StatusCode::CREATED, Json(rank_summary(rank))))
}

#[utoipa::path(patch, path = "/api/v1/units/{unit_id}/ranks/{rank_id}",
    params(("unit_id" = UnitId, Path), ("rank_id" = RankId, Path)), request_body = UpdateRankRequest,
    responses((status = 200, body = RankSummary, description = "Rank updated"),
        (status = 400, description = "Invalid rank"), (status = 401, description = "Authentication required"),
        (status = 403, description = "ManageRanks required"), (status = 404, description = "Unit or rank not found"),
        (status = 409, description = "Abbreviation already exists")))]
async fn update_rank(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path((unit_id, rank_id)): Path<(UnitId, RankId)>,
    Json(body): Json<UpdateRankRequest>,
) -> Result<Json<RankSummary>> {
    let actor_id = require_user(&principal)?;
    let slug = body.slug.map(|value| value.trim().to_owned());
    let display_name = body
        .display_name
        .map(|value| value.map(|name| name.trim().to_owned()));
    validate_rank(
        slug.as_deref(),
        display_name.as_ref().and_then(|value| value.as_deref()),
        body.icon_url.as_ref().and_then(|value| value.as_deref()),
        body.description.as_ref().and_then(|value| value.as_deref()),
    )?;
    let rank = UnitRankManagementStore::patch_managed_rank(
        storage.as_ref(),
        actor_id,
        unit_id,
        rank_id,
        UnitRankPatch {
            slug,
            display_name,
            icon_url: body.icon_url,
            description: body.description,
        },
    )
    .await?;
    Ok(Json(rank_summary(rank)))
}

#[utoipa::path(delete, path = "/api/v1/units/{unit_id}/ranks/{rank_id}",
    params(("unit_id" = UnitId, Path), ("rank_id" = RankId, Path)),
    responses((status = 204, description = "Rank deleted"), (status = 401, description = "Authentication required"),
        (status = 403, description = "ManageRanks required"), (status = 404, description = "Unit or rank not found"),
        (status = 409, description = "Rank is assigned or is the initial rank")))]
async fn delete_rank(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path((unit_id, rank_id)): Path<(UnitId, RankId)>,
) -> Result<StatusCode> {
    UnitRankManagementStore::delete_managed_rank(
        storage.as_ref(),
        require_user(&principal)?,
        unit_id,
        rank_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(patch, path = "/api/v1/units/{unit_id}/ranks/order",
    params(("unit_id" = UnitId, Path)), request_body = ReorderRanksRequest,
    responses((status = 200, body = ListRanksResponse, description = "Ranks in highest-first order"),
        (status = 400, description = "Incomplete, duplicate, or foreign rank IDs"), (status = 401, description = "Authentication required"),
        (status = 403, description = "ManageRanks required"), (status = 404, description = "Unit not found")))]
async fn reorder_ranks(
    Storage(storage): Storage,
    Principal(principal): Principal,
    Path(unit_id): Path<UnitId>,
    Json(body): Json<ReorderRanksRequest>,
) -> Result<Json<ListRanksResponse>> {
    let ranks = UnitRankManagementStore::reorder_managed_ranks(
        storage.as_ref(),
        require_user(&principal)?,
        unit_id,
        body.rank_ids,
    )
    .await?;
    Ok(Json(ListRanksResponse {
        ranks: ranks.into_iter().map(rank_summary).collect(),
    }))
}
