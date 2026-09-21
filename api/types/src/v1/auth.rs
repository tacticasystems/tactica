use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tactica_uuid_kinds::UserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct LoginResponse {
    pub token_type: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct MeResponse {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub biography: Option<String>,
    pub is_active: bool,

    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub is_superuser: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
