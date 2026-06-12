use crate::api::auth::create_auth_router;
use crate::api::handlers::*;
use crate::api::media::create_media_router;
use crate::api::server::ServerState;
use axum::Router;
use axum::routing::{get, post};
use std::sync::Arc;

pub fn create_router(state: Arc<ServerState>) -> Router {
    Router::new()
        .nest("/auth", create_auth_router(state.clone()))
        .nest("/media", create_media_router(state.clone()))
        .route("/authenticate", get(authenticate))
        .route("/login", get(login))
        .route("/user/me", get(profile))
        .route("/media/ongoing", get(get_ongoing_media))
        .route("/enqueue", post(enqueue_process))
        .route(
            "/episode/{id}/files/playlist.m3u8",
            get(get_episode_playlist),
        )
        .route(
            "/episode/{id}/files/{*path}",
            get(get_episode_playlist_file),
        )
        .route("/media/{media_id}", get(get_media_details))
        .with_state(state)
}
