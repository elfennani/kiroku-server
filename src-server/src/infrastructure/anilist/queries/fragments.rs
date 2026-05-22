use std::fmt::Display;
use crate::domain::models;
use crate::domain::models::Image;
use crate::errors::AppError;
use crate::infrastructure::anilist::schema;

#[derive(cynic::QueryFragment)]
pub struct MediaCoverImage {
    extra_large: Option<String>,
    large: Option<String>,
    medium: Option<String>,
}

impl TryFrom<MediaCoverImage> for models::Image {
    type Error = AppError;

    fn try_from(cover: MediaCoverImage) -> Result<Self, Self::Error> {
        if cover.large.is_none() || cover.extra_large.is_none() {
            Err(Self::Error::InternalServer(
                "Failed to parse cover".to_string(),
            ))
        } else {
            Ok(Image {
                thumbnail: cover.large.unwrap(),
                url: cover.extra_large.unwrap(),
                width: None,
                height: None,
            })
        }
    }
}

#[derive(cynic::Enum)]
pub enum MediaListStatus {
    Completed,
    Current,
    Dropped,
    Paused,
    Planning,
    Repeating,
}

impl From<MediaListStatus> for models::Status {
    fn from(value: MediaListStatus) -> Self {
        match value {
            MediaListStatus::Completed => Self::Completed,
            MediaListStatus::Current => Self::Current,
            MediaListStatus::Dropped => Self::Dropped,
            MediaListStatus::Paused => Self::Paused,
            MediaListStatus::Planning => Self::Planned,
            MediaListStatus::Repeating => Self::Revisiting,
        }
    }
}

#[derive(cynic::QueryFragment)]
pub struct MediaTitle {
    user_preferred: Option<String>,
    english: Option<String>,
    romaji: Option<String>,
    native: Option<String>,
}

impl Display for MediaTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = self
            .user_preferred
            .clone()
            .or(self.english.clone())
            .or(self.romaji.clone())
            .or(self.native.clone())
            .unwrap_or("UNTITLED".to_string());

        write!(f, "{}", str)
    }
}

#[derive(cynic::Enum)]
pub enum MediaType {
    Anime,
    Manga,
}