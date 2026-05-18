pub mod metadata;

use crate::errors::AppError;
use crate::infrastructure::packager::metadata::MediaMetadata;
use crate::prelude::*;
use std::fmt::format;
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use tokio::process::Command;

pub struct Packager {
    file: PathBuf,
}

impl Packager {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(AppError::NotFound(format!(
                "File not found: {}",
                path.display()
            )));
        }

        Ok(Self {
            file: <Path as AsRef<Path>>::as_ref(path).to_path_buf(),
        })
    }

    pub async fn get_metadata(&self) -> Result<MediaMetadata> {
        // ffprobe -v quiet -print_format json -show_format -show_streams -show_chapters input.mp4
        let output = Command::new("ffprobe")
            .args(&[
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
                "-show_chapters",
                self.file.to_str().unwrap(),
            ])
            .output()
            .await
            .map_err(|err| AppError::InternalServer(format!("ffprobe error: {}", err)))?;

        let data = String::from_utf8_lossy(&output.stdout);

        Ok(MediaMetadata::from_serde_value(serde_json::from_str(
            &data,
        )?)?)
    }

    pub async fn encode(&self, metadata: &MediaMetadata, dir: impl AsRef<Path>) -> Result<()> {
        let dir = dir.as_ref().to_path_buf();
        let resolutions = [720, 1080];

        // Encode Video Resolutions
        for resolution in resolutions {
            // WARNING: `h264_videotoolbox` is an encoder that uses Apple Silicon hardware encoder,
            //          meaning it would not function in other platforms, so change to `libx264` it
            //          if someone wants to use this project.

            // ffmpeg -i input.mkv -an -sn -vf "scale=-2:720" -c:v h264_videotoolbox video_720p.mp4

            println!("Encoding video in {}p", resolution);
            let output = Command::new("ffmpeg")
                .args([
                    "-i",
                    self.file.to_str().unwrap(),
                    "-an",
                    "-sn",
                    "-vf",
                    format!("scale=-2:{}", resolution).as_str(),
                    "-c:v",
                    "h264_videotoolbox",
                    dir.join(format!("video_{}p.mp4", resolution))
                        .to_str()
                        .unwrap(),
                ])
                .output()
                .await;

            if let Err(err) = output {
                eprintln!("ffmpeg error: {}", err);
                return Err(AppError::InternalServer("ffmpeg error".to_string()));
            }
        }

        // Encode audio streams
        for audio in metadata.audio.iter() {
            // Note: Shaka packager doesn't accept `aac` or `mp3`, but for some reason
            //       it take `mp4` as a container for audio.
            // ffmpeg -i input.mkv -map 0:a:0 audio_eng.mp4
            println!(
                "Encoding audio stream #{} ({})",
                audio.index,
                audio.language.clone().unwrap_or("NO_LANG".to_string())
            );
            let output = Command::new("ffmpeg")
                .args([
                    "-i",
                    self.file.to_str().unwrap(),
                    "-map", // -map automatically disables automatic stream selection so no need to exclude video and subs streams (with `-vn` and `-sn`)
                    format!("0:a:{}", audio.index).as_str(), // Audio stream selection.
                    dir.join(format!(
                        "audio_{}.mp4",
                        audio.language.clone().unwrap_or(audio.index.to_string())
                    ))
                    .to_str()
                    .unwrap(),
                ])
                .output()
                .await;

            if let Err(err) = output {
                eprintln!("ffmpeg error: {}", err);
                return Err(AppError::InternalServer(
                    "ffmpeg (audio encoding) error".to_string(),
                ));
            }
        }

        // Extract subtitle tracks
        for subtitle in metadata.subtitles.iter() {
            // ffmpeg -i input.mkv -map 0:s:0 subtitles.vtt
            println!(
                "Extracting subtitle stream #{} ({})",
                subtitle.index,
                subtitle.language.clone().unwrap_or("NO_LANG".to_string())
            );
            let output = Command::new("ffmpeg")
                .args([
                    "-i",
                    self.file.to_str().unwrap(),
                    "-map",
                    format!("0:s:{}", subtitle.index).as_str(),
                    dir.join(format!(
                        "subtitles_{}.vtt",
                        subtitle
                            .language
                            .clone()
                            .unwrap_or(subtitle.index.to_string())
                    ))
                    .to_str()
                    .unwrap(),
                ])
                .output()
                .await;

            if let Err(err) = output {
                eprintln!("ffmpeg error: {}", err);
                return Err(AppError::InternalServer(
                    "ffmpeg (subtitle extracting) error".to_string(),
                ));
            }
        }

        Ok(())
    }
    pub async fn package(&self, metadata: &MediaMetadata, dir: impl AsRef<Path>) -> Result<()> {
        let dir = dir.as_ref().to_path_buf();
        // IMPORTANT: Both ffmpeg and Shaka packager need to be installed. Shaka installs by
        //            default as `packager`, I have renamed it in my system to `shaka`
        //            solely to distinguish it

        //shaka \
        //   'in=audio_eng.mp4,stream=audio,segment_template=audio_eng/$Number$.aac,playlist_name=audio_eng/main.m3u8,hls_group_id=audio,hls_name=ENGLISH' \
        //   'in=audio_jap.mp4,stream=audio,segment_template=audio_jap/$Number$.aac,playlist_name=audio_jap/main.m3u8,hls_group_id=audio,hls_name=JAPANESE' \
        //   'in=subtitles.vtt,stream=text,segment_template=text/$Number$.vtt,playlist_name=text/main.m3u8,hls_group_id=text,hls_name=ENGLISH' \
        //   'in=video_480p.mp4,stream=video,segment_template=h264_480p/$Number$.ts,playlist_name=h264_480p/main.m3u8,iframe_playlist_name=h264_480p/iframe.m3u8' \
        //   'in=video_720p.mp4,stream=video,segment_template=h264_720p/$Number$.ts,playlist_name=h264_720p/main.m3u8,iframe_playlist_name=h264_720p/iframe.m3u8' \
        //   --hls_master_playlist_output h264_master.m3u8

        let mut streams: Vec<String> = vec![];
        let temp_files: Vec<PathBuf> = vec![];

        let resolutions = [720, 1080];

        for res in resolutions {
            streams.push(format!("in=video_{}p.mp4,stream=video,segment_template=h264_{}p/$Number$.ts,playlist_name=h264_{}p/main.m3u8,iframe_playlist_name=h264_{}p/iframe.m3u8", res, res, res, res))
        }

        for audio in metadata.audio.iter() {
            let suffix = audio.language.clone().unwrap_or(audio.index.to_string());
            streams.push(
                format!(
                    "in=audio_{}.mp4,stream=audio,segment_template=audio_{}/$Number$.aac,playlist_name=audio_{}/main.m3u8,hls_group_id=audio,hls_name={}", suffix, suffix, suffix, audio.language.clone().unwrap_or(format!("Track {}", audio.index + 1))
                )
            );
        }

        for subtitle in metadata.subtitles.iter() {
            let suffix = subtitle
                .language
                .clone()
                .unwrap_or(subtitle.index.to_string());
            streams.push(format!("in=subtitles_{}.vtt,stream=text,segment_template=text_{}/$Number$.vtt,playlist_name=text_{}/main.m3u8,hls_group_id=text,hls_name={}", suffix, suffix, suffix, subtitle.language.clone().unwrap_or(format!("Track {}", subtitle.index + 1))))
        }

        let mut command = Command::new("shaka");
        for stream in streams {
            command.arg(stream);
        }

        command
            .arg("--hls_master_playlist_output")
            .arg("h264_master.m3u8");

        let output = command.current_dir(dir).output().await.unwrap();

        println!("{}", String::from_utf8_lossy(&output.stdout));
        println!("{}", String::from_utf8_lossy(&output.stderr));

        Ok(())
    }
}
