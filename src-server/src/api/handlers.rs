use crate::api::payloads::{AuthenticateParams, EnqueueVideo};
use crate::api::server::ServerState;
use crate::domain::models::{Media, MediaDetails, MediaStatus, MediaType, ProcessedEpisode, User};
use crate::infrastructure::anilist::client::AnilistClient;
use crate::infrastructure::anilist::queries::media_details;
use crate::infrastructure::anilist::queries::media_details::MediaDetailsQueryParams;
use crate::infrastructure::anilist::queries::ongoing::{OngoingQuery, OngoingQueryParams};
use crate::infrastructure::anilist::queries::viewer::ViewerQuery;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::{Json, http};
use cynic::{GraphQlResponse, QueryBuilder};
use log::error;
use reqwest::StatusCode;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

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

    let req = ViewerQuery::build(());
    let client = AnilistClient::new(access_token.as_str());
    let response = client.post(&req).await.map_err(|err| {
        eprintln!("Error sending request {}", err);
        http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if !response.status().is_success() {
        return Err(http::StatusCode::BAD_REQUEST);
    }

    let body = response
        .json::<GraphQlResponse<ViewerQuery>>()
        .await
        .unwrap();

    match body.data {
        Some(data) => match data.viewer {
            None => Err(http::StatusCode::BAD_REQUEST),
            Some(viewer) => {
                let user: User = viewer.into();

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

    let request_body = OngoingQuery::build(OngoingQueryParams {
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

    let body: GraphQlResponse<OngoingQuery> = response.json().await.unwrap();

    match body.data {
        None => Err(http::StatusCode::BAD_REQUEST),
        Some(data) => match data.collection {
            None => Err(http::StatusCode::BAD_REQUEST),
            Some(collections) => {
                let lists = collections.lists.unwrap();
                let mut media_list: Vec<Media> = vec![];

                for list in lists {
                    let list = list.unwrap();

                    for entry in list.entries.unwrap() {
                        if let Some(entry) = entry {
                            media_list.push(entry.try_into()?)
                        }
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
    let req = media_details::MediaDetailsQuery::build(MediaDetailsQueryParams {
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

    let body: GraphQlResponse<media_details::MediaDetailsQuery> = response.json().await.unwrap();

    match body.data {
        None => Err(http::StatusCode::BAD_REQUEST),
        Some(data) => {
            let processed_eps = data.update_processed_episodes_metadata(processed_eps);
            let mut media: MediaDetails = data.try_into()?;
            media.set_episodes(processed_eps?);

            Ok(Json(media))
        }
    }
}
