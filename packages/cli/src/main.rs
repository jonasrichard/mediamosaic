use std::env;

use log::debug;
use mosaic_media::{scanner::directory::Directory, thumbnail::bundle::ImageBundle};

fn main() {
    init_logger();

    let mut system_base_dir = String::from("");
    let mut root_dir = String::from("");
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match &arg as &str {
            "--system-base-dir" => {
                if let Some(value) = args.next() {
                    system_base_dir = value;
                } else {
                    eprintln!("Expected value after --system-base-dir");
                    std::process::exit(1);
                }
            }
            "--root-dir" => {
                if let Some(value) = args.next() {
                    root_dir = value;
                } else {
                    eprintln!("Expected value after --root-dir");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Unknown argument: {}", arg);
                std::process::exit(1);
            }
        }
    }

    debug!("System base directory: {}", system_base_dir);
    debug!("Root directory: {}", root_dir);

    //generate_thumbnails(&system_base_dir, &root_dir);
    let directories = image_directories(&root_dir);
    debug!("Found {} image directories", directories.len());

    for (i, directory) in directories.iter().enumerate() {
        debug!(
            "Processing directory {}/{}: {}",
            i + 1,
            directories.len(),
            directory
        );
        generate_thumbnails(&system_base_dir, directory);
    }
}

fn generate_thumbnails(system_base_dir: &str, root_dir: &str) {
    let mut scanner = Directory::new(system_base_dir, root_dir);
    let image_entries = scanner.list_images();

    debug!("Found {} images", image_entries.len());

    scanner.read_par_images(image_entries);

    let bundle = ImageBundle::from_directory(&scanner);
    scanner.save(&bundle);
}

fn image_directories(path: &str) -> Vec<String> {
    let mut directories = Vec::new();

    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            debug!("Checking directory: {}", path.to_str().unwrap());

            if is_to_be_processed(path.to_str().unwrap()) {
                directories.push(path.to_str().unwrap().to_string());
            }
        }
    }

    directories
}

fn is_to_be_processed(path: &str) -> bool {
    let image_extensions = ["jpg", "jpeg", "png"];
    let mut has_image = false;
    let mut has_bundle = false;

    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension()
                && let Some(ext_str) = ext.to_str()
                && image_extensions.contains(&ext_str.to_lowercase().as_str())
            {
                has_image = true;
            }

            if path.file_name().unwrap() == "bundles.json" {
                has_bundle = true;
            }
        }
    }

    has_image && !has_bundle
}

fn init_logger() {
    let mut builder = env_logger::builder();
    builder.filter_level(log::LevelFilter::Debug);
    builder.init();
}
