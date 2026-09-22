use serde::{Deserialize, Serialize};
use tactica_uuid_kinds::{RankId, UnitId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct RankSummary {
    pub id: RankId,
    pub unit_id: UnitId,
    pub slug: String,
    pub display_name: Option<String>,
    pub icon_url: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ListRanksResponse {
    pub ranks: Vec<RankSummary>,
}
