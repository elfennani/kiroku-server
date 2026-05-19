use crate::domain::models::{
    MediaStatus, ProcessedFileType, ProcessingQueueItem, ProcessingStatus,
};
use crate::domain::traits::MediaProcessorRepository;
use crate::infrastructure::database::Database;
use crate::infrastructure::packager::metadata::{
    AudioStream, Chapter, MediaMetadata, SubtitleStream,
};
use crate::prelude::*;
use log::error;
use rusqlite::params;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

pub struct MediaProcessorRepositoryImpl {
    db: Arc<Database>,
}

impl MediaProcessorRepositoryImpl {
    pub fn new(db: Arc<Database>) -> MediaProcessorRepositoryImpl {
        Self { db }
    }

    fn save_chapters(&self, metadata_id: usize, chapters: Vec<Chapter>) -> Result<()> {
        let conn = self.db.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            // language=sqlite
            "
                INSERT INTO
                    chapters (metadata_id, title, start, end, `index`)
                VALUES (?, ?, ?, ?, ?)
            ",
        )?;

        for chapter in chapters {
            stmt.execute(params![
                metadata_id as isize,
                chapter.title,
                chapter.start as i64,
                chapter.end as i64,
                chapter.index as isize,
            ])?;
        }

        Ok(())
    }

    fn save_audio_streams(&self, metadata_id: usize, streams: Vec<AudioStream>) -> Result<()> {
        let conn = self.db.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            // language=sqlite
            "
                INSERT INTO
                    streams (metadata_id, title, language, `index`, `type`)
                VALUES (?, ?, ?, ?, ?)
            ",
        )?;

        for stream in streams {
            stmt.execute(params![
                metadata_id as isize,
                stream.title,
                stream.language,
                stream.index as isize,
                "audio"
            ])?;
        }

        Ok(())
    }

    fn save_subtitle_streams(
        &self,
        metadata_id: usize,
        streams: Vec<SubtitleStream>,
    ) -> Result<()> {
        let conn = self.db.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            // language=sqlite
            "
                INSERT INTO
                    streams (metadata_id, title, language, `index`, `type`)
                VALUES (?, ?, ?, ?, ?)
            ",
        )?;

        for stream in streams {
            stmt.execute(params![
                metadata_id as isize,
                stream.title,
                stream.language,
                stream.index as isize,
                "subtitle"
            ])?;
        }

        Ok(())
    }
}

impl MediaProcessorRepository for MediaProcessorRepositoryImpl {
    fn save_metadata(&self, media_metadata: MediaMetadata) -> Result<i64> {
        let id = {
            let conn = self.db.connection.lock().unwrap();
            let mut stmt = conn.prepare(
                // language=sqlite
                "
                INSERT INTO
                    metadata (title, duration)
                VALUES (?, ?);
            ",
            )?;

            stmt.execute(params![
                media_metadata.title,
                media_metadata.duration as i64
            ])?;

            conn.last_insert_rowid()
            // `conn` needs to be dropped or else the next functions will hang waiting indefinitely
            // for the mutex to unlock.
        };

        self.save_chapters(id as usize, media_metadata.chapters)?;
        self.save_audio_streams(id as usize, media_metadata.audio)?;
        self.save_subtitle_streams(id as usize, media_metadata.subtitles)?;

        Ok(id)
    }

    fn enqueue(
        &self,
        metadata_id: usize,
        output_dir: &PathBuf,
        input_file: &PathBuf,
    ) -> Result<Uuid> {
        let uuid = Uuid::new_v4();
        let conn = self.db.connection.lock().unwrap();

        conn.execute(
            // language=sqlite
            "
                INSERT INTO
                    processing_queue (uuid, metadata_id, status, path, input_path)
                VALUES (?, ?, ?, ?, ?);
            ",
            params![
                uuid.to_string(),
                metadata_id as isize,
                "queued",
                output_dir.to_str().unwrap(),
                input_file.to_str().unwrap()
            ],
        )?;

        Ok(uuid)
    }

    fn set_processing_status(
        &self,
        processing_uuid: Uuid,
        processing_status: ProcessingStatus,
    ) -> Result<()> {
        let status = match processing_status {
            ProcessingStatus::Queued => "queued",
            ProcessingStatus::Processing => "processing",
            ProcessingStatus::Done => "done",
        };
        let conn = self.db.connection.lock().unwrap();
        conn.execute(
            // language=sqlite
            "
                UPDATE processing_queue
                    SET status = ?
                WHERE uuid = ?
            ",
            params![status, processing_uuid.to_string()],
        )?;

        Ok(())
    }

    fn set_processing_playlist(
        &self,
        processing_uuid: Uuid,
        playlist_path: &PathBuf,
    ) -> Result<()> {
        let conn = self.db.connection.lock().unwrap();
        conn.execute(
            // language=sqlite
            "
                UPDATE processing_queue
                    SET playlist_path = ?
                WHERE uuid = ?
            ",
            params![playlist_path.to_str().unwrap(), processing_uuid.to_string()],
        )?;

        Ok(())
    }

