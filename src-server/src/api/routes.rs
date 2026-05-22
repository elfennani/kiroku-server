use crate::api::handlers::*;
use crate::api::server::ServerState;
use axum::Router;
use axum::routing::{get, post};
use std::sync::Arc;

pub fn create_router(state: Arc<ServerState>) -> Router {
    Router::new()
        .route("/authenticate", get(authenticate))
        .route("/login", get(login))
        .route("/user/me", get(profile))
        .route("/media/ongoing", get(get_ongoing_media))
        .route("/enqueue", post(enqueue_process))
        .route(
            "/media/{media_id}",
            get(get_media_details),
        )
        .with_state(state)
}
