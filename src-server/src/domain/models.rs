use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    pub access_token: String,
    pub user_id: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MediaSummary {
    pub id: i32,
    pub banner: Option<String>,
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

impl From<Media> for MediaSummary {
    fn from(value: Media) -> Self {
        MediaSummary {
            id: value.id,
            banner: value.banner,
            cover: value.cover.map(|cover| cover.thumbnail),
            title: value.title,
            progress: value.progress,
            total: value.total,
            status: value.status,
        }
    }
}

impl Media {
    pub fn set_episodes(mut self, episodes: Vec<EpisodeSummary>) -> Media {
        self.episodes = episodes;

        self
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EpisodeSummary {
    pub id: String,
    pub title: Option<String>,
    pub duration: u32,
    pub number: f64,
    pub thumbnail: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnqueueData {
    path: PathBuf,
    number: f64,
}

impl EnqueueData {
    pub fn new(episode_number: f64, path: PathBuf) -> Self {
        Self {
            number: episode_number,
            path,
        }
    }

    pub fn episode_number(&self) -> f64 {
        self.number
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

#[derive(Serialize, Deserialize, Debug, PartialOrd, Ord, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProcessingStep {
    InQueue,
    #[serde(rename = "PROCESSING_1080P")]
    Processing1080p,
    #[serde(rename = "PROCESSING_720P")]
    Processing720p,
    ProcessingAudio,
    ProcessingSubtitles,
    Packaging,
    Done,
    Cancelled
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EpisodeQueueItem {
    pub id: String,
    pub media_id: i64,
    pub episode_number: f64,
    pub file_path: PathBuf,
    pub output_dir: PathBuf,
    pub step: ProcessingStep,
    pub progress: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Episode {
    pub id: String,
    pub title: Option<String>,
    pub duration: i64,
    pub number: f64,
    pub thumbnail: Option<String>,
    pub media: MediaSummary,
    pub chapters: Vec<Chapter>,
    pub url: String,
}

impl Episode {
    pub fn use_server_urls(mut self) -> Self {
        self.url = format!(
            "/files/{}/{}",
            self.id,
            PathBuf::from_str(&self.url)
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        );

        self.thumbnail = Some(format!("/files/{}/thumbnail.jpg", self.id));

        self
    }
}

impl EpisodeSummary {
    pub fn use_server_urls(mut self) -> Self {
        self.thumbnail = Some(format!("/files/{}/thumbnail.jpg", self.id));
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chapter {
    pub start: i64,
    pub name: String,
}
