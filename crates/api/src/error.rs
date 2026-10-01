use axum::response::IntoResponse;
use tactica_api_types::v1;
use tactica_db_model::StoreError;

/// A specialized `Result` type for API operations.
pub type Result<T> = core::result::Result<T, Error>;

/// An error type for API operations, encapsulating various error scenarios.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("File exceeds upload limit")]
    PayloadTooLarge,
    #[error("File uploads are not configured")]
    UploadsUnavailable,
    #[error("File storage failed: {0}")]
    FileStorage(#[from] tactica_files::FileStorageError),

    #[error("Not Found")]
    NotFound,

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Database error: {0}")]
    Store(#[from] tactica_db_model::StoreError),

    #[error("An unknown error occurred.")]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status_code, message, code) = match &self {
            Self::PayloadTooLarge => (
                axum::http::StatusCode::PAYLOAD_TOO_LARGE,
                "File exceeds upload limit".to_owned(),
                "payload_too_large".to_owned(),
            ),
            Self::UploadsUnavailable => (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "File uploads are not configured".to_owned(),
                "uploads_unavailable".to_owned(),
            ),
            Self::FileStorage(tactica_files::FileStorageError::NotFound) | Self::NotFound => (
                axum::http::StatusCode::NOT_FOUND,
                "Resource not found".to_string(),
                "not_found".to_string(),
            ),

            Self::Validation(msg) => (
                axum::http::StatusCode::BAD_REQUEST,
                msg.clone(),
                "invalid_input".to_string(),
            ),

            Self::Unauthorized(msg) => (
                axum::http::StatusCode::UNAUTHORIZED,
                msg.clone(),
                "unauthorized".to_string(),
            ),

            Self::Forbidden(msg) => (
                axum::http::StatusCode::FORBIDDEN,
                msg.clone(),
                "forbidden".to_string(),
            ),

            Self::Store(StoreError::Conflict) => (
                axum::http::StatusCode::CONFLICT,
                "A conflict occurred with the database.".to_string(),
                "conflict".to_string(),
            ),

            Self::Other(err) => {
                tracing::error!("Internal server error: {:?}", err);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error occurred.".to_string(),
                    "internal_server_error".to_string(),
                )
            }

            _ => {
                tracing::error!(error = ?self, "Internal server error");
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error occurred.".to_string(),
                    "internal_server_error".to_string(),
                )
            }
        };

        let error = v1::ApiError {
            code,
            message,
            details: None,
        };

        (status_code, axum::Json(error)).into_response()
    }
}

impl From<tactica_db_model::RoleWriteError> for Error {
    fn from(error: tactica_db_model::RoleWriteError) -> Self {
        use tactica_db_model::RoleWriteError;
        match error {
            RoleWriteError::Unauthorized => {
                Self::Unauthorized("User is missing or inactive".to_owned())
            }
            RoleWriteError::Forbidden => {
                Self::Forbidden("Insufficient permissions or role hierarchy".to_owned())
            }
            RoleWriteError::NotFound => Self::NotFound,
            RoleWriteError::ProtectedRole => {
                Self::Forbidden("This operation is not allowed on a built-in role".to_owned())
            }
            RoleWriteError::InvalidPermissions => {
                Self::Validation("Permissions contain undefined bits".to_owned())
            }
            RoleWriteError::InvalidOrder => {
                Self::Validation("Role order must contain every unit role exactly once".to_owned())
            }
            RoleWriteError::Store(error) => Self::Store(error),
        }
    }
}
