use crate::api::handlers::{authenticate, get_ongoing_media, login, profile};
use crate::api::server::ServerState;
use axum::Router;
use axum::routing::get;
use std::sync::Arc;

pub fn create_router(state: Arc<ServerState>) -> Router {
    Router::new()
        .route("/authenticate", get(authenticate))
        .route("/login", get(login))
        .route("/user/me", get(profile))
        .route("/media/ongoing", get(get_ongoing_media))
        .with_state(state)
}
