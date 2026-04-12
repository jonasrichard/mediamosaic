use std::{
    fs::{DirEntry, File},
    io::BufWriter,
    path::{Path, PathBuf},
    time::Instant,
};

use log::debug;
use rayon::prelude::*;

use crate::thumbnail::{bundle::ImageBundle, image::Image};

pub struct Directory {
    pub id: u32,
    pub absolute_path: PathBuf,
    pub relative_path: PathBuf,
    pub file_count: u32,
    pub total_size: u64,
    pub scanned_at: Instant,
    pub images: Vec<Image>,
}

impl Directory {
    pub fn new(system_base_dir: impl AsRef<Path>, root_dir: impl AsRef<Path>) -> Self {
        let base_dir = root_dir.as_ref().to_path_buf();
        let relative_path = crate::scanner::to_relative_path(system_base_dir, &base_dir);

        Directory {
            id: 0,
            absolute_path: base_dir,
            relative_path: relative_path,
            file_count: 0,
            total_size: 0,
            scanned_at: Instant::now(),
            images: vec![],
        }
    }

    /// List all image files in the directory, and sort them by name.
    pub fn list_images(&self) -> Vec<DirEntry> {
        let mut entries = self
            .absolute_path
            .read_dir()
            .unwrap()
            .map(Result::unwrap)
            .filter(|e| crate::scanner::is_image(e))
            .collect::<Vec<_>>();

        entries.sort_by(|e1: &DirEntry, e2: &DirEntry| {
            e1.file_name().partial_cmp(&e2.file_name()).unwrap()
        });
        entries
    }

    /// Read image file and create thumbnail for it.
    pub fn read_image(&self, entry: &DirEntry) -> Image {
        debug!("Creating thumbnail for entry: {:?}", entry.path());

        Image::from_path(entry)
    }

    /// Read all image files in the directory, and create thumbnails for them in parallel.
    pub fn read_par_images(&mut self, images: Vec<DirEntry>) {
        self.images = images
            .par_iter()
            .map(|entry| self.read_image(entry))
            .collect();
        self.file_count = self.images.len() as u32;
        self.total_size = self.images.iter().map(|img| img.size).sum();
    }

    // TODO
    // here we also need to calculate a hash from the names of the file names
    // and mtimes, so if someone delete a file, it should sync
    pub fn save(&self, bundles: &Vec<ImageBundle<'_>>) {
        let json_file = self.absolute_path.join("bundles.json");

        for bundle in bundles {
            bundle.create_thumbnails();
        }

        let mut thumbnails = vec![];

        for image in &self.images {
            for bundle in bundles {
                if let Some(t) = bundle.extract_metadata(&image.id) {
                    thumbnails.push(t);
                }
            }
        }

        let jf = File::create(json_file).unwrap();
        let writer = BufWriter::new(jf);

        serde_json::to_writer_pretty(writer, &thumbnails).unwrap();
    }
}
