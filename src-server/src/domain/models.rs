use crate::infrastructure::packager::metadata::MediaMetadata;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    pub access_token: String,
    pub user_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaStatus {
    Completed,
    Planned,
    #[serde(rename = "PENDING")]
    Current,
    #[serde(rename = "REPEATING")]
    Revisiting,
    Dropped,
    Paused,
    Unknown(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaType {
    Anime,
    Manga,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MediaSummary {
    pub id: i32,
    pub banner: Option<String>,
    pub description: Option<String>,
    pub cover: Option<String>,
    pub title: String,
    pub progress: Option<u32>,
    pub total: Option<u32>,
    pub status: Option<MediaStatus>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MediaCover {
    pub thumbnail: String,
    pub original: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Media {
    pub id: i32,
    pub banner: Option<String>,
    pub cover: Option<MediaCover>,
    pub title: String,
    pub description: Option<String>,
    pub progress: Option<u32>,
    pub total: Option<u32>,
    pub status: Option<MediaStatus>,
    pub genres: Vec<String>,
    pub episodes: Vec<EpisodeSummary>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProcessingStatusV2 {
    Queued,
    Processing,
    Ready,
}

impl Media {
    pub fn set_episodes(&mut self, episodes: Vec<EpisodeSummary>) {
        self.episodes = episodes;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EpisodeSummary {
    pub id: Uuid,
    pub title: Option<String>,
    pub duration: Option<u32>,
    pub number: u32,
    pub thumbnail: Option<String>,
}

#[derive(Debug)]
pub enum ProcessingStatus {
    Queued,
    Processing,
    Done,
}

#[derive(Debug)]
pub struct ProcessingQueueItem {
    pub id: Uuid,
    pub status: ProcessingStatus,
    pub path: PathBuf,
    pub playlist_path: Option<PathBuf>,
    pub metadata: MediaMetadata,
    pub processed_files: Vec<PathBuf>,
    pub input_file: PathBuf,
}

pub enum ProcessedFileType {
    Audio,
    Subtitle,
    Video,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessedEpisode {
    pub id: Uuid,
    pub episode: f32,
    pub duration: i32,
    pub title: Option<String>,
    pub thumbnail: Option<String>,
}
