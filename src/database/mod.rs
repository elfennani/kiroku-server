pub mod migration;
pub mod table;

use crate::database::migration::Migration;
use crate::database::table::Session;
use anyhow::Context;
use rusqlite::{params, Connection, OpenFlags, Result};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct Database {
    connection: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open(db_file: PathBuf, migrations: Vec<Migration>) -> anyhow::Result<Database> {
        let conn = Connection::open_with_flags(
            db_file,
            OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
        )
        .context("Connection to database failed!")?;

        conn.execute(
            // language=SQL
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

            migration.execute(&conn)?;
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

    pub fn save_session(&self, access_token: &str) -> anyhow::Result<()> {
        self.connection
            .lock()
            .unwrap()
            .execute(
                // language=sqlite
                "INSERT OR REPLACE INTO sessions (id, token) VALUES (1, ?)",
                params![access_token],
            )
            .context("Failed to insert session row")?;

        Ok(())
    }

    pub fn get_session(&self) -> anyhow::Result<Option<Session>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            // language=sqlite
            "SELECT id,token FROM sessions",
        )?;
        let result = stmt.query_one((), |row| {
            Ok(Session {
                id: row.get(0)?,
                access_token: row.get(1)?,
            })
        });

        if let Err(rusqlite::Error::QueryReturnedNoRows) = result {
            Ok(None)
        } else {
            result
                .context("Failed to get session from database")
                .map(Some)
        }
    }
}
