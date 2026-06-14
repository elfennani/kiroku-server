use crate::errors::AppError;
use crate::prelude::*;
use log::debug;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct MediaMetadata {
    /// Title in `tags.title`, fallback to filename (without the extension)
    pub title: String,
    /// Duration of the video In milliseconds
    pub duration: u64,
    pub chapters: Vec<Chapter>,
    pub audio: Vec<AudioStream>,
    pub subtitles: Vec<SubtitleStream>,
}

#[derive(Debug, Clone)]
pub struct Chapter {
    pub index: usize,
    pub start: u64,
    pub end: u64,
    pub title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AudioStream {
    pub index: usize,
    pub title: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SubtitleStream {
    pub index: usize,
    pub title: String,
    pub language: Option<String>,
}

impl MediaMetadata {
    pub fn from_serde_value(value: serde_json::Value) -> Result<MediaMetadata> {
        let mut data = serde_json::from_value::<FfprobeMetadata>(value)?;
        let mut audio: Vec<AudioStream> = vec![];
        let mut subtitles: Vec<SubtitleStream> = vec![];
        let mut chapters: Vec<Chapter> = vec![];
        data.streams.sort_by_key(|s| s.index);
        data.chapters.sort_by_key(|c| c.start);

        // TODO: Handle multiple subtitles in the same language
        for stream in data.streams {
            if stream.codec_type == "audio" {
                audio.push(AudioStream {
                    index: audio.len(),
                    title: match stream.tags.get("title") {
                        None => format!("Track {}", audio.len() + 1),
                        Some(title) => title.to_owned(),
                    },
                    language: stream.tags.get("language").map(|s| s.to_owned()),
                })
            } else if stream.codec_type == "subtitle" {
                subtitles.push(SubtitleStream {
                    index: subtitles.len(),
                    title: match stream.tags.get("title") {
                        None => format!("Track {}", audio.len() + 1),
                        Some(title) => title.to_owned(),
                    },
                    language: stream.tags.get("language").map(|s| s.to_owned()),
                })
            }
        }

        for chapter in data.chapters {
            chapters.push(Chapter {
                index: chapters.len(),
                start: (f32::from_str(chapter.start_time.as_str()).map_err(|_err| {
                    AppError::InternalServer("Failed to parse start_time".to_owned())
                })? * 1000f32)
                    .round() as u64,
                end: (f32::from_str(chapter.end_time.as_str()).map_err(|_err| {
                    AppError::InternalServer("Failed to parse end_time".to_owned())
                })? * 1000f32)
                    .round() as u64,
                title: chapter.tags.get("title").map(|s| s.to_owned()),
            })
        }

        let file = PathBuf::from_str(data.format.filename.as_str()).ok();
        let file_stem = file
            .and_then(|p| p.file_stem().map(|s| s.to_owned()))
            .and_then(|s| s.to_str().map(|s| s.to_owned()));

        let filename = match file_stem {
            None => {
                return Err(AppError::InternalServer(
                    "Failed to get file stem".to_owned(),
                ));
            }
            Some(filename) => filename.to_owned(),
        };

        let duration = (f32::from_str(data.format.duration.as_str())
            .map_err(|_err| AppError::InternalServer("Failed to parse duration".to_owned()))?
            * 1000f32)
            .round() as u64;

        debug!("duration: {}", duration);

        Ok(MediaMetadata {
            title: data
                .format
                .tags
                .get("title")
                .map(|s| s.to_owned())
                .unwrap_or(filename),
            duration,
            chapters,
            audio,
            subtitles,
        })
    }
}

#[derive(Deserialize)]
struct FfprobeMetadata {
    streams: Vec<FfprobeStream>,
    chapters: Vec<FfprobeChapter>,
    format: FfprobeFormat,
}

#[derive(Deserialize)]
struct FfprobeStream {
    index: usize,
    codec_type: String,
    tags: HashMap<String, String>,
}

#[derive(Deserialize)]
struct FfprobeChapter {
    start: u64,
    start_time: String,
    end_time: String,
    tags: HashMap<String, String>,
}

#[derive(Deserialize)]
struct FfprobeFormat {
    filename: String,
    duration: String,
    tags: HashMap<String, String>,
}
