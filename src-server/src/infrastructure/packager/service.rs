use crate::domain::models::{EpisodeQueueItem, ProcessingStatus, ProcessingStep};
use crate::errors::AppError;
use crate::infrastructure::database::connection::Database;
use crate::infrastructure::episode_repo::EpisodeRepository;
use crate::infrastructure::packager::Packager;
use crate::prelude::*;
use log::{debug, error, info};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct PackagerService {
    episode_repository: Arc<EpisodeRepository>,
    output_dir: PathBuf,
    tx: UnboundedSender<String>,
    rx: Arc<Mutex<UnboundedReceiver<String>>>,
}

impl PackagerService {
    pub fn new(db: Arc<Database>, output_dir: impl AsRef<Path>) -> PackagerService {
        let output_dir = output_dir.as_ref();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        Self {
            output_dir: output_dir.to_owned(),
            tx,
            rx: Arc::new(Mutex::new(rx)),
            episode_repository: Arc::new(EpisodeRepository::new(db.clone(), output_dir)),
        }
    }

    pub async fn enqueue(&self, episode_id_list: &[String]) -> Result<()> {
        info!("Enqueued media in process: {}", episode_id_list.join(","));

        for episode_id in episode_id_list {
            self.tx.send(String::from(episode_id)).map_err(|err| {
                error!("Failed to enqueue packager: {}", err);
                AppError::PackagerError(err.to_string())
            })?;
        }

        info!("Process sent to packager!");

        Ok(())
    }

    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    pub async fn start(&self) -> Result<()> {
        let mut rx = self.rx.lock().await;

        while let Some(id) = rx.recv().await {
            let result: Result<()> = {
                debug!("PackagerService received: {:?}", id);

                let process = match self.episode_repository.get_queue_item(&id).await? {
                    None => {
                        return Err(AppError::NotFound(format!(
                            "Episode queue item #{} not found",
                            id
                        )));
                    }
                    Some(ep) => ep,
                };

                let mut packager = Packager::new(process.file_path, process.output_dir)?;
                let metadata = packager.get_metadata().await?;

                if process.step <= ProcessingStep::Processing720p {
                    self.episode_repository
                        .update_queue_status(&id, ProcessingStep::Processing720p, None)
                        .await?;

                    debug!("Transcoding Video in 720p");
                    let mut _last_update = SystemTime::now();
                    let video = packager
                        .transcode_video(720, |progress| {
                            let episode_repo = self.episode_repository.clone();
                            let queue_id = id.clone();

                            tokio::spawn(async move {
                                let curr = SystemTime::now();

                                // Debounce by updating every second.
                                if curr.duration_since(_last_update).unwrap().as_millis() < 1000 {
                                    return;
                                }

                                _last_update = curr;

                                episode_repo
                                    .update_queue_status(
                                        queue_id,
                                        ProcessingStep::Processing720p,
                                        Some(progress),
                                    )
                                    .await
                                    .ok();
                            });
                        })
                        .await?;

                    self.episode_repository
                        .insert_temp_file(&id, video.to_str().unwrap())
                        .await?;
                }

                if process.step <= ProcessingStep::Processing1080p {
                    debug!("Transcoding Video in 1080p");
                    let mut _last_update = SystemTime::now();
                    self.episode_repository
                        .update_queue_status(&id, ProcessingStep::Processing720p, None)
                        .await?;
                    let video = packager
                        .transcode_video(1080, |progress| {
                            let episode_repo = self.episode_repository.clone();
                            let queue_id = id.clone();

                            tokio::spawn(async move {
                                let curr = SystemTime::now();

                                // Debounce by updating every second.
                                if curr.duration_since(_last_update).unwrap().as_millis() < 1000 {
                                    return;
                                }

                                _last_update = curr;

                                episode_repo
                                    .update_queue_status(
                                        queue_id,
                                        ProcessingStep::Processing1080p,
                                        Some(progress),
                                    )
                                    .await
                                    .ok();
                            });
                        })
                        .await?;
                    self.episode_repository
                        .insert_temp_file(&id, video.to_str().unwrap())
                        .await?;
                }

                if process.step <= ProcessingStep::ProcessingAudio {
                    self.episode_repository
                        .update_queue_status(&id, ProcessingStep::ProcessingAudio, None)
                        .await?;
                    for audio in metadata.audio {
                        debug!(
                            "Transcoding Audio Stream #{} ({})",
                            audio.index, audio.title
                        );
                        let audio = packager.transcode_audio(audio).await?;
                        self.episode_repository
                            .insert_temp_file(&id, audio.to_str().unwrap())
                            .await?;
                    }
                }

                if process.step <= ProcessingStep::ProcessingSubtitles {
                    self.episode_repository
                        .update_queue_status(&id, ProcessingStep::ProcessingSubtitles, None)
                        .await?;
                    for subtitle in metadata.subtitles {
                        debug!(
                            "Transcoding Subtitle Stream #{} ({})",
                            subtitle.index, subtitle.title
                        );
                        let subtitle = packager.extract_subtitles(subtitle).await?;
                        self.episode_repository
                            .insert_temp_file(&id, subtitle.to_str().unwrap())
                            .await?;
                    }
                }

                self.episode_repository
                    .update_queue_status(&id, ProcessingStep::Packaging, None)
                    .await?;
                debug!("Generating thumbnail");
                let thumbnail = packager.generate_thumbnail().await?;

                debug!("Packaging everything to a playlist");
                let metadata = packager.get_metadata().await?;
                let playlist = packager.package().await?;

                debug!("Saving...");
                self.episode_repository
                    .save_episode(&id, metadata, thumbnail, playlist)
                    .await?;
                self.episode_repository
                    .update_queue_status(&id, ProcessingStep::Done, None)
                    .await?;

                debug!("Deleting temporary files");
                let temp_files = self
                    .episode_repository
                    .get_temp_files_by_queue_id(&id)
                    .await?;

                debug!("Deleting {} temporary files", temp_files.len());
                for file in temp_files {
                    debug!("Deleting temporary file: {}", file.to_string_lossy());
                    std::fs::remove_file(&file)
                        .map_err(|err| {
                            error!(
                                "Failed to delete temporary file {}: {}",
                                file.to_string_lossy(),
                                err
                            );
                            AppError::InternalServer(err.to_string())
                        })
                        .ok();
                    debug!("Deleted file: {:?}", file);
                }

                self.episode_repository.clear_temp_files(&id).await?;

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
