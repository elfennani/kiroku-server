use crate::api::server::Server;
use crate::config::Config;
use crate::infrastructure::database::connection::Database;
use crate::infrastructure::packager::service::PackagerService;
use directories::ProjectDirs;
use log::info;
use serde::Deserialize;
use std::env;
use std::sync::Arc;

mod api;
mod domain;
// pub mod errors;
mod config;
mod errors;
mod infrastructure;
mod prelude;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let project_dirs = ProjectDirs::from("com.elfen", "", "kiroku-server").unwrap();
    let args: Vec<String> = env::args().collect();

    let config_file = project_dirs.config_local_dir().join("config.toml");

    if !config_file.exists() {
        panic!("Config file does not exist {}", config_file.display());
    }

    let config = Config::from_file(config_file)?;

    let db = Arc::new(
        Database::open(project_dirs.data_dir().join("./app.db"))
            .await
            .expect("Failed to open database."),
    );

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

    let client_id = config.anilist.client_id;
    let client_secret = config.anilist.client_secret;

    info!("Starting Server");
    let app = Server::new(
        db.clone(),
        packager_service.clone(),
        client_id,
        client_secret.as_str(),
    );
    app.serve().await?;

    Ok(())
}
