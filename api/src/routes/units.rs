use axum::{
    Json, Router,
    extract::{Path, Query},
    response::IntoResponse,
    routing::{get, post},
};
use tactica_api_types::v1;
use tactica_auth::principal;
use tactica_db_model::{
    CreateUnit, ListPagination, NewUnit, StoreError, UnitFilter, UnitMembershipStore, UnitStore,
};
use tactica_uuid_kinds::UnitId;

use crate::{
    error::{Error, Result},
    state::{ApiState, Principal, Storage},
};

pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/api/v1/units", get(list_units))
        .route("/api/v1/units", post(create_unit))
        .route("/api/v1/units/{unit_id}", get(get_unit))
}

#[utoipa::path(
    get,
    path = "/api/v1/units/{unit_id}",
    params(("unit_id" = UnitId, Path, description = "Unit ID")),
    responses(
        (status = 200, description = "Unit detail", body = v1::units::UnitSummary),
        (status = 404, description = "Unit not found"),
    ),
)]
async fn get_unit(
    Storage(storage): Storage,
    Path(unit_id): Path<UnitId>,
) -> Result<Json<v1::units::UnitSummary>> {
    let unit = super::common::require_unit(storage.as_ref(), unit_id).await?;
    let counts = UnitMembershipStore::count_by_unit(storage.as_ref(), vec![unit_id]).await?;
    Ok(Json(v1::units::UnitSummary {
        id: unit.id,
        slug: unit.slug,
        display_name: unit.display_name.unwrap_or_default(),
        icon_url: unit.icon_url,
        banner_url: unit.banner_url,
        biography: unit.biography,
        member_count: counts.get(&unit_id).copied().unwrap_or_default(),
    }))
}

async fn list_units(
    Storage(stg): Storage,
    Query(pag): Query<ListPagination>,
) -> Result<impl IntoResponse> {
    super::common::validate_pagination(&pag)?;
    let units = UnitStore::list(stg.as_ref(), UnitFilter::default(), &pag)
        .await
        .inspect_err(|err| tracing::error!(?err, "Failed to list units"))?;

    let counts = UnitMembershipStore::count_by_unit(
        stg.as_ref(),
        units.iter().map(|unit| unit.id).collect(),
    )
    .await?;
    let unit_responses: Vec<v1::units::UnitSummary> = units
        .into_iter()
        .map(|unit| v1::units::UnitSummary {
            id: unit.id,
            slug: unit.slug,
            display_name: unit.display_name.unwrap_or_default(),
            icon_url: unit.icon_url,
            banner_url: unit.banner_url,
            biography: unit.biography,
            member_count: counts.get(&unit.id).copied().unwrap_or_default(),
        })
        .collect();

    Ok(Json(v1::units::ListUnitsResponse {
        units: unit_responses,
    }))
}

async fn create_unit(
    Storage(stg): Storage,
    Principal(prn): Principal,
    Json(body): Json<v1::units::CreateUnitRequest>,
) -> Result<impl IntoResponse> {
    if !matches!(prn, principal::Principal::User(_)) {
        return Err(crate::error::Error::Unauthorized(
            "Only users can create units".to_string(),
        ));
    }

    let user_id = match prn {
        principal::Principal::User(user_id) => user_id,
        _ => unreachable!(),
    };

    let created = UnitStore::create_with_defaults(
        stg.as_ref(),
        CreateUnit {
            owner_id: user_id,
            unit: NewUnit {
                id: UnitId::new(),
                slug: body.slug,
                display_name: Some(body.display_name),
                icon_url: body.icon_url,
                banner_url: body.banner_url,
                biography: body.biography,
            },
        },
    )
    .await
    .map_err(|err| {
        if matches!(err, StoreError::Conflict) {
            Error::Forbidden("A unit with that slug already exists".to_string())
        } else {
            err.into()
        }
    })
    .inspect_err(|err| tracing::error!(?err, "Failed to create unit with defaults"))?;

    Ok(Json(v1::units::CreateUnitResponse {
        id: created.unit.id,
        slug: created.unit.slug,
        display_name: created.unit.display_name.unwrap(),
        icon_url: created.unit.icon_url,
        banner_url: created.unit.banner_url,
        biography: created.unit.biography,

        rank_id: created.owner_rank_id,
        member_id: created.owner_membership_id,
    }))
}
