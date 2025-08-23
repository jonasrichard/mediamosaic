use std::{
    fs::{DirEntry, File},
    io::BufWriter,
    path::{Path, PathBuf},
    time::Instant,
};

use log::debug;

use super::{bundle::ImageBundle, image::Image};

pub struct ScannerContext {
    /// The directory which is the root of the whole application.
    pub base_dir: PathBuf,
}

pub struct Directory {
    pub id: u32,
    pub absolute_path: PathBuf,
    pub relative_path: PathBuf,
    pub file_count: u32,
    pub total_size: u64,
    pub scanned_at: Instant,
    pub images: Vec<Image>,
}

impl ScannerContext {
    pub fn new(root_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: root_dir.as_ref().to_path_buf(),
        }
    }

    pub fn to_relative_path(&self, path: impl AsRef<Path>) -> PathBuf {
        path.as_ref()
            .strip_prefix(&self.base_dir)
            .expect("Path is outside of base dir")
            .to_path_buf()
    }

    pub fn to_absolute_path(&self, path: impl AsRef<Path>) -> PathBuf {
        if path.as_ref().is_absolute() {
            self.base_dir.join(path.as_ref().strip_prefix("/").unwrap())
        } else {
            self.base_dir.join(path)
        }
    }

    pub fn scan(&self, path: impl AsRef<Path>) -> Directory {
        let mut images = Vec::new();
        let abs_path = self.to_absolute_path(&path);

        let mut entries: Vec<_> = abs_path.read_dir().unwrap().map(Result::unwrap).collect();

        entries.sort_by(|e1: &DirEntry, e2: &DirEntry| {
            e1.file_name().partial_cmp(&e2.file_name()).unwrap()
        });

        let mut id = 1u32;

        for entry in &entries {
            debug!("{entry:?}, {}", entry.path().ends_with("jpg"));

            // TODO
            // Here we need to check if file mtime > related thumbnail file mtime
            if Directory::is_image(entry) {
                images.push(Image::from_path(entry, id));

                id += 1;
            }
        }

        debug!(
            "Create directory with absolute_path: {abs_path:?} and relative_path: {:?}",
            path.as_ref()
        );

        Directory {
            id: 0,
            absolute_path: abs_path,
            relative_path: path.as_ref().to_path_buf(),
            file_count: 0,
            total_size: 0,
            scanned_at: Instant::now(),
            images,
        }
    }
}

impl Directory {
    pub fn is_image(entry: &DirEntry) -> bool {
        if entry.file_type().unwrap().is_file() {
            if let Some(ext) = entry.path().extension() {
                return ext == "jpg";
            }
        }

        false
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
                if let Some(t) = bundle.extract_metadata(image.id) {
                    thumbnails.push(t);
                }
            }
        }

        let jf = File::create(json_file).unwrap();
        let writer = BufWriter::new(jf);

        serde_json::to_writer_pretty(writer, &thumbnails).unwrap();
    }
}
