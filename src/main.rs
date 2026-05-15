use crate::database::migration::Migration;
use crate::database::Database;
use anyhow::Context;
use axum::extract::Query;
use axum::response::{IntoResponse, Redirect};
use axum::{extract::State, http, routing::get, Router};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;

mod database;

struct AppState {
    database: Arc<Database>,
    client_id: String,
    client_secret: String,
}

struct App {
    state: Arc<AppState>,
}

#[derive(Deserialize)]
struct AuthenticateParams {
    code: String,
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
