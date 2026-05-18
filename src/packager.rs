use crate::infrastructure::packager::Packager;

mod api;
mod domain;
pub mod errors;
mod infrastructure;
mod prelude;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: kiroku-packager <path-to-input> <path-to-output>");
        std::process::exit(1);
    }

    let file_name = &args[1];
    let output_dir = &args[2];

    let packager = Packager::new(file_name).unwrap();

    let metadata = packager.get_metadata().await.unwrap();

    println!("{:?}", metadata);

    // packager.encode(&metadata, output_dir).await.unwrap();
    packager.package(&metadata, output_dir).await.unwrap();
}
