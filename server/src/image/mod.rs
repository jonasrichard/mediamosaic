use std::{
    fs::{DirEntry, File},
    io::BufWriter,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    time::Instant,
};

use image::{DynamicImage, ImageDecoder, ImageReader, RgbImage};
use serde::Serialize;

pub struct Directory {
    id: u32,
    path: PathBuf,
    file_count: u32,
    total_size: u64,
    scanned_at: Instant,
    images: Vec<Image>,
}

#[derive(Debug)]
pub struct Image {
    id: u32,
    file_path: PathBuf,
    width: u32,
    height: u32,
    size: u64,
    thumbnail: RgbImage,
}

pub struct ImageBundle<'dir> {
    id: u32,
    file_name: String,
    height: u32,
    images: Vec<&'dir Image>,
}

#[derive(Debug, Serialize)]
pub struct Thumbnail {
    thumbnail_name: String,
    position_x: u32,
    height: u32,
    original_name: String,
}

impl Directory {
    pub fn scan(path: impl AsRef<Path>) -> Self {
        let mut images = Vec::new();

        let mut entries: Vec<_> = path
            .as_ref()
            .read_dir()
            .unwrap()
            .map(Result::unwrap)
            .collect();

        entries.sort_by(|e1: &DirEntry, e2: &DirEntry| {
            e1.file_name().partial_cmp(&e2.file_name()).unwrap()
        });

        let mut id = 1u32;

        for entry in &entries {
            println!("{entry:?}, {}", entry.path().ends_with("jpg"));

            if Directory::is_image(entry) {
                images.push(Image::from_path(entry, id));

                id += 1;
            }
        }

        Directory {
            id: 0,
            path: path.as_ref().to_path_buf(),
            file_count: 0,
            total_size: 0,
            scanned_at: Instant::now(),
            images,
        }
    }

    pub fn is_image(entry: &DirEntry) -> bool {
        if entry.file_type().unwrap().is_file() {
            if let Some(ext) = entry.path().extension() {
                return ext == "jpg";
            }
        }

        false
    }

    pub fn save(&self, json_file: impl AsRef<Path>, bundles: &Vec<ImageBundle<'_>>) {
        // TODO pull in serde and create a JSON and save to a file
        // list of
        //   - thumbnail name
        //   - x position
        //   - original image name
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

impl Image {
    pub fn from_path(entry: &DirEntry, id: u32) -> Self {
        let path = entry.path();
        let thumbnail = Image::create_thumbnail(&path);

        Image {
            id,
            file_path: path,
            width: thumbnail.width(),
            height: thumbnail.height(),
            size: entry.metadata().unwrap().size(),
            thumbnail,
        }
    }

    // TODO
    // We need to follow a different method. We need to read the images, apply orientation,
    // and get the dimensions, create thumbnail and start to collect them into different
    // bundles.
    pub fn create_thumbnail(path: impl AsRef<Path>) -> RgbImage {
        let start = Instant::now();
        let mut decoder = ImageReader::open(&path).unwrap().into_decoder().unwrap();
        let orientation = decoder.orientation().unwrap();
        let mut img = DynamicImage::from_decoder(decoder).unwrap();

        img.apply_orientation(orientation);

        let thumb = img.thumbnail(256, 256);

        if let DynamicImage::ImageRgb8(rgb_image) = thumb {
            println!("{:?} {:?}", path.as_ref(), start.elapsed());

            return rgb_image;
        }

        panic!("Image is not an RGB8 image");
    }
}

impl<'dir> ImageBundle<'dir> {
    pub fn from_directory(dir: &'dir Directory) -> Vec<ImageBundle<'dir>> {
        let mut bundles: Vec<ImageBundle> = vec![];
        let mut id = 1u32;

        'outer: for image in &dir.images {
            println!(
                "Handling image {:?} {}x{}",
                image.file_path.file_name(),
                image.width,
                image.height
            );

            for bundle in bundles.iter_mut() {
                if bundle.height == image.height && bundle.images.len() < 8 {
                    println!("  Bundle found with height {}", bundle.height);

                    bundle.images.push(image);

                    continue 'outer;
                }
            }

            let bundle = ImageBundle {
                id,
                file_name: format!("thumbs_{id}.jpg"),
                height: image.height,
                images: Vec::from(&[image]),
            };

            id += 1;

            println!("  Bundle created with the image");

            bundles.push(bundle);
        }

        bundles
    }

    pub fn create_thumbnails(&self, name: impl AsRef<Path>) {
        let total_width = self
            .images
            .iter()
            .map(|i| i.thumbnail.width())
            .reduce(|acc, e| acc + e)
            .unwrap();
        let height = self.images.first().unwrap().thumbnail.height();

        let start = Instant::now();

        let mut thumbs = RgbImage::new(total_width, height);

        let mut x_offset = 0u32;

        for image in &self.images {
            for y in 0..image.thumbnail.height() {
                for x in 0..image.thumbnail.width() {
                    thumbs.put_pixel(x + x_offset, y, *image.thumbnail.get_pixel(x, y));
                }
            }

            println!("  thumb {:?}", start.elapsed());

            x_offset += image.thumbnail.width();
        }

        thumbs.save(name).unwrap();

        println!("Saved {:?}", start.elapsed());
    }

    pub fn extract_metadata(&self, id: u32) -> Option<Thumbnail> {
        let mut offset_x = 0u32;

        for (i, image) in self.images.iter().enumerate() {
            if i != 0 {
                offset_x += image.width;
            }

            if image.id == id {
                return Some(Thumbnail {
                    thumbnail_name: self.file_name.clone(),
                    position_x: offset_x,
                    height: self.height,
                    original_name: image
                        .file_path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned(),
                });
            }
        }

        None
    }
}
