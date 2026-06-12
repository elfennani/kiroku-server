use crate::domain::models::Session;
use crate::infrastructure::database::connection::Database;
use crate::prelude::*;
use std::sync::Arc;

pub struct SessionRepository {
    db: Arc<Database>,
}

impl SessionRepository {
    pub fn new(db: Arc<Database>) -> Self {
        SessionRepository { db }
    }
}

impl SessionRepository {
    pub async fn get_access_token(&self) -> Result<Option<Session>> {
        let mut conn = self.db.conn.lock().await;

        let result: Option<Session> =
            sqlx::query_as!(Session, "SELECT user_id, token as access_token FROM sessions")
                .fetch_optional(&mut *conn)
                .await?;

        Ok(result)
    }

    pub async fn save_access_token(&self, access_token: String, user_id: u32) -> Result<()> {
        let mut conn = self.db.conn.lock().await;
        sqlx::query!(
            "INSERT INTO sessions (id, token, user_id) VALUES (1, ?, ?)",
            access_token,
            user_id
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }
}
