use crate::domain::models::{
    ProcessedFileType, ProcessedEpisode, ProcessingQueueItem, ProcessingStatus, User,
};
use crate::infrastructure::packager::metadata::{
    AudioStream, Chapter, MediaMetadata, SubtitleStream,
};
use crate::prelude::*;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub trait SessionRepository: Sync + Send {
    fn get_access_token(&self) -> Result<Option<String>>;
    fn save_access_token(&self, access_token: String) -> Result<()>;
}

pub trait UserRepository: Sync + Send {
    fn get_user_by_id(&self, id: i32) -> Result<Option<User>>;
    fn get_viewer_user(&self) -> Result<Option<User>>;
    fn save_user(&self, user: &User, is_viewer: bool) -> Result<()>;
}

pub trait MediaProcessorRepository: Sync + Send {
    // Inserts media metadata to the database and returns the ID of the row.
    fn save_metadata(&self, media_metadata: MediaMetadata) -> Result<i64>;
    fn enqueue(
        &self,
        metadata_id: usize,
        output_dir: &PathBuf,
        input_file: &PathBuf,
    ) -> Result<Uuid>;
    fn set_processing_status(
        &self,
        processing_uuid: Uuid,
        processing_status: ProcessingStatus,
    ) -> Result<()>;
    fn set_processing_playlist(&self, processing_uuid: Uuid, playlist_path: &PathBuf)
    -> Result<()>;

    fn insert_processed_files(
        &self,
        processing_uuid: Uuid,
        filename: &str,
        path: &PathBuf,
        file_type: ProcessedFileType,
    ) -> Result<i64>;

    fn delete_processed_file(&self, id: i64) -> Result<()>;
    fn delete_processed_file_by_path(&self, path: &PathBuf) -> Result<()>;

    fn insert_media(&self, media_id: usize, metadata_id: usize, episode_number: f32) -> Result<()>;

    fn get_metadata(&self, id: i64) -> Result<MediaMetadata>;

    fn get_processing_item(&self, id: Uuid) -> Result<ProcessingQueueItem>;

    fn get_processed_media_by_media_id(&self, media_id: usize) -> Result<Vec<ProcessedEpisode>>;
}
