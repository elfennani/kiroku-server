use crate::api::auth::create_auth_router;
use crate::api::media::create_media_router;
use crate::api::server::ServerState;
use axum::Router;
use axum::routing::{get, post};
use std::sync::Arc;

pub fn create_router(state: Arc<ServerState>) -> Router {
    Router::new()
        .nest("/auth", create_auth_router(state.clone()))
        .nest("/media", create_media_router(state.clone()))
        .with_state(state)
}
