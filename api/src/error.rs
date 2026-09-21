use axum::response::IntoResponse;
use tactica_api_types::v1;
use tactica_db_model::StoreError;

/// A specialized `Result` type for API operations.
pub type Result<T> = core::result::Result<T, Error>;

/// An error type for API operations, encapsulating various error scenarios.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not Found")]
    NotFound,

    #[error("Validation failed: {0}")]
    ValidationError(String),

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
        let status = match &self {
            Error::NotFound => axum::http::StatusCode::NOT_FOUND,
            Error::ValidationError(_) => axum::http::StatusCode::BAD_REQUEST,
            Error::Unauthorized(_) => axum::http::StatusCode::UNAUTHORIZED,
            Error::Forbidden(_) => axum::http::StatusCode::FORBIDDEN,

            Error::Store(err) => match err {
                StoreError::Conflict => axum::http::StatusCode::CONFLICT,
                _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            },

            _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        let error = v1::ApiError {
            code: match &self {
                Error::NotFound => "not_found".to_string(),
                Error::ValidationError(_) => "validation_error".to_string(),

                Error::Store(err) => match err {
                    StoreError::Conflict => "conflict".to_string(),
                    _ => "internal_server_error".to_string(),
                },

                _ => "internal_server_error".to_string(),
            },
            message: match &self {
                Error::Store(err) => match err {
                    StoreError::Conflict => "A conflict occurred with the database.".to_string(),
                    _ => "Internal server error".to_string(),
                },

                _ => self.to_string(),
            },

            details: None,
        };

        (status, axum::Json(error)).into_response()
    }
}
