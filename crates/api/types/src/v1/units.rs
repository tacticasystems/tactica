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
    pub member_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ListUnitsResponse {
    pub units: Vec<UnitSummary>,
}

/// Only supplied fields change. Null clears biography and banner_url.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct UpdateUnitRequest {
    #[serde(
        default,
        deserialize_with = "provided_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub display_name: Option<String>,
    #[serde(
        default,
        deserialize_with = "provided_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub slug: Option<String>,
    #[serde(
        default,
        deserialize_with = "nullable_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub biography: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "nullable_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub banner_url: Option<Option<String>>,
}

fn provided_string<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    String::deserialize(deserializer).map(Some)
}
fn nullable_string<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error> {
    Option::<String>::deserialize(deserializer).map(Some)
}
