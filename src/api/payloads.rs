use serde::Deserialize;

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
