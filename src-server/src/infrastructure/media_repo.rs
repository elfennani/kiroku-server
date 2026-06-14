use crate::domain::models::MediaSummary;
use crate::infrastructure::database::connection::Database;
use crate::prelude::*;
use sqlx::{Acquire, Row};
use std::sync::Arc;

pub struct MediaRepository {
    db: Arc<Database>,
}

impl MediaRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn cache_media(&self, media: &Vec<MediaSummary>) -> Result<()> {
        let mut conn = self.db.conn.lock().await;
        let mut tx = conn.begin().await?;

        for media in media {
            sqlx::query!(
                "INSERT OR REPLACE INTO cached_media VALUES (?,?,?,?,?,?,?)",
                media.id,
                media.title,
                media.banner,
                media.cover,
                media.progress,
                media.total,
                media
                    .status
                    .clone()
                    .map(|s| serde_plain::to_string(&s).unwrap()),
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn get_cached_media_by_id(&self, id: i32) -> Result<Option<MediaSummary>> {
        let mut conn = self.db.conn.lock().await;

        let result = sqlx::query("SELECT * FROM cached_media WHERE id = ?")
            .bind(id)
            .fetch_optional(&mut *conn)
            .await?
            .map(|row| MediaSummary {
                id,
                banner: row.get("banner"),
                cover: row.get("cover"),
                title: row.get("title"),
                progress: row.get("progress"),
                total: row.get("total"),
                status: row
                    .get::<Option<String>, _>("status")
                    .map(|status| serde_plain::from_str(&status).unwrap()),
            });

        Ok(result)
    }
}
