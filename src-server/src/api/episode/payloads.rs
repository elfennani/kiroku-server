use crate::domain::models::{EnqueueData, MediaSummary, ProcessingStep};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct EnqueueEpisodesRequest {
    pub media_id: i64,
    pub items: Vec<EnqueueData>,
}

#[derive(Serialize)]
pub struct GetQueueResponseItem {
    pub id: String,
    pub step: ProcessingStep,
    pub progress: Option<f64>,
    pub media: MediaSummary,
    pub number: f64,
}
