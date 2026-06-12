use crate::domain::models;
use crate::domain::models::ProcessedEpisode;
use crate::errors::AppError;
use crate::infrastructure::anilist::queries::fragments::*;
use crate::infrastructure::anilist::schema;

#[derive(cynic::QueryFragment)]
struct MediaList {
    status: Option<MediaListStatus>,
    progress: Option<i32>,
}

#[derive(cynic::QueryFragment)]
struct MediaStreamingEpisode {
    title: Option<String>,
    thumbnail: Option<String>,
}

#[derive(cynic::QueryFragment)]
pub struct Media {
    id: i32,
    title: Option<MediaTitle>,
    #[cynic(rename = "type")]
    media_type: Option<MediaType>,
    description: Option<String>,
    banner_image: Option<String>,
    episodes: Option<i32>,
    cover_image: Option<MediaCoverImage>,
    media_list_entry: Option<MediaList>,
    streaming_episodes: Option<Vec<Option<MediaStreamingEpisode>>>,
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
    media: Option<Media>,
}

impl MediaDetailsQuery {
    pub fn update_processed_episodes_metadata(
        &self,
        processed_episodes: Vec<ProcessedEpisode>,
    ) -> Result<Vec<ProcessedEpisode>, AppError> {
        let mut processed_eps_with_metadata = vec![];
        let media = self
            .media
            .as_ref()
            .ok_or(AppError::NotFound("Media not found".to_string()))?;

        for ep in processed_episodes {
            let mut title = None::<String>;
            let mut thumbnail = None::<String>;

            if let Some(streaming_eps) = media.streaming_episodes.as_ref() {
                let streaming_ep = streaming_eps
                    .iter()
                    .filter(|ep| ep.is_some())
                    .map(|ep| ep.as_ref().unwrap())
                    .find(|ep| ep.title.is_some());

                if let Some(streaming_ep) = streaming_ep {
                    let ep_title = streaming_ep.title.as_ref().unwrap().as_str();
                    if ep_title.starts_with(format!("Episode {} -", ep.episode).as_str()) {
                        title = Some(ep_title.to_string());
                    }
                    let str_thumbnail = streaming_ep.thumbnail.as_ref();

                    if let Some(str_thumbnail) = str_thumbnail {
                        thumbnail = Some(str_thumbnail.clone());
                    }
                }
            }

            processed_eps_with_metadata.push(ProcessedEpisode {
                id: ep.id,
                episode: ep.episode,
                duration: ep.duration,
                title,
                thumbnail,
            });
        }

        Ok(processed_eps_with_metadata)
    }
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
