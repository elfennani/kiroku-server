use crate::api::payloads::{AuthenticateParams, EnqueueVideo};
use crate::api::server::ServerState;
use crate::domain::models::{
    Image, Media, MediaDetails, MediaStatus, MediaType, ProcessedEpisode, User,
};
use crate::infrastructure::anilist;
use crate::infrastructure::anilist::client::AnilistClient;
use crate::infrastructure::anilist::media_details_query::MediaDetailsQueryMedia;
use crate::infrastructure::anilist::viewer_query::Variables;
use crate::infrastructure::anilist::{GraphQLResponse, ongoing_query, viewer_query};
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::{Json, http};
use graphql_client::GraphQLQuery;
use log::error;
use reqwest::StatusCode;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub async fn authenticate(
    Query(params): Query<AuthenticateParams>,
    State(state): State<Arc<ServerState>>,
) -> Result<impl IntoResponse, http::StatusCode> {
    let client = reqwest::Client::new();

    let mut body = HashMap::new();
    body.insert("grant_type", "authorization_code");
    body.insert("client_id", state.client_id.as_str());
    body.insert("client_secret", state.client_secret.as_str());
    body.insert("redirect_uri", "http://localhost:8642/authenticate");
    body.insert("code", params.code.as_str());

    let response = client
        .post("https://anilist.co/api/v2/oauth/token")
        .json(&body)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .send()
        .await;

    #[derive(Deserialize)]
    struct ExchangeResponse {
        access_token: String,
    }

    if let Ok(response) = response {
        let data: ExchangeResponse = response.json().await.unwrap();
        state
            .session_repository
            .save_access_token(data.access_token)?;

        println!("Session saved!");
        Ok("Session successfully logged in.")
    } else {
        Err(http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub async fn login(State(state): State<Arc<ServerState>>) -> Result<Redirect, http::StatusCode> {
    if state
        .session_repository
        .get_access_token()
        .is_ok_and(|s| s.is_some())
    {
        return Err(http::StatusCode::BAD_REQUEST);
    }

    let mut url = url::Url::parse(
        format!(
            "https://anilist.co/api/v2/oauth/authorize?client_id={}&response_type=code",
            state.client_id
        )
        .as_str(),
    )
    .unwrap();

    url.query_pairs_mut()
        .append_pair("redirect_uri", "http://localhost:8642/authenticate");

    println!("Redirecting to {}", url);
    Ok(Redirect::permanent(url.as_str()))
}

pub async fn profile(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<User>, http::StatusCode> {
    let access_token = match state.session_repository.get_access_token()? {
        None => return Err(http::StatusCode::UNAUTHORIZED),
        Some(token) => token,
    };

    let request_body = anilist::ViewerQuery::build_query(Variables);
    let client = AnilistClient::new(access_token.as_str());
    let response = client.post(&request_body).await.map_err(|err| {
        eprintln!("Error sending request {}", err);
        http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if !response.status().is_success() {
        return Err(http::StatusCode::BAD_REQUEST);
    }

    let body: GraphQLResponse<viewer_query::ResponseData> = response.json().await.unwrap();

    match body.into_inner().data {
        Some(data) => match data.viewer {
            None => Err(http::StatusCode::BAD_REQUEST),
            Some(viewer) => {
                let user = User {
                    id: viewer.id as i32,
                    name: viewer.name,
                    avatar_url: viewer.avatar.and_then(|avatar| avatar.large),
                    banner_url: viewer.banner_image,
                    description: None,
                };

                if let Err(err) = state.user_repository.save_user(&user, true) {
                    eprintln!("Error saving user {:?}", err);
                }

                Ok(Json::from(user))
            }
        },
        _ => Err(http::StatusCode::BAD_REQUEST),
    }
}

pub async fn get_ongoing_media(
    State(state): State<Arc<ServerState>>,
) -> Result<impl IntoResponse, http::StatusCode> {
    let user = match state.user_repository.get_viewer_user()? {
        None => {
            // TODO: Create an AniList service that encapsulates the request logic to fetch anywhere.
            return Err(http::StatusCode::UNAUTHORIZED);
        }
        Some(user) => user,
    };

    let access_token = match state.session_repository.get_access_token()? {
        None => return Err(http::StatusCode::UNAUTHORIZED),
        Some(token) => token,
    };

    let request_body = anilist::OngoingQuery::build_query(ongoing_query::Variables {
        user_id: user.id.into(),
    });
    let client = AnilistClient::new(access_token.as_str());
    let response = client.post(&request_body).await.map_err(|err| {
        eprintln!("Error sending request {}", err);
        http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if !response.status().is_success() {
        return Err(http::StatusCode::BAD_REQUEST);
    }

    let body: GraphQLResponse<ongoing_query::ResponseData> = response.json().await.unwrap();

    match body.into_inner().data {
        None => Err(http::StatusCode::BAD_REQUEST),
        Some(data) => match data.media_list_collection {
            None => Err(http::StatusCode::BAD_REQUEST),
            Some(collections) => {
                let lists = collections.lists.unwrap();
                let mut media_list: Vec<Media> = vec![];

                for list in lists {
                    let list = list.unwrap();

                    for entry in list.entries.unwrap() {
                        let entry = entry.unwrap();
                        let media = entry.media.unwrap();

                        media_list.push(Media {
                            id: media.id.try_into().unwrap(),
                            title: media.title.unwrap().user_preferred.unwrap(),
                            cover: media.cover_image.and_then(|cover| {
                                if cover.large.is_none() || cover.extra_large.is_none() {
                                    None
                                } else {
                                    Some(Image {
                                        thumbnail: cover.large.unwrap(),
                                        url: cover.extra_large.unwrap(),
                                        width: None,
                                        height: None,
                                    })
                                }
                            }),
                            banner: media.banner_image,
                            description: None,
                            media_type: MediaType::Anime,
                            status: MediaStatus {
                                status: entry.status.map(|status| {
                                    serde_json::to_value(&status).unwrap().try_into().unwrap()
                                }),
                                progress: entry.progress.map(|progress| progress as i32),
                                total: media.episodes.map(|eps| eps as i32),
                            },
                        })
                    }
                }

                Ok(Json(media_list))
            }
        },
    }
}

pub async fn enqueue_process(
    State(state): State<Arc<ServerState>>,
    Json(input): Json<EnqueueVideo>,
) -> Result<impl IntoResponse, http::StatusCode> {
    if let Err(err) = state
        .packager_service
        .enqueue(input.path.into(), input.media_id, input.episode)
        .await
    {
        error!("Enqueue route error: {}", err);
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct Params {
    media_id: usize,
}

pub async fn get_media_details(
    State(state): State<Arc<ServerState>>,
    Path(Params { media_id }): Path<Params>,
) -> Result<Json<MediaDetails>, http::StatusCode> {
    let processed_eps = state
        .media_processor_repo
        .get_processed_media_by_media_id(media_id)?;

    let access_token = match state.session_repository.get_access_token()? {
        None => return Err(StatusCode::UNAUTHORIZED),
        Some(token) => token,
    };

    let client = AnilistClient::new(access_token.as_str());
    let req = anilist::MediaDetailsQuery::build_query(anilist::media_details_query::Variables {
        id: media_id.try_into().unwrap(),
    });
    let response = client.post(&req).await.map_err(|err| {
        eprintln!("Error sending request {}", err);
        http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if !response.status().is_success() {
        eprintln!("Error: {}", response.text().await.unwrap());
        return Err(http::StatusCode::BAD_REQUEST);
    }

    let body: GraphQLResponse<anilist::media_details_query::ResponseData> =
        response.json().await.unwrap();

    match body.into_inner().data {
        None => Err(http::StatusCode::BAD_REQUEST),
        Some(data) => match data.media {
            None => return Err(http::StatusCode::NOT_FOUND),
            Some(media) => {
                let mut processed_eps_with_metadata = vec![];

                for ep in processed_eps {
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

                Ok(Json(MediaDetails {
                    id: media.id.try_into().unwrap(),
                    title: media
                        .title
                        .and_then(|title| {
                            title
                                .english
                                .or(title.user_preferred)
                                .or(title.romaji)
                                .or(title.native)
                        })
                        .unwrap(),
                    description: media.description,
                    cover: media.cover_image.and_then(|cover| {
                        if cover.large.is_none() || cover.extra_large.is_none() {
                            return None;
                        }

                        Some(Image {
                            thumbnail: cover.large.unwrap(),
                            url: cover.extra_large.unwrap(),
                            width: None,
                            height: None,
                        })
                    }),
                    banner: media.banner_image,
                    status: match media.media_list_entry {
                        Some(entry) => MediaStatus {
                            status: entry.status.map(|status| {
                                serde_json::to_value(&status).unwrap().try_into().unwrap()
                            }),
                            progress: entry.progress.map(|progress| progress as i32),
                            total: media.episodes.map(|eps| eps as i32),
                        },
                        None => MediaStatus {
                            status: None,
                            progress: None,
                            total: media.episodes.map(|eps| eps as i32),
                        },
                    },
                    episodes: processed_eps_with_metadata,
                }))
            }
        },
    }
    // Ok(Json(processed_eps))
}
