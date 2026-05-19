use crate::domain::traits::SessionRepository;
use crate::errors::AppError;
use crate::infrastructure::database::Database;
use crate::infrastructure::database::table::Session;
use crate::prelude::*;
use rusqlite::params;
use std::sync::Arc;

pub struct SessionRepositoryImpl {
    db: Arc<Database>,
}

impl SessionRepositoryImpl {
    pub fn new(db: Arc<Database>) -> Self {
        SessionRepositoryImpl { db }
    }
}

impl SessionRepository for SessionRepositoryImpl {
    fn get_access_token(&self) -> Result<Option<String>> {
        let conn = self.db.connection.lock().unwrap();
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
                .map_err(AppError::from)
                .map(|session| Some(session.access_token))
        }
    }

    fn save_access_token(&self, access_token: String) -> Result<()> {
        self.db
            .connection
            .lock()
            .unwrap()
            .execute(
                // language=sqlite
                "INSERT OR REPLACE INTO sessions (id, token) VALUES (1, ?)",
                params![access_token],
            )
            .map_err(|e| AppError::InternalServer(e.to_string()))?;

        Ok(())
    }
}
