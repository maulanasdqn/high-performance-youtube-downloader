use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use hpyd_types::ErrorResponse;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Video unavailable: {0}")]
    VideoUnavailable(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::BadRequest(msg) | Self::ValidationError(msg) | Self::InvalidUrl(msg) => {
                (StatusCode::BAD_REQUEST, msg)
            }
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            Self::DownloadFailed(msg) | Self::VideoUnavailable(msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, msg)
            }
            Self::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let error_response = ErrorResponse::new(message);
        let body = Json(error_response);

        (status, body).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::InternalError(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::InternalError(err.to_string())
    }
}

impl From<url::ParseError> for AppError {
    fn from(err: url::ParseError) -> Self {
        Self::InvalidUrl(err.to_string())
    }
}
