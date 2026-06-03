use crate::domain::models::{ProcessedFileType, ProcessingStatus};
use crate::domain::traits::MediaProcessorRepository;
use crate::errors::AppError;
use crate::infrastructure::database::Database;
use crate::infrastructure::media_processor::MediaProcessorRepositoryImpl;
use crate::infrastructure::packager::Packager;
use crate::prelude::*;
use log::{error, info};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use uuid::Uuid;

pub struct PackagerService {
    media_processor_repo: Arc<dyn MediaProcessorRepository>,
    output_dir: PathBuf,
    tx: UnboundedSender<Uuid>,
    rx: Arc<Mutex<UnboundedReceiver<Uuid>>>,
}

impl PackagerService {
    pub fn new(db: Arc<Database>, output_dir: impl AsRef<Path>) -> PackagerService {
        let output_dir = output_dir.as_ref();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let media_processor_repo = Arc::new(MediaProcessorRepositoryImpl::new(db));

        Self {
            output_dir: output_dir.to_owned(),
            tx,
            rx: Arc::new(Mutex::new(rx)),
            media_processor_repo,
        }
    }

    pub async fn enqueue(&self, path_buf: PathBuf, media_id: usize, episode: f32) -> Result<Uuid> {
        let dir_uuid = Uuid::new_v4();
        let output_dir = self.output_dir.join(dir_uuid.to_string());
        let packager = Packager::new(path_buf.clone(), self.output_dir.join(dir_uuid.to_string()))?;
        let metadata = packager.get_metadata().await?;
        let metadata_id = self.media_processor_repo.save_metadata(metadata)?;

        self.media_processor_repo
            .insert_media(media_id, metadata_id as usize, episode)?;
        let process =
            self.media_processor_repo
                .enqueue(metadata_id as usize, &output_dir, &path_buf)?;

        info!("Enqueued media in process: {}", process);
        self.tx.send(process).map_err(|err| {
            error!("Failed to enqueue packager: {}", err);
            AppError::PackagerError(err.to_string())
        })?;

        info!("Process sent to packager!");

        Ok(process)
    }

    pub async fn start(&self) -> Result<()> {
        let mut rx = self.rx.lock().await;
        while let Some(uuid) = rx.recv().await {
            let result: Result<()> = {
                info!("PackagerService received: {:?}", uuid);

                let process = self.media_processor_repo.get_processing_item(uuid)?;
                let mut packager =
                    Packager::new(process.input_file, self.output_dir.join(uuid.to_string()))?;
                let metadata = packager.get_metadata().await?;

                self.media_processor_repo
                    .set_processing_status(uuid, ProcessingStatus::Processing)?;

                info!("Transcoding Video in 720p");
                let video = packager.transcode_video(720).await?;
                self.media_processor_repo.insert_processed_files(
                    uuid,
                    video.file_name().unwrap().to_str().unwrap(),
                    &video,
                    ProcessedFileType::Video,
                )?;

                info!("Transcoding Video in 1080p");
                let video = packager.transcode_video(1080).await?;
                self.media_processor_repo.insert_processed_files(
                    uuid,
                    video.file_name().unwrap().to_str().unwrap(),
                    &video,
                    ProcessedFileType::Video,
                )?;

                for audio in metadata.audio {
                    info!(
                    "Transcoding Audio Stream #{} ({})",
                    audio.index, audio.title
                );
                    let audio = packager.transcode_audio(audio).await?;
                    self.media_processor_repo.insert_processed_files(
                        uuid,
                        audio.file_stem().unwrap().to_str().unwrap(),
                        &audio,
                        ProcessedFileType::Video,
                    )?;
                }

                for subtitle in metadata.subtitles {
                    info!(
                    "Transcoding Subtitle Stream #{} ({})",
                    subtitle.index, subtitle.title
                );
                    let subtitle = packager.extract_subtitles(subtitle).await?;
                    self.media_processor_repo.insert_processed_files(
                        uuid,
                        subtitle.file_stem().unwrap().to_str().unwrap(),
                        &subtitle,
                        ProcessedFileType::Video,
                    )?;
                }

                info!("Packaging everything to a playlist");
                let playlist = packager.package().await?;
                info!("Saving...");
                self.media_processor_repo
                    .set_processing_playlist(uuid, &playlist)?;
                self.media_processor_repo
                    .set_processing_status(uuid, ProcessingStatus::Done)?;

                info!("Deleting temporary files");
                let process = self.media_processor_repo.get_processing_item(uuid)?;
                info!("Deleting temporary files: {:?}", process.processed_files);
                for file in process.processed_files {
                    info!("Deleting temporary file: {}", file.to_string_lossy());
                    self.media_processor_repo
                        .delete_processed_file_by_path(&file)?;
                    info!("Deleted db entry for file: {}", file.to_string_lossy());
                    std::fs::remove_file(&file)
                        .map_err(|err| AppError::InternalServer(err.to_string()))?;
                    info!("Deleted file: {:?}", file);
                }

                info!("Packaging Done");

                Ok(())
            };

            if let Err(e) = result {
                error!("Error while processing: {:?}", e);
            }
        }

        info!("PackagerService stopped");

        Ok(())
    }
}
