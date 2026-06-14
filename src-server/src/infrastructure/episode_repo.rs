use crate::domain::models::{EnqueueData, EpisodeQueueItem, ProcessingStep};
use crate::errors::AppError;
use crate::infrastructure::database::connection::Database;
use crate::infrastructure::packager::metadata::MediaMetadata;
use crate::prelude::*;
use log::error;
use nanoid::nanoid;
use sqlx::sqlite::SqliteRow;
use sqlx::{Connection, Row};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct EpisodeRepository {
    db: Arc<Database>,
    output_dir: PathBuf,
}

impl EpisodeRepository {
    pub fn new(db: Arc<Database>, output_dir: impl AsRef<Path>) -> EpisodeRepository {
        Self {
            db,
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }

    pub async fn enqueue(&self, media_id: i64, episodes: Vec<EnqueueData>) -> Result<Vec<String>> {
        let mut conn = self.db.conn.lock().await;
        let mut ids: Vec<String> = vec![];

        for episode in episodes {
            let id = nanoid!(10);
            let dir = self.output_dir.join(&id);
            std::fs::create_dir_all(&dir).map_err(|err| {
                error!(
                    "Failed to create episode directory {}: {}",
                    dir.display(),
                    err
                );

                AppError::InternalServer(err.to_string())
            })?;

            sqlx::query!(
                "
                INSERT INTO episode_queue
                    (id, media_id, episode_number, file_path, output_path, step)
                VALUES
                    (?, ?, ?, ?, ?, ?)
            ",
                id,
                media_id,
                episode.episode_number(),
                episode.path().to_str(),
                dir.to_str(),
                serde_plain::to_string(&ProcessingStep::InQueue).unwrap()
            )
            .execute(&mut *conn)
            .await?;

            ids.push(id);
        }

        Ok(ids)
    }

    pub async fn get_queue_item(&self, id: impl AsRef<str>) -> Result<Option<EpisodeQueueItem>> {
        let mut conn = self.db.conn.lock().await;

        Ok(sqlx::query!(
            "SELECT * FROM episode_queue WHERE id=?",
            id.as_ref().to_string()
        )
        .map(|row| EpisodeQueueItem {
            id: row.id.unwrap(),
            media_id: row.media_id,
            episode_number: row.episode_number,
            file_path: PathBuf::from(row.file_path.unwrap()),
            output_dir: PathBuf::from(row.output_path),
            step: serde_plain::from_str(&row.step).unwrap(),
            progress: row.progress,
        })
        .fetch_optional(&mut *conn)
        .await?)
    }

    pub async fn update_queue_status(
        &self,
        id: impl AsRef<str>,
        step: ProcessingStep,
        progress: Option<f64>,
    ) -> Result<()> {
        let mut conn = self.db.conn.lock().await;

        sqlx::query!(
            "UPDATE episode_queue SET step = ?, progress = ?, updated_at = CURRENT_TIMESTAMP WHERE id=?",
            serde_plain::to_string(&step).unwrap(),
            progress,
            id.as_ref()
        )
            .execute(&mut *conn)
            .await?;

        Ok(())
    }

    pub async fn insert_temp_file(
        &self,
        id: impl AsRef<str>,
        file_path: impl AsRef<Path>,
    ) -> Result<()> {
        let mut conn = self.db.conn.lock().await;

        sqlx::query!(
            "INSERT INTO episode_queue_temp_files (episode_queue_id, file_path) VALUES (?, ?)",
            id.as_ref(),
            file_path.as_ref().to_str().unwrap()
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    pub async fn get_temp_files_by_queue_id(&self, id: impl AsRef<str>) -> Result<Vec<PathBuf>> {
        let mut conn = self.db.conn.lock().await;

        let paths = sqlx::query(
            "SELECT DISTINCT file_path FROM episode_queue_temp_files WHERE episode_queue_id=?",
        )
        .bind(id.as_ref())
        .map(|row: SqliteRow| row.get::<String, _>("file_path"))
        .fetch_all(&mut *conn)
        .await?
        .iter()
        .map(|path_str| PathBuf::from(path_str))
        .collect();

        Ok(paths)
    }

    pub async fn clear_temp_files(&self, queue_id: impl AsRef<str>) -> Result<()> {
        let mut conn = self.db.conn.lock().await;

        sqlx::query!(
            "DELETE FROM episode_queue_temp_files WHERE episode_queue_id=?",
            queue_id.as_ref()
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    pub async fn save_episode(
        &self,
        queue_id: impl AsRef<str>,
        metadata: MediaMetadata,
        thumbnail: PathBuf,
        playlist: PathBuf,
    ) -> Result<()> {
        let queue_item = match self.get_queue_item(queue_id.as_ref()).await? {
            Some(item) => item,
            None => return Err(AppError::NotFound(queue_id.as_ref().to_string())),
        };

        let mut conn = self.db.conn.lock().await;
        let mut tx = conn.begin().await?;

        sqlx::query!(
            "INSERT INTO episode (id, media_id, title, duration, number, thumbnail, url) VALUES (?,?,?,?,?,?,?)",
            queue_id.as_ref(),
            queue_item.media_id,
            None::<String>,
            metadata.duration as i64,
            queue_item.episode_number,
            thumbnail.to_str(),
            playlist.to_str(),
        ).execute(&mut *tx).await?;

        for chapter in metadata.chapters {
            sqlx::query!(
                "INSERT INTO chapters (episode_id, start_time, name) VALUES (?,?,?)",
                queue_id.as_ref(),
                chapter.start as i64,
                chapter.title
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}
