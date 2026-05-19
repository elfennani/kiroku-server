use axum::http::StatusCode;
use std::fmt::Display;

#[derive(Debug)]
pub enum AppError {
    JsonParseError(serde_json::Error),
    BadRequest(String),
    NotFound(String),
    InternalServer(String),

    TranscodeError(String), // Transcoder (ffmpeg) error
    PackagerError(String),  // Shaka packager specific errors
}

impl From<rusqlite::Error> for AppError {
    fn from(value: rusqlite::Error) -> Self {
        AppError::InternalServer(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        AppError::JsonParseError(value)
    }
}

impl From<AppError> for StatusCode {
    fn from(value: AppError) -> Self {
        match value {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::InternalServer(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JsonParseError(_) => StatusCode::BAD_REQUEST,
            AppError::TranscodeError(_) => StatusCode::BAD_REQUEST,
            AppError::PackagerError(_) => StatusCode::BAD_REQUEST,
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::JsonParseError(err) => write!(f, "JsonParseError: {}", err),
            AppError::BadRequest(err) => write!(f, "BadRequest: {}", err),
            AppError::NotFound(err) => write!(f, "NotFound: {}", err),
            AppError::InternalServer(err) => write!(f, "InternalServer: {}", err),
            AppError::TranscodeError(err) => write!(f, "TranscodeError: {}", err),
            AppError::PackagerError(err) => write!(f, "PackagerError: {}", err),
        }
    }
}
