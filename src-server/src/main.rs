use crate::api::server::Server;
use crate::infrastructure::packager::service::PackagerService;
use directories::ProjectDirs;
use infrastructure::database::Database;
use infrastructure::database::migration::Migration;
use log::info;
use std::env;
use std::sync::Arc;

mod api;
mod domain;
pub mod errors;
mod infrastructure;
mod prelude;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let project_dirs = ProjectDirs::from("com.elfen", "", "kiroku-server").unwrap();
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
        Migration::new(
            3,
            // language=sqlite
            "
                CREATE TABLE IF NOT EXISTS metadata (
                    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL,
                    duration INT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS chapters (
                    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                    metadata_id INTEGER NOT NULL REFERENCES metadata (id),
                    title TEXT,
                    start INT NOT NULL,
                    end INT NOT NULL,
                    `index` INT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS streams (
                    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                    metadata_id INTEGER NOT NULL REFERENCES metadata (id),
                    title TEXT NOT NULL,
                    language TEXT NOT NULL,
                    `index` INT NOT NULL,
                    type TEXT NOT NULL CHECK ( type in ('audio', 'subtitle') )
                );
                CREATE TABLE IF NOT EXISTS processing_queue (
                    uuid TEXT NOT NULL PRIMARY KEY,
                    metadata_id INTEGER NOT NULL REFERENCES metadata (id),
                    status TEXT NOT NULL CHECK ( status in ('queued', 'processing', 'done') ),
                    path TEXT NOT NULL, -- Output directory where all files are generated
                    playlist_path TEXT, -- The path to generated m3u8 file.
                    input_path TEXT -- The path to input file to be processed.
                );
                CREATE TABLE IF NOT EXISTS processed_files (
                    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                    processing_id TEXT NOT NULL REFERENCES processing_queue (uuid),
                    filename TEXT NOT NULL,
                    path TEXT NOT NULL,
                    type TEXT NOT NULL CHECK ( type in ('audio', 'subtitle', 'video') )
                );
                CREATE TABLE IF NOT EXISTS media_metadata (
                    metadata_id INTEGER NOT NULL REFERENCES metadata (id),
                    media_id INTEGER NOT NULL,
                    episode REAL NOT NULL,

                    PRIMARY KEY (metadata_id, media_id, episode)
                );
            ",
        )?,
    ];

    let db = Arc::new(Database::open(
        project_dirs.data_dir().join("./app.db"),
        migrations,
    )?);

    let packager_dir = project_dirs.data_dir().join("packager");

    if !packager_dir.exists() {
        std::fs::create_dir_all(&packager_dir)?;
    } else if !packager_dir.is_dir() {
        panic!("{} is not a directory", packager_dir.display());
    }

    let packager_service = Arc::new(PackagerService::new(db.clone(), packager_dir));

    {
        let packager_service = packager_service.clone();
        tokio::spawn(async move {
            packager_service.start().await.ok();
        });
    }

    let client_id = args[1].clone();
    let client_secret = args[2].clone();
    println!("Client ID: {}", client_id);
    env_logger::init();

    info!("Starting Server");
    let app = Server::new(
        db.clone(),
        packager_service.clone(),
        client_id.as_str(),
        client_secret.as_str(),
    );
    app.serve().await?;

    Ok(())
}
