mod payloads;

use crate::api::episode::payloads::{EnqueueEpisodesRequest, GetQueueResponseItem};
use crate::api::payloads::{DataResponse, ErrorResponse};
use crate::api::server::ServerState;
use crate::domain::models::Episode;
use crate::errors::AppError;
use axum::body::Body;
use axum::extract::{ConnectInfo, Path, State};
use axum::http::{HeaderMap, Response};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router, http};
use local_ip_address::local_ip;
use log::debug;
use reqwest::StatusCode;
use std::net::SocketAddr;
use std::sync::Arc;

pub fn create_episode_router(state: Arc<ServerState>) -> Router<Arc<ServerState>> {
    Router::new()
        .route("/queue", post(queue_episode))
        .route("/queue", get(get_queue))
        .route("/{id}", get(get_episode_details))
        .route("/{id}/playlist.m3u8", get(get_episode_playlist))
        .route("/{id}/{*path}", get(get_episode_playlist_file))
        .with_state(state)
}

async fn queue_episode(
    State(state): State<Arc<ServerState>>,
    ConnectInfo(socket): ConnectInfo<SocketAddr>,
    Json(data): Json<EnqueueEpisodesRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Prefetch and cache media (return 404 if not found)
    // TODO: Check whether each episode exists (return 400 if at least one is invalid)
    // TODO: Restrict enqueuing to requests made through the loopback.

    let queue_ids = state
        .episode_repository
        .enqueue(data.media_id, data.items)
        .await?;

    state.packager_service.enqueue(&queue_ids).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_queue(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<DataResponse<Vec<GetQueueResponseItem>>>, (StatusCode, Json<ErrorResponse>)> {
    let queue_items = state.episode_repository.get_queue_items().await?;
    let media = state.media_repository.get_cached_media().await?;

    let mut items: Vec<GetQueueResponseItem> = vec![];

    for queue_item in queue_items {
        items.push(GetQueueResponseItem {
            id: queue_item.id,
            step: queue_item.step,
            progress: queue_item.progress,
            media: media
                .iter()
                .find(|m| (m.id as i64) == queue_item.media_id)
                .ok_or(AppError::NotFound("".to_string()))?
                .clone(),
            number: queue_item.episode_number,
        })
    }

    Ok(Json(DataResponse::new(items)))
}

async fn get_episode_details(
    Path(id): Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Result<Json<DataResponse<Episode>>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Fetching episode {}", id);
    let episode = match state.episode_repository.get_episode_by_id(&id).await? {
        Some(episode) => episode,
        None => return Err(AppError::NotFound("Episode not found!".to_string()).into()),
    };
    let local_ip =
        local_ip().map_err(|_| AppError::InternalServer("Failed to get local ip".to_string()))?;

    Ok(Json(DataResponse::new(Episode {
        url: format!("http://{}:8642/api/episodes/{}/playlist.m3u8", local_ip, id),
        ..episode
    })))
}

async fn get_episode_playlist(
    Path(id): Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Result<(HeaderMap, Body), (StatusCode, Json<ErrorResponse>)> {
    let episode = match state.episode_repository.get_episode_by_id(&id).await? {
        Some(episode) => episode,
        None => return Err(AppError::NotFound("Episode not found!".to_string()).into()),
    };

    let content = std::fs::read_to_string(episode.url)
        .map_err(|_| AppError::NotFound("Episode file not found!".to_string()))?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/vnd.apple.mpegurl".parse().unwrap(),
    );

    Ok((headers, Body::from(content)))
}

async fn get_episode_playlist_file(
    Path((id, path)): Path<(String, String)>,
    State(state): State<Arc<ServerState>>,
) -> Result<Body, StatusCode> {
    let file = state.packager_service.output_dir().join(id).join(path);
    let content = std::fs::read_to_string(file)
        .map_err(|_| AppError::NotFound("Episode file not found!".to_string()))?;

    Ok(Body::from(content))
}
