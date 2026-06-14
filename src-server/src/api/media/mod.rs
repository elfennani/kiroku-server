use crate::api::payloads::{DataResponse, ErrorResponse};
use crate::api::server::{AppRouter, RouterState};
use crate::domain::models::{EpisodeSummary, Media, MediaSummary};
use crate::errors::AppError;
use crate::infrastructure::anilist::client::AnilistClient;
use crate::infrastructure::anilist::queries::media_details::{
    MediaDetailsQuery, MediaDetailsQueryParams,
};
use crate::infrastructure::anilist::queries::ongoing::{OngoingQuery, OngoingQueryParams};
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router, http};
use cynic::{GraphQlResponse, QueryBuilder};
use log::error;

pub mod payloads;

pub fn create_media_router(state: RouterState) -> AppRouter {
    Router::new()
        .route("/", get(get_current_media))
        .route("/{media_id}", get(get_media_details))
        .with_state(state)
}

type MediaSummaryList = (http::StatusCode, Json<DataResponse<Vec<MediaSummary>>>);

pub async fn get_current_media(
    State(state): State<RouterState>,
) -> Result<MediaSummaryList, (http::StatusCode, Json<ErrorResponse>)> {
    let session = match state.session_repository.get_access_token().await? {
        None => {
            return Err((
                http::StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Not logged in!")),
            ));
        }
        Some(session) => session,
    };

    let request_body = OngoingQuery::build(OngoingQueryParams {
        user_id: session.user_id.try_into().unwrap(),
    });
    let client = AnilistClient::new(session.access_token.as_str());
    let response = client.post(&request_body).await.map_err(|err| {
        error!("Error sending request {}", err);
        (
            http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch media")),
        )
    })?;

    if !response.status().is_success() {
        error!(
            "retrieved request {} returned {:?}",
            response.status(),
            response.text().await.ok()
        );
        return Err((
            http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch media")),
        ));
    }

    let body: GraphQlResponse<OngoingQuery> = response.json().await.unwrap();

    match body.data {
        None => Err((
            http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to process data")),
        )),
        Some(data) => match data.collection {
            None => Err((
                http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to process data")),
            )),
            Some(collections) => {
                let lists = collections.lists.unwrap();
                let mut media_list: Vec<MediaSummary> = vec![];

                for list in lists {
                    let list = list.unwrap();

                    for entry in list.entries.unwrap() {
                        if let Some(entry) = entry {
                            media_list.push(entry.try_into()?)
                        }
                    }
                }

                let media_list_cloned = media_list.clone();

                tokio::spawn(async move {
                    if let Err(err) = state.media_repository.cache_media(&media_list_cloned).await {
                        error!("Error caching media: {}", err);
                    }
                });

                Ok((http::StatusCode::OK, Json(DataResponse::new(media_list))))
            }
        },
    }
}

pub async fn get_media_details(
    Path(media_id): Path<i64>,
    State(state): State<RouterState>,
) -> Result<Json<DataResponse<Media>>, (http::StatusCode, Json<ErrorResponse>)> {
    let session = match state.session_repository.get_access_token().await? {
        None => {
            return Err((
                http::StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Not logged in!")),
            ));
        }
        Some(session) => session,
    };

    let episodes = state
        .episode_repository
        .get_episodes_by_media_id(media_id)
        .await?
        .into_iter()
        .map(|ep| ep.use_server_urls())
        .collect::<Vec<EpisodeSummary>>();

    let request_body = MediaDetailsQuery::build(MediaDetailsQueryParams {
        id: media_id as i32,
    });
    let client = AnilistClient::new(session.access_token.as_str());
    let response = client.post(&request_body).await.map_err(|err| {
        error!("Error sending request {}", err);
        (
            http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch media")),
        )
    })?;

    if !response.status().is_success() {
        error!(
            "retrieved request {} returned {:?}",
            response.status(),
            response.text().await.ok()
        );
        return Err((
            http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to fetch media")),
        ));
    }

    let body: GraphQlResponse<MediaDetailsQuery> = response.json().await.unwrap();

    match body.data {
        None => {
            error!("MediaDetailsQuery returned None");
            Err(AppError::InternalServer("A".to_string()).into())
        }
        Some(data) => {
            let media: Media = data.try_into()?;

            Ok(Json(DataResponse::new(media.set_episodes(episodes))))
        }
    }
}
