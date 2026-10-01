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
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ListRanksResponse {
    pub ranks: Vec<RankSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CreateRankRequest {
    pub slug: String,
    pub display_name: Option<String>,
    pub icon_url: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct UpdateRankRequest {
    #[serde(
        default,
        deserialize_with = "required_patch",
        skip_serializing_if = "Option::is_none"
    )]
    pub slug: Option<String>,
    #[serde(
        default,
        deserialize_with = "nullable_patch",
        skip_serializing_if = "Option::is_none"
    )]
    pub display_name: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "nullable_patch",
        skip_serializing_if = "Option::is_none"
    )]
    pub icon_url: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "nullable_patch",
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<Option<String>>,
}

fn required_patch<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    String::deserialize(deserializer).map(Some)
}

#[expect(
    clippy::option_option,
    reason = "PATCH distinguishes omitted, null, and a value"
)]
fn nullable_patch<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error> {
    Option::<String>::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ReorderRanksRequest {
    /// Every rank in the unit, exactly once, lowest first.
    pub rank_ids: Vec<RankId>,
}
