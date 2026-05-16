use crate::api::server::Server;
use infrastructure::database::Database;
use infrastructure::database::migration::Migration;
use std::env;
use std::sync::Arc;

mod api;
mod domain;
pub mod errors;
mod infrastructure;
mod prelude;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let migrations: Vec<Migration> = vec![
        Migration::new(
            1, // language=sqlite
            "
            CREATE TABLE sessions (
                id INT NOT NULL PRIMARY KEY,
                token TEXT NOT NULL
            );
        ",
        )?,
        Migration::new(
            2,
            // language=sqlite
            "
            CREATE TABLE users (
                id INT NOT NULL PRIMARY KEY,
                name TEXT NOT NULL,
                avatar_url TEXT,
                banner_url TEXT,
                description TEXT,
                is_viewer INT NOT NULL DEFAULT 0
            );
        ",
        )?,
    ];

    let db = Arc::new(Database::open(
        env::current_dir()?.join("app.db"),
        migrations,
    )?);

    let client_id = args[1].clone();
    let client_secret = args[2].clone();
    println!("Client ID: {}", client_id);
    let app = Server::new(db.clone(), client_id.as_str(), client_secret.as_str());
    app.serve().await?;

    Ok(())
}
