use crate::anilist::client::AnilistClient;
use crate::anilist::viewer_query::Variables;
use crate::anilist::{GraphQLResponse, ongoing_query, viewer_query};
use crate::database::Database;
use crate::database::migration::Migration;
use anyhow::Context;
use axum::extract::Query;
use axum::response::{IntoResponse, Redirect};
use axum::{Json, Router, extract::State, http, routing::get};
use graphql_client::GraphQLQuery;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;

mod anilist;
mod database;

struct AppState {
    database: Arc<Database>,
    client_id: String,
    client_secret: String,
}

impl AppState {
    fn get_access_token(&self) -> Result<String, http::StatusCode> {
        let session = self.database.get_session().map_err(|err| {
            eprintln!("Error getting session: {}", err);
            http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

        match session {
            Some(session) => Ok(session.access_token),
            None => Err(axum::http::StatusCode::UNAUTHORIZED),
        }
    }
}

struct App {
    state: Arc<AppState>,
}

#[derive(Deserialize)]
struct AuthenticateParams {
    code: String,
}

#[derive(Serialize, Deserialize)]
struct Media {
    id: i32,
    title: String,
    status: Option<String>,
    cover: Option<String>,
    banner: Option<String>,
    progress: Option<i32>,
    total: Option<i32>,
}

#[derive(Serialize, Deserialize)]
struct Profile {
    id: i32,
    name: String,
    avatar_url: Option<String>,
    banner_url: Option<String>,
    media: Vec<Media>,
}

impl App {
    pub fn new(db: Arc<Database>, client_id: &str, client_secret: &str) -> Self {
        Self {
            state: Arc::new(AppState {
                database: db,
                client_id: client_id.to_owned(),
                client_secret: client_secret.to_owned(),
            }),
        }
    }

    pub async fn serve(&self) -> anyhow::Result<()> {
        let app = Router::new()
            .route("/authenticate", get(Self::authenticate))
            .route("/login", get(Self::login))
            .route("/profile", get(Self::profile))
            .with_state(self.state.clone());
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8642")
            .await
            .context("failed to bind TCP listener")?;

        println!("Listening on {}", listener.local_addr()?);
        axum::serve(listener, app)
            .await
            .context("error serving axum server")?;

        Ok(())
    }

    async fn authenticate(
        Query(params): Query<AuthenticateParams>,
        State(state): State<Arc<AppState>>,
    ) -> Result<impl IntoResponse, axum::http::StatusCode> {
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
                .database
                .save_session(data.access_token.as_str())
                .unwrap();

            println!("Session saved!");
            Ok("Session successfully logged in.")
        } else {
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }

    async fn login(State(state): State<Arc<AppState>>) -> Result<Redirect, axum::http::StatusCode> {
        if state.database.get_session().is_ok_and(|s| s.is_some()) {
            return Err(axum::http::StatusCode::BAD_REQUEST);
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
    async fn profile(
        State(state): State<Arc<AppState>>,
    ) -> Result<Json<Profile>, axum::http::StatusCode> {
        let access_token = state.get_access_token()?;

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
                Some(viewer) => Ok(Json::from(Profile {
                    id: viewer.id.try_into().unwrap(),
                    name: viewer.name,
                    avatar_url: viewer.avatar.map(|avatar| avatar.large).flatten(),
                    banner_url: viewer.banner_image,
                    media: Self::get_user_media(&state, viewer.id as i32).await?,
                })),
            },
            _ => Err(http::StatusCode::BAD_REQUEST),
        }
    }

    async fn get_user_media(
        state: &AppState,
        user_id: i32,
    ) -> Result<Vec<Media>, http::StatusCode> {
        let access_token = state.get_access_token()?;
        let request_body = anilist::OngoingQuery::build_query(ongoing_query::Variables {
            user_id: user_id.try_into().unwrap(),
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
                                id: entry.id.try_into().unwrap(),
                                title: media.title.unwrap().user_preferred.unwrap(),
                                status: entry.status.map(|status| status.to_string()),
                                cover: media.cover_image.map(|cover| cover.extra_large).flatten(),
                                banner: media.banner_image,
                                progress: entry.progress.map(|progress| progress as i32),
                                total: media.episodes.map(|eps| eps as i32),
                            })
                        }
                    }

                    Ok(media_list)
                }
            },
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let migrations: Vec<Migration> = vec![Migration::new(
        1, // language=sqlite
        "
            CREATE TABLE sessions (
                id INT NOT NULL PRIMARY KEY,
                token TEXT NOT NULL
            );
        ",
    )?];

    let db = Arc::new(Database::open(
        env::current_dir()?.join("app.db"),
        migrations,
    )?);

    let client_id = args[1].clone();
    let client_secret = args[2].clone();
    println!("Client ID: {}", client_id);
    let app = App::new(db.clone(), client_id.as_str(), client_secret.as_str());
    app.serve().await?;

    Ok(())
}
