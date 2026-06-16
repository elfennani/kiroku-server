use crate::domain::models::ProcessingStep;
use crate::errors::AppError;
use crate::infrastructure::database::connection::Database;
use crate::infrastructure::episode_repo::EpisodeRepository;
use crate::infrastructure::packager::Packager;
use crate::prelude::*;
use log::{debug, error, info};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::select;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;

struct ProcessingJob {
    id: String,
    cancellation_token: CancellationToken,
}

pub struct PackagerService {
    episode_repository: Arc<EpisodeRepository>,
    app_data_dir: PathBuf,
    tx: UnboundedSender<String>,
    rx: Arc<Mutex<UnboundedReceiver<String>>>,
    current_job: Arc<Mutex<Option<ProcessingJob>>>,
}

impl PackagerService {
    pub async fn new(db: Arc<Database>, app_data_dir: impl AsRef<Path>) -> PackagerService {
        let output_dir = app_data_dir.as_ref();
        let episode_repository = Arc::new(EpisodeRepository::new(db.clone(), output_dir));
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // TODO: Retry incomplete queues
        let queue = episode_repository.get_queue_items().await;

        Self {
            app_data_dir: output_dir.to_owned(),
            tx,
            rx: Arc::new(Mutex::new(rx)),
            episode_repository,
            current_job: Arc::new(Mutex::new(None)),
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

    pub async fn cancel(&self, id: String) -> Result<()> {
        self.episode_repository
            .update_queue_status(&id, ProcessingStep::Cancelled, None)
            .await?;

        let mut current_job = self.current_job.lock().await;
        if let Some(job) = current_job.as_ref() {
            if job.id == id {
                job.cancellation_token.cancel();
                *current_job = None;
            }
        }

        Ok(())
    }

    pub fn app_data_dir(&self) -> &Path {
        &self.app_data_dir
    }

    pub async fn start(&self) -> Result<()> {
        let mut rx = self.rx.lock().await;

        while let Some(id) = rx.recv().await {
            let cancellation_token = CancellationToken::new();
            let job = ProcessingJob {
                id: id.clone(),
                cancellation_token: cancellation_token.clone(),
            };
            {
                let mut current_job = self.current_job.lock().await;
                *current_job = Some(job);
            }

            let episode_repo = self.episode_repository.clone();
            let id_clone = id.clone();
            let processing_job = tokio::spawn(async move {
                debug!("PackagerService received: {:?}", id);

                let process = match episode_repo.get_queue_item(&id).await? {
                    None => {
                        return Err(AppError::NotFound(format!(
                            "Episode queue item #{} not found",
                            id
                        )));
                    }
                    Some(ep) => ep,
                };

                if process.step == ProcessingStep::Cancelled {
                    return Ok(());
                }

                let mut packager = Packager::new(process.file_path, process.output_dir)?;
                let metadata = packager.get_metadata().await?;

                if process.step <= ProcessingStep::Processing720p {
                    episode_repo
                        .update_queue_status(&id, ProcessingStep::Processing720p, None)
                        .await?;

                    debug!("Transcoding Video in 720p");
                    let mut _last_update = SystemTime::now();
                    let video = packager
                        .transcode_video(720, |progress| {
                            let episode_repo = episode_repo.clone();
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

                    episode_repo
                        .insert_temp_file(&id, video.to_str().unwrap())
                        .await?;
                }

                if process.step <= ProcessingStep::Processing1080p {
                    debug!("Transcoding Video in 1080p");
                    let mut _last_update = SystemTime::now();
                    episode_repo
                        .update_queue_status(&id, ProcessingStep::Processing720p, None)
                        .await?;
                    let video = packager
                        .transcode_video(1080, |progress| {
                            let episode_repo = episode_repo.clone();
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
                    episode_repo
                        .insert_temp_file(&id, video.to_str().unwrap())
                        .await?;
                }

                if process.step <= ProcessingStep::ProcessingAudio {
                    episode_repo
                        .update_queue_status(&id, ProcessingStep::ProcessingAudio, None)
                        .await?;
                    for audio in metadata.audio {
                        debug!(
                            "Transcoding Audio Stream #{} ({})",
                            audio.index, audio.title
                        );
                        let audio = packager.transcode_audio(audio).await?;
                        episode_repo
                            .insert_temp_file(&id, audio.to_str().unwrap())
                            .await?;
                    }
                }

                if process.step <= ProcessingStep::ProcessingSubtitles {
                    episode_repo
                        .update_queue_status(&id, ProcessingStep::ProcessingSubtitles, None)
                        .await?;
                    for subtitle in metadata.subtitles {
                        debug!(
                            "Transcoding Subtitle Stream #{} ({})",
                            subtitle.index, subtitle.title
                        );
                        let subtitle = packager.extract_subtitles(subtitle).await?;
                        episode_repo
                            .insert_temp_file(&id, subtitle.to_str().unwrap())
                            .await?;
                    }
                }

                episode_repo
                    .update_queue_status(&id, ProcessingStep::Packaging, None)
                    .await?;
                debug!("Generating thumbnail");
                let thumbnail = packager.generate_thumbnail().await?;

                debug!("Packaging everything to a playlist");
                let metadata = packager.get_metadata().await?;
                let playlist = packager.package().await?;

                debug!("Saving...");
                episode_repo
                    .save_episode(&id, metadata, thumbnail, playlist)
                    .await?;
                episode_repo
                    .update_queue_status(&id, ProcessingStep::Done, None)
                    .await?;

                debug!("Deleting temporary files");
                let temp_files = episode_repo.get_temp_files_by_queue_id(&id).await?;

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

                episode_repo.clear_temp_files(&id).await?;

                info!("Packaging Done");

                Ok(())
            });

            let handle = processing_job.abort_handle();
            let data: Result<()> = select! {
                _ = cancellation_token.cancelled() => {
                    info!("Job \"{}\" cancelled", id_clone);
                    handle.abort();
                    Ok(())
                }

                result = processing_job => {
                    result.map_err(|err| AppError::TranscodeError("Failed to run processing thread".to_string()))?
                }
            };

            if let Err(e) = data {
                error!("Error while processing: {:?}", e);
            }
        }

        info!("PackagerService stopped");

        Ok(())
    }
}
