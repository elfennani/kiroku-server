use axum::http::StatusCode;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    InternalServer(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(value: rusqlite::Error) -> Self {
        AppError::InternalServer(value.to_string())
    }
}

impl From<AppError> for StatusCode {
    fn from(value: AppError) -> Self {
        match value {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::InternalServer(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}