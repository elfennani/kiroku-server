use crate::api::payloads::ErrorResponse;
use axum::http::StatusCode;
use axum::{Json, http};
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug)]
pub enum AppError {
    JsonParseError(serde_json::Error),
    BadRequest(String),
    NotFound(String),
    InternalServer(String),

    TranscodeError(String), // Transcoder (ffmpeg) error
    PackagerError(String),  // Shaka packager specific errors
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::InternalServer(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JsonParseError(_) => StatusCode::BAD_REQUEST,
            AppError::TranscodeError(_) => StatusCode::BAD_REQUEST,
            AppError::PackagerError(_) => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
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
        value.status_code()
    }
}

impl From<AppError> for (StatusCode, Json<ErrorResponse>) {
    fn from(value: AppError) -> Self {
        let status_code: StatusCode = value.status_code();

        let response = match value {
            AppError::BadRequest(message) => ErrorResponse::new(message),
            AppError::NotFound(message) => ErrorResponse::new(message),
            AppError::InternalServer(message) => ErrorResponse::new(message),
            AppError::JsonParseError(err) => ErrorResponse::new(err.to_string()),
            AppError::TranscodeError(message) => ErrorResponse::new(message),
            AppError::PackagerError(message) => ErrorResponse::new(message),
        };

        (status_code, Json(response))
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
