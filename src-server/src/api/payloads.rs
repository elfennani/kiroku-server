use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AuthenticateParams {
    pub code: String,
}

#[derive(Deserialize)]
pub struct EnqueueVideo {
    pub path: String,
    pub media_id: usize,
    pub episode: f32,
}

#[derive(Deserialize, Serialize)]
pub struct ErrorResponse {
    message: String,
}

impl ErrorResponse {
    pub fn new(message: &'static str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}
