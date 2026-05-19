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

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    pub thumbnail: String,
    pub url: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    Completed,
    Planned,
    Current,
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
pub struct MediaStatus {
    pub status: Option<Status>,
    pub progress: Option<i32>,
    pub total: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Media {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub cover: Option<Image>,
    pub banner: Option<String>,
    pub media_type: MediaType,
    pub status: MediaStatus,
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
