use crate::prelude::*;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, SqliteConnection};
use std::path::PathBuf;
use tokio::sync::Mutex;

pub struct Database {
    pub conn: Mutex<SqliteConnection>,
}

impl Database {
    pub async fn open(path: PathBuf) -> Result<Database> {
        let opts = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true);
        let conn = SqliteConnection::connect_with(&opts).await?;

        Ok(Database {
            conn: Mutex::new(conn),
        })
    }
}
