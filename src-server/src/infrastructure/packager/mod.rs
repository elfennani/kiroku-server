pub mod metadata;
pub mod service;

use crate::errors::AppError;
use crate::infrastructure::packager::metadata::{AudioStream, MediaMetadata, SubtitleStream};
use crate::prelude::*;
use log::info;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::OnceLock;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// This module takes an input mkv video, then separates all streams using `ffmpeg`, which
/// then are fed into Google's Shaka packager for the purpose of generating HLS playlist (Similar
/// to how other streaming platforms do like YouTube, Netflix, etc...
///
/// Why? Because I needed a way to share media between three devices, one of them is weak tablet
/// and cannot handle 1080p playback, and 720p doesn't play smoothly through SMB.
///
/// So I decided to make my MacBook a central server since it technically never turns off
/// even when the lid is closed. And this server simply streams HLS videos through a web interface
/// yet to be added. I could've avoided all the hassle and streamed MKV instead, but with web
/// browsers not supporting MKV that means I'd have to connect VLC every single time I want
/// to watch something. Plus this server has a bonus of updating AniList tracker automatically :)
pub struct Packager {
    file: PathBuf,
    output_dir: PathBuf,
    streams: HashSet<String>,
    metadata: OnceLock<MediaMetadata>,
}

impl Packager {
    pub fn new(input: impl AsRef<Path>, output_dir: impl AsRef<Path>) -> Result<Self> {
        let input = input.as_ref();
        let output_dir = output_dir.as_ref();

        if !input.exists() {
            return Err(AppError::NotFound(format!(
                "File not found: {}",
                input.display()
            )));
        }

        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)
                .map_err(|err| AppError::InternalServer(err.to_string()))?;
        }

        if !output_dir.is_dir() {
            return Err(AppError::BadRequest(String::from(
                "Output path is not a directory",
            )));
        }

        Ok(Self {
            file: input.to_owned(),
            output_dir: output_dir.to_owned(),
            streams: HashSet::new(),
            metadata: OnceLock::new(),
        })
    }

    pub async fn get_metadata(&self) -> Result<MediaMetadata> {
        if let Some(metadata) = self.metadata.get() {
            return Ok(metadata.clone());
        }

        // ffprobe -v quiet -print_format json -show_format -show_streams -show_chapters input.mp4
        let output = Command::new("ffprobe")
            .args([
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
        let metadata = MediaMetadata::from_serde_value(serde_json::from_str(&data)?)?;

        self.metadata.set(metadata.clone()).ok();

        Ok(metadata)
    }

    pub async fn transcode_video(&mut self, resolution: usize) -> Result<PathBuf> {
        let output_file = self.output_dir.join(format!("video_{}p.mp4", resolution));
        let metadata = self.get_metadata().await?;

        // WARNING: `h264_videotoolbox` is an encoder that uses Apple Silicon hardware encoder/decoder,
        //          meaning it would not function in other platforms, so change to `libx264` it
        //          if someone wants to use this project.

        // ffmpeg -i input.mkv -an -sn -vf "scale=-2:720" -c:v h264_videotoolbox video_720p.mp4
        let handle = Command::new("ffmpeg")
            .args([
                "-hide_banner",
                "-progress",
                "pipe:1", // Print progress in stdout (FD is 1).
                "-nostats",
                "-y", // to overwrite if exists
                "-i",
                self.file.to_str().unwrap(),
                "-an",
                "-sn",
                "-vf",
                format!("scale=-2:{}", resolution).as_str(),
                "-c:v",
                "h264_videotoolbox",
                output_file.to_str().unwrap(),
            ])
            .stdout(Stdio::piped())
            .spawn();

        if let Err(err) = handle {
            eprintln!("Failed to transcode: {}", err);
            return Err(AppError::TranscodeError(err.to_string()));
        }

        let mut handle = handle.unwrap();

        // https://rust-lang-nursery.github.io/rust-cookbook/os/external.html#continuously-process-child-process-outputs
        let stdout = handle.stdout.take().unwrap();
        let reader = BufReader::new(stdout);

        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if line.starts_with("out_time_ms") {
                let progress_ms = line.split("=").collect::<Vec<&str>>()[1]
                    .parse::<u64>()
                    .unwrap();
                let percentage = progress_ms as f64 / metadata.duration as f64;
                info!(
                    "{}p transcode progress: {:.2}%",
                    resolution,
                    percentage * 100.0
                );
            }
        }

        // We need to wait for the command to finish or else the function returns prematurely.
        match handle.wait_with_output().await {
            Ok(output) => {
                if !output.status.success() {
                    return Err(AppError::TranscodeError(
                        "ffmpeg exited with an error".to_string(),
                    ));
                }
            }
            Err(err) => {
                return Err(AppError::TranscodeError(err.to_string()));
            }
        }

        self.streams.insert(format!("in=video_{res}p.mp4,stream=video,segment_template=h264_{res}p/$Number$.ts,playlist_name=h264_{res}p/main.m3u8,iframe_playlist_name=h264_{res}p/iframe.m3u8", res = resolution));

        Ok(output_file)
    }
    pub async fn transcode_audio(&mut self, audio: AudioStream) -> Result<PathBuf> {
        let output_file = self.output_dir.join(format!(
            "audio_{}.mp4",
            audio.language.clone().unwrap_or(audio.index.to_string())
        ));

        // Note: Shaka packager doesn't accept `aac` or `mp3`, but for some reason
        //       it take `mp4` as a container for audio.
        // ffmpeg -i input.mkv -map 0:a:0 audio_eng.mp4
        let output = Command::new("ffmpeg")
            .args([
                "-y",
                "-i",
                self.file.to_str().unwrap(),
                "-map", // -map automatically disables automatic stream selection so no need to exclude video and subs streams (with `-vn` and `-sn`)
                format!("0:a:{}", audio.index).as_str(), // Audio stream selection.
                output_file.to_str().unwrap(),
            ])
            .output()
            .await;

        if let Err(err) = output {
            eprintln!("ffmpeg error: {}", err);
            return Err(AppError::TranscodeError(err.to_string()));
        }

        let suffix = audio.language.clone().unwrap_or(audio.index.to_string());
        let name = audio
            .language
            .clone()
            .unwrap_or(format!("Track {}", audio.index + 1));
        self.streams.insert(
            format!(
                "in=audio_{suffix}.mp4,stream=audio,segment_template=audio_{suffix}/$Number$.aac,playlist_name=audio_{suffix}/main.m3u8,hls_group_id=audio,hls_name={name}", suffix = suffix, name = name
            )
        );

        Ok(output_file)
    }
    pub async fn extract_subtitles(&mut self, subtitle_stream: SubtitleStream) -> Result<PathBuf> {
        let output_file = self.output_dir.join(format!(
            "subtitles_{}.vtt",
            subtitle_stream
                .language
                .clone()
                .unwrap_or(subtitle_stream.index.to_string())
        ));

        // ffmpeg -i input.mkv -map 0:s:0 subtitles.vtt
        let output = Command::new("ffmpeg")
            .args([
                "-y",
                "-i",
                self.file.to_str().unwrap(),
                "-map",
                format!("0:s:{}", subtitle_stream.index).as_str(),
                output_file.to_str().unwrap(),
            ])
            .output()
            .await;

        if let Err(err) = output {
            eprintln!("ffmpeg error: {}", err);
            return Err(AppError::TranscodeError(err.to_string()));
        }

        let suffix = subtitle_stream
            .language
            .clone()
            .unwrap_or(subtitle_stream.index.to_string());
        let name = subtitle_stream
            .language
            .clone()
            .unwrap_or(format!("Track {}", subtitle_stream.index + 1));
        self.streams.insert(format!("in=subtitles_{suffix}.vtt,stream=text,segment_template=text_{suffix}/$Number$.vtt,playlist_name=text_{suffix}/main.m3u8,hls_group_id=text,hls_name={name}", suffix = suffix, name = name));

        Ok(output_file)
    }

    /// Packages the encoded files for HLS Streaming using Google's Shaka packager
    ///
    /// Note: This function takes ownership of the instance, so create a new instance
    ///       to process other videos.
    pub async fn package(self) -> Result<PathBuf> {
        // IMPORTANT: Both ffmpeg and Google's Shaka packager need to be installed. Shaka installs by
        //            default as `packager`, I have renamed it in my system to `shaka`
        //            solely to distinguish it

        //shaka \
        //   'in=audio_eng.mp4,stream=audio,segment_template=audio_eng/$Number$.aac,playlist_name=audio_eng/main.m3u8,hls_group_id=audio,hls_name=ENGLISH' \
        //   'in=audio_jap.mp4,stream=audio,segment_template=audio_jap/$Number$.aac,playlist_name=audio_jap/main.m3u8,hls_group_id=audio,hls_name=JAPANESE' \
        //   'in=subtitles.vtt,stream=text,segment_template=text/$Number$.vtt,playlist_name=text/main.m3u8,hls_group_id=text,hls_name=ENGLISH' \
        //   'in=video_480p.mp4,stream=video,segment_template=h264_480p/$Number$.ts,playlist_name=h264_480p/main.m3u8,iframe_playlist_name=h264_480p/iframe.m3u8' \
        //   'in=video_720p.mp4,stream=video,segment_template=h264_720p/$Number$.ts,playlist_name=h264_720p/main.m3u8,iframe_playlist_name=h264_720p/iframe.m3u8' \
        //   --hls_master_playlist_output h264_master.m3u8

        let mut command = Command::new("shaka");

        for stream in self.streams {
            command.arg(stream);
        }

        command
            .arg("--hls_master_playlist_output")
            .arg("h264_master.m3u8");

        let output = command.current_dir(&self.output_dir).output().await;

        if let Err(err) = output {
            eprintln!("Failed to transcode: {}", err);
            return Err(AppError::PackagerError(err.to_string()));
        }

        Ok(self.output_dir.join("h264_master.m3u8"))
    }
}
