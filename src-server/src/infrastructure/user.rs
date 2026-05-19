use crate::domain::models::User;
use crate::domain::traits::UserRepository;
use crate::infrastructure::database::Database;
use crate::prelude::*;
use rusqlite::params;
use std::sync::Arc;

pub struct UserRepositoryImpl {
    db: Arc<Database>,
}

impl UserRepositoryImpl {
    pub fn new(db: Arc<Database>) -> Self {
        UserRepositoryImpl { db }
    }
}

impl UserRepository for UserRepositoryImpl {
    fn get_user_by_id(&self, id: i32) -> Result<Option<User>> {
        let conn = self.db.connection.lock().unwrap();
        let query = conn.query_one(
            // language=sqlite
            "SELECT (id, name, avatar_url, banner_url, description) FROM users WHERE id=?",
            params![id],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    avatar_url: row.get(2)?,
                    banner_url: row.get(3)?,
                    description: row.get(4)?,
                })
            },
        );

        if let Err(rusqlite::Error::QueryReturnedNoRows) = query {
            return Ok(None);
        }

        Ok(Some(query?))
    }

    fn get_viewer_user(&self) -> Result<Option<User>> {
        let conn = self.db.connection.lock().unwrap();
        let query = conn.query_one(
            // language=sqlite
            "SELECT id, name, avatar_url, banner_url, description FROM users WHERE is_viewer=1",
            params![],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    avatar_url: row.get(2)?,
                    banner_url: row.get(3)?,
                    description: row.get(4)?,
                })
            },
        );

        if let Err(rusqlite::Error::QueryReturnedNoRows) = query {
            return Ok(None);
        }

        Ok(Some(query?))
    }

    fn save_user(&self, user: &User, mut is_viewer: bool) -> Result<()> {
        let viewer = self.get_viewer_user()?;
        let conn = self.db.connection.lock().unwrap();
        
        if is_viewer {
            if viewer.is_some() && viewer.unwrap().id != user.id {
                conn.execute(
                    // language=sqlite
                    "DELETE FROM users WHERE is_viewer=1",
                    params![],
                )?;
            }
        } else {
            if viewer.is_some() && viewer.unwrap().id == user.id {
                is_viewer = true
            }
        }

        let mut stmt =
            // language=sqlite
            conn.prepare("
                INSERT OR REPLACE INTO
                    users (id, name, avatar_url, banner_url, description, is_viewer)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6);
            ")?;

        stmt.execute(params![
            user.id,
            user.name,
            user.avatar_url,
            user.banner_url,
            user.description,
            if is_viewer { 1 } else { 0 }
        ])?;

        Ok(())
    }
}
