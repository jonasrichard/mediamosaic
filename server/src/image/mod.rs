use std::{
    fs::{DirEntry, FileType},
    path::Path,
    time::Instant,
};

use image::ImageReader;

pub struct Directory {
    id: u32,
    path: Path,
    file_count: u32,
    total_size: u64,
    scanned_at: Instant,
    images: Vec<Image>,
}

pub struct Image {
    file_path: Path,
    width: u32,
    height: u32,
    size: u32,
}

impl Directory {
    pub fn scan(path: impl AsRef<Path>) -> Self {
        let mut images = Vec::new();

        for f in path.as_ref().read_dir().unwrap() {
            println!("{f:?}");

            let entry = f.unwrap();

            if Directory::is_image(&entry) {
                images.push(Image {
                    file_path: entry.path(),
                    width: 0,
                    height: 0,
                    size: 0,
                });
            }
        }

        Directory {
            id: 0,
            path: path.as_ref().deref(),
            file_count: 0,
            total_size: 0,
            scanned_at: Instant::now(),
            images,
        }
    }

    pub fn is_image(entry: &DirEntry) -> bool {
        if entry.file_type().unwrap().is_file() {
            return entry.path().ends_with(".jpg");
        }

        false
    }
}

impl Image {
    pub fn peek_into(entry: DirEntry) -> Self {
        let dim = ImageReader::open(entry.path())
            .unwrap()
            .into_dimensions()
            .unwrap();

        Image {
            file_path: entry.path(),
            width: dim.0,
            height: dim.1,
            size: 0,
        }
    }

    pub fn create_thumbnail(&self) {
        let img = ImageReader::open(&self.file_path)
            .unwrap()
            .decode()
            .unwrap();

        let thumb = img.thumbnail(128, 128);

        //thumb.save(
    }
}
