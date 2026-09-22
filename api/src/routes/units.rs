use axum::{Json, Router, extract::Query, response::IntoResponse, routing::{get, post}};
use tactica_api_types::v1;
use tactica_auth::principal;
use tactica_db_model::{ListPagination, NewUnit, NewUnitMembership, NewUnitRank, NewUnitSettings, UnitFilter, UnitMembershipStore, UnitRankStore, UnitSettingsStore, UnitStore};
use tactica_uuid_kinds::{MemberId, RankId, UnitId};

use crate::{error::Result, state::{ApiState, Principal, Storage}};

pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/api/v1/units", get(list_units))
        .route("/api/v1/units", post(create_unit))
}

async fn list_units(
    Storage(stg): Storage,
    Query(pag): Query<ListPagination>,
) -> Result<impl IntoResponse> {
    let units = UnitStore::list(stg.as_ref(), UnitFilter::default(), &pag)
        .await
        .inspect_err(|err| tracing::error!(?err, "Failed to list units"))?;

    let unit_responses: Vec<v1::units::UnitSummary> = units.into_iter().map(|unit| {
        v1::units::UnitSummary {
            id: unit.id,
            slug: unit.slug,
            display_name: unit.display_name.unwrap_or_default(),
            icon_url: unit.icon_url,
            banner_url: unit.banner_url,
            biography: unit.biography,
        }
    }).collect();

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
        ))
    }

    let user_id = match prn {
        principal::Principal::User(user_id) => user_id,
        _ => unreachable!(),
    };

    let unit = UnitStore::create(
        stg.as_ref(),
        NewUnit {
            id: UnitId::new(),
            slug: body.slug,
            display_name: Some(body.display_name),
            icon_url: body.icon_url,
            banner_url: body.banner_url,
            biography: body.biography,
        },
    )
        .await
        .inspect_err(|err| tracing::error!(?err, "Failed to create unit"))?;

    let unit_rank = UnitRankStore::create(
        stg.as_ref(),
        NewUnitRank {
            id: RankId::new(),
            unit_id: unit.id,
            slug: "Maj.".to_string(),
            display_name: Some("Major".to_string()),
            description: Some("The owner of the unit".to_string()),
            icon_url: None,
        }
    )
        .await
        .inspect_err(|err| tracing::error!(?err, "Failed to create unit rank"))?;

    let unit_join_rank = UnitRankStore::create(
        stg.as_ref(),
        NewUnitRank {
            id: RankId::new(),
            unit_id: unit.id,
            slug: "Pvt.".to_string(),
            display_name: Some("Private".to_string()),
            description: Some("The enlisted members".to_string()),
            icon_url: None,
        }
    )
        .await
        .inspect_err(|err| tracing::error!(?err, "Failed to create unit rank"))?;

    UnitSettingsStore::create(
        stg.as_ref(),
        NewUnitSettings {
            unit_id: unit.id,
            initial_rank_id: unit_join_rank.id,
            discord_guild_id: None,
            discord_guild_joined_at: None,
            updated_by: Some(user_id),
        },
    )
        .await
        .inspect_err(|err| tracing::error!(?err, "Failed to create unit settings"))?;

    let unit_member = UnitMembershipStore::create(
        stg.as_ref(),
        NewUnitMembership {
            id: MemberId::new(),
            rank_id: unit_rank.id,
            unit_id: unit.id,
            user_id: user_id,
        }
    )
        .await
        .inspect_err(|err| tracing::error!(?err, "Failed to create unit membership"))?;

    Ok(Json(v1::units::CreateUnitResponse {
        id: unit.id,
        slug: unit.slug,
        display_name: unit.display_name.unwrap(),
        icon_url: unit.icon_url,
        banner_url: unit.banner_url,
        biography: unit.biography,

        rank_id: unit_rank.id,
        member_id: unit_member.id,
    }))
}
