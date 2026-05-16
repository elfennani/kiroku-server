pub mod migration;
pub mod table;

use crate::infrastructure::database::migration::Migration;
use crate::infrastructure::database::table::Session;
use anyhow::Context;
use rusqlite::{params, Connection, OpenFlags, Result};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct Database {
    pub connection: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open(db_file: PathBuf, migrations: Vec<Migration>) -> anyhow::Result<Database> {
        let mut conn = Connection::open_with_flags(
            db_file,
            OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
        )
        .context("Connection to database failed!")?;

        conn.execute(
            // language=sqlite
            "
            CREATE TABLE IF NOT EXISTS migrations (
              version INTEGER PRIMARY KEY
            )
        ",
            (),
        )
        .context("Failed to create migrations table")?;

        let version = Self::get_version(&conn)?;

        for migration in migrations {
            if migration.version <= version {
                continue;
            }

            migration.execute(&mut conn)?
        }

        Ok(Database {
            connection: Arc::new(Mutex::new(conn)),
        })
    }

    fn get_version(conn: &Connection) -> anyhow::Result<i32> {
        let mut stmt =
            conn.prepare("SELECT version FROM migrations ORDER BY version DESC LIMIT 1")?;
        let version: Result<i32> = stmt.query_one((), |row| row.get(0));

        if let Err(rusqlite::Error::QueryReturnedNoRows) = version {
            Ok(0)
        } else {
            version.context("Failed to get version from database")
        }
    }
}