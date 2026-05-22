use crate::domain::models;
use crate::domain::models::{Image, MediaStatus};
use crate::errors::AppError;
use crate::infrastructure::anilist::queries::fragments::*;
use crate::infrastructure::anilist::schema;



#[derive(cynic::QueryFragment)]
struct Media {
    id: i32,
    title: Option<MediaTitle>,
    episodes: Option<i32>,
    banner_image: Option<String>,
    cover_image: Option<MediaCoverImage>,
    description: Option<String>,
}

#[derive(cynic::QueryFragment)]
pub struct MediaList {
    id: Option<i32>,
    status: Option<MediaListStatus>,
    media: Option<Media>,
    progress: Option<i32>,
}

#[derive(cynic::QueryFragment)]
pub struct MediaListGroup {
    pub name: Option<String>,
    pub entries: Option<Vec<Option<MediaList>>>,
}

#[derive(cynic::QueryVariables)]
pub struct OngoingQueryParams {
    pub user_id: i32,
}

#[derive(cynic::QueryFragment)]
pub struct MediaListCollection {
    pub lists: Option<Vec<Option<MediaListGroup>>>,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "OngoingQueryParams")]
pub struct OngoingQuery {
    #[cynic(rename = "MediaListCollection")]
    #[arguments(userId: $user_id, status_in: [CURRENT, REPEATING], type: ANIME)]
    pub(crate) collection: Option<MediaListCollection>,
}

impl TryFrom<MediaList> for models::Media {
    type Error = AppError;

    fn try_from(entry: MediaList) -> Result<Self, Self::Error> {
        let media = entry
            .media
            .ok_or(AppError::NotFound("MediaList".to_string()))?;

        Ok(models::Media {
            id: media
                .id
                .try_into()
                .expect(format!("Media id ({}) failed to be casted!", media.id).as_str()),
            title: media
                .title
                .ok_or(AppError::InternalServer("Title not found".to_string()))?
                .to_string(),
            cover: media.cover_image.and_then(|cover| cover.try_into().ok()),
            banner: media.banner_image,
            description: media.description,
            media_type: models::MediaType::Anime,
            status: MediaStatus {
                status: entry.status.map(|status| status.into()),
                progress: entry.progress,
                total: media.episodes,
            },
        })
    }
}
