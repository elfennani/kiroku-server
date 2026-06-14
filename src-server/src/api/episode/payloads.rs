use crate::domain::models::EnqueueData;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct EnqueueEpisodesRequest {
    pub media_id: i64,
    pub items: Vec<EnqueueData>,
}
