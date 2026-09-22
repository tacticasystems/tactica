use serde::{Deserialize, Serialize};
use tactica_uuid_kinds::{MemberId, RankId, UnitId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CreateUnitRequest {
    pub slug: String,
    pub display_name: String,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub biography: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CreateUnitResponse {
    pub id: UnitId,
    pub slug: String,
    pub display_name: String,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub biography: Option<String>,

    pub rank_id: RankId,
    pub member_id: MemberId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct UnitSummary {
    pub id: UnitId,
    pub slug: String,
    pub display_name: String,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub biography: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ListUnitsResponse {
    pub units: Vec<UnitSummary>,
}