    fn insert_processed_files(
        &self,
        processing_uuid: Uuid,
        filename: &str,
        path: &PathBuf,
        file_type: ProcessedFileType,
    ) -> Result<i64> {
        let file_type = match file_type {
            ProcessedFileType::Audio => "audio",
            ProcessedFileType::Video => "video",
            ProcessedFileType::Subtitle => "subtitle",
        };

        let conn = self.db.connection.lock().unwrap();
        conn.execute(
            // language=sqlite
            "
                INSERT INTO
                    processed_files (processing_id, filename, path, type)
                VALUES (?, ?, ?, ?);
            ",
            params![
                processing_uuid.to_string(),
                filename,
                path.to_str().unwrap(),
                file_type
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    fn delete_processed_file(&self, id: i64) -> Result<()> {
        let conn = self.db.connection.lock().unwrap();

        conn.execute(
            // language=sqlite
            "DELETE FROM processed_files WHERE id=?",
            params![id],
        )?;

        Ok(())
    }

    fn delete_processed_file_by_path(&self, path: &PathBuf) -> Result<()> {
        let conn = self.db.connection.lock().unwrap();

        conn.execute(
            // language=sqlite
            "DELETE FROM processed_files WHERE path=?",
            params![path.to_str().unwrap()],
        )?;

        Ok(())
    }

    fn insert_media(&self, media_id: usize, metadata_id: usize, episode_number: f32) -> Result<()> {
        let conn = self.db.connection.lock().unwrap();
        conn.execute(
            // language=sqlite
            "
                INSERT INTO
                    media_metadata (metadata_id, media_id, episode)
                VALUES (?, ?, ?);
            ",
            params![metadata_id as isize, media_id as isize, episode_number],
        )?;

        Ok(())
    }

    fn get_metadata(&self, id: i64) -> Result<MediaMetadata> {
        let conn = self.db.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            // language=sqlite
            "SELECT title, start, end, `index` FROM chapters WHERE metadata_id=?",
        )?;
        let chapters: Vec<Chapter> = stmt
            .query_map(params![id], |row| {
                Ok(Chapter {
                    title: row.get(0)?,
                    start: row.get::<usize, i64>(1)? as u64,
                    end: row.get::<usize, i64>(2)? as u64,
                    index: row.get::<usize, i64>(3)? as usize,
                })
            })?
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect();

        let mut stmt = conn.prepare(
            // language=sqlite
            "SELECT title, language, `index` FROM streams WHERE type=? AND metadata_id=?",
        )?;

        let audio_streams: Vec<AudioStream> = stmt
            .query_map(params!["audio", id], |row| {
                Ok(AudioStream {
                    title: row.get(0)?,
                    language: row.get(1)?,
                    index: row.get::<usize, i64>(2)? as usize,
                })
            })?
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect();

        let subtitle_streams: Vec<SubtitleStream> = stmt
            .query_map(params!["subtitle", id], |row| {
                Ok(SubtitleStream {
                    title: row.get(0)?,
                    language: row.get(1)?,
                    index: row.get::<usize, i64>(2)? as usize,
                })
            })?
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect();

        let metadata = conn.query_one(
            "SELECT title, duration FROM metadata WHERE id=?",
            params![id],
            |row| {
                Ok(MediaMetadata {
                    title: row.get(0)?,
                    duration: row.get::<usize, i64>(1)? as u64,
                    chapters,
                    audio: audio_streams,
                    subtitles: subtitle_streams,
                })
            },
        )?;

        Ok(metadata)
    }

    fn get_processing_item(&self, id: Uuid) -> Result<ProcessingQueueItem> {
        let metadata_id = {
            let conn = self.db.connection.lock().unwrap();
            let metadata_id: i64 = conn.query_one(
                // language=sqlite
                "SELECT metadata_id FROM processing_queue WHERE uuid=?",
                params![id.to_string()],
                |row| Ok(row.get(0)?),
            )?;

            metadata_id
        };
        let metadata = self.get_metadata(metadata_id)?;
        let conn = self.db.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            // language=sqlite
            "SELECT path FROM processed_files WHERE processing_id=?",
        )?;
        let processed_files: Vec<PathBuf> = stmt
            .query_map(params![id.to_string()], |row| {
                let path: String = row.get(0)?;

                Ok(PathBuf::from_str(&path))
            })?
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect();

        let processing_item = conn.query_one(
            // language=sqlite
            "SELECT status, path, playlist_path, input_path FROM processing_queue WHERE uuid=?",
            params![id.to_string()],
            |row| {
                let status: String = row.get(0)?;
                let path: String = row.get(1)?;
                let playlist_path: Option<String> = row.get(2)?;
                let input_path: String = row.get(3)?;

                Ok(ProcessingQueueItem {
                    id,
                    status: match status.as_str() {
                        "queued" => ProcessingStatus::Queued,
                        "processing" => ProcessingStatus::Processing,
                        "done" => ProcessingStatus::Done,
                        _ => {
                            error!("Unknown status: {}", status);
                            ProcessingStatus::Queued
                        }
                    },
                    path: path.into(),
                    input_file: input_path.into(),
                    playlist_path: playlist_path.map(|x| x.into()),
                    metadata,
                    processed_files,
                })
            },
        );

        Ok(processing_item?)
    }
}
