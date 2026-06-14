use crate::api::auth::payloads::AuthenticateParams;
use crate::api::payloads::ErrorResponse;
use crate::api::server::ServerState;
use crate::infrastructure::anilist::client::AnilistClient;
use crate::infrastructure::anilist::queries::viewer::ViewerQuery;
use axum::extract::rejection::QueryRejection;
use axum::extract::{ConnectInfo, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get};
use axum::{Json, Router};
use cynic::{GraphQlResponse, QueryBuilder};
use log::{error, info, warn};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

pub mod payloads;

pub fn create_auth_router(state: Arc<ServerState>) -> Router<Arc<ServerState>> {
    Router::new()
        .route("/login", get(login))
        .route("/token", get(authenticate))
        .with_state(state)
}

const REDIRECT_URI: &str = "http://localhost:8642/api/auth/token";

async fn authenticate(
    query: Result<Query<AuthenticateParams>, QueryRejection>,
    State(state): State<Arc<ServerState>>,
    ConnectInfo(socket): ConnectInfo<SocketAddr>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    if !socket.ip().is_loopback() {
        warn!("Request must be made through the host device.");
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new(
                "Request must be made through the host device.",
            )),
        ));
    }

    let params = match query {
        Ok(Query(param)) => param,
        Err(_) => {
            warn!("code query parameter is required.");
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("\"code\" query parameter missing")),
            ));
        }
    };

    if params.code.is_empty() {
        warn!("code must not be empty.");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("\"code\" must not be empty")),
        ));
    }

    let client = reqwest::Client::new();

    let client_id = state.client_id.to_string();
    let mut body = HashMap::new();
    body.insert("grant_type", "authorization_code");
    body.insert("client_id", client_id.as_str());
    body.insert("client_secret", state.client_secret.as_str());
    body.insert("redirect_uri", REDIRECT_URI);
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
        if response.status() == StatusCode::BAD_REQUEST {
            if let Ok(text) = response.text().await {
                error!("Got bad request in token exchange: {}", text);
            }
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Something went wrong.")),
            ));
        }

        let data: ExchangeResponse = response.json().await.map_err(|err| {
            error!("Unexpected parsing error: {:?}", err);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!(
                    "Failed to decode exchange response: {}",
                    err
                ))),
            )
        })?;

        // Fetch user ID
        let client = AnilistClient::new(data.access_token.as_str());
        let query = ViewerQuery::build(());
        let response = client.post(&query).await.map_err(|err| {
            error!("Unexpected error: {:?}", err);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Something went wrong.")),
            )
        })?;

        let user_id = match response.json::<GraphQlResponse<ViewerQuery>>().await {
            Ok(data) => match data.data {
                None => {
                    error!("Viewer data empty");

                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new("Something went wrong.")),
                    ));
                }
                Some(viewer) => viewer.viewer.unwrap().id,
            },
            Err(err) => {
                error!("Unexpected error: {:?}", err);

                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("Something went wrong.")),
                ));
            }
        };

        state
            .session_repository
            .save_access_token(data.access_token, user_id as u32)
            .await
            .map_err(|e| {
                error!("Failed to save access token: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("Failed to save access token")),
                )
            })?;

        info!("Session saved");
        Ok(StatusCode::NO_CONTENT)
    } else {
        error!("Request failed: {:?}", response.unwrap_err());
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Failed to authenticate")),
        ))
    }
}

async fn login(
    State(state): State<Arc<ServerState>>,
    ConnectInfo(socket): ConnectInfo<SocketAddr>,
) -> Result<Redirect, impl IntoResponse> {
    if !socket.ip().is_loopback() {
        warn!("Request must be made through the host device.");
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new(
                "Request must be made through the host device.",
            )),
        ));
    }

    if state
        .session_repository
        .get_access_token()
        .await
        .is_ok_and(|s| s.is_some())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("User already logged in.")),
        ));
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
        .append_pair("redirect_uri", REDIRECT_URI);

    println!("Redirecting to {}", url);
    Ok(Redirect::permanent(url.as_str()))
}
