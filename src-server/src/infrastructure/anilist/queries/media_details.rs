use crate::domain::models;
use crate::errors::AppError;
use crate::infrastructure::anilist::queries::fragments::*;
use crate::infrastructure::anilist::schema;

#[derive(cynic::QueryFragment)]
struct MediaList {
    status: Option<MediaListStatus>,
    progress: Option<i32>,
}

#[derive(cynic::QueryFragment)]
pub struct Media {
    id: i32,
    title: Option<MediaTitle>,
    description: Option<String>,
    banner_image: Option<String>,
    episodes: Option<i32>,
    cover_image: Option<MediaCoverImage>,
    media_list_entry: Option<MediaList>,
    genres: Option<Vec<Option<String>>>,
}

#[derive(cynic::QueryVariables)]
pub struct MediaDetailsQueryParams {
    pub(crate) id: i32,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "MediaDetailsQueryParams")]
pub struct MediaDetailsQuery {
    #[cynic(rename = "Media")]
    #[arguments(id: $id)]
    pub media: Option<Media>,
}


impl TryFrom<MediaDetailsQuery> for models::Media {
    type Error = AppError;

    fn try_from(query: MediaDetailsQuery) -> Result<Self, Self::Error> {
        let media = query
            .media
            .ok_or(AppError::NotFound("Media not found".to_string()))?;
        let entry = media.media_list_entry;

        Ok(models::Media {
            id: media.id.try_into().unwrap(),
            banner: media.banner_image,
            cover: media.cover_image.and_then(|cover| cover.try_into().ok()),
            title: media.title.map(|title| title.to_string()).unwrap(),
            description: media.description,
            progress: entry
                .as_ref()
                .and_then(|media_list_entry| media_list_entry.progress.map(|p| p as u32)),
            total: media.episodes.map(|eps| eps.try_into().unwrap()),
            status: entry.and_then(|mle| mle.status).map(|m| m.into()),
            genres: media
                .genres
                .unwrap_or(vec![])
                .iter()
                .filter_map(|g| g.as_ref().and_then(|g| String::try_from(g).ok()))
                .collect(),
            episodes: vec![],
        })
    }
}
