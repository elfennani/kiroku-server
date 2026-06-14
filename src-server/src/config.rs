use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    pub anilist: AniList,
}

#[derive(Deserialize)]
pub struct AniList {
    #[serde(rename = "client-id")]
    pub client_id: i32,
    #[serde(rename = "client-secret")]
    pub client_secret: String,
}

impl Config {
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let config = std::fs::read_to_string(path)?;
        let config = toml::from_str::<Self>(config.as_str())?;

        Ok(config)
    }
}
