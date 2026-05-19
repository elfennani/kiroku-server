use crate::infrastructure::packager::Packager;
use log::info;
use std::collections::HashSet;

mod api;
mod domain;
pub mod errors;
mod infrastructure;
mod prelude;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    env_logger::init();

    if args.len() != 3 {
        eprintln!("Usage: kiroku-packager <path-to-input> <path-to-output>");
        std::process::exit(1);
    }

    let file_name = &args[1];
    let output_dir = &args[2];

    let mut packager = Packager::new(file_name, output_dir).unwrap();
    info!("Initialized packager for: {}", &file_name);

    let metadata = packager.get_metadata().await.unwrap();

    let mut temp_files = HashSet::new();

    info!("Transcoding Video in 720p");
    temp_files.insert(packager.transcode_video(720).await.unwrap());
    info!("Transcoding Video in 1080p");
    temp_files.insert(packager.transcode_video(1080).await.unwrap());

    for audio in metadata.audio {
        info!(
            "Transcoding Audio Stream #{} ({})",
            audio.index, audio.title
        );
        temp_files.insert(packager.transcode_audio(audio).await.unwrap());
    }

    for subtitle in metadata.subtitles {
        info!(
            "Transcoding Subtitle Stream #{} ({})",
            subtitle.index, subtitle.title
        );
        temp_files.insert(packager.extract_subtitles(subtitle).await.unwrap());
    }

    info!("Packaging everything to a playlist");
    packager.package().await.unwrap();

    for file in temp_files {
        info!("Removed temporary file: {}", &file.display());
        std::fs::remove_file(file).ok();
    }
}
