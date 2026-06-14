mod payloads;

use crate::api::episode::payloads::EnqueueEpisodesRequest;
use crate::api::payloads::ErrorResponse;
use crate::api::server::ServerState;
use axum::extract::{ConnectInfo, State};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router, http};
use reqwest::StatusCode;
use std::net::SocketAddr;
use std::sync::Arc;

pub fn create_episode_router(state: Arc<ServerState>) -> Router<Arc<ServerState>> {
    Router::new()
        .route("/queue", post(queue_episode))
        .with_state(state)
}

async fn queue_episode(
    State(state): State<Arc<ServerState>>,
    ConnectInfo(socket): ConnectInfo<SocketAddr>,
    Json(data): Json<EnqueueEpisodesRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let queue_ids = state
        .episode_repository
        .enqueue(data.media_id, data.items)
        .await?;

    state.packager_service.enqueue(&queue_ids).await?;

    Ok(StatusCode::NO_CONTENT)
}
