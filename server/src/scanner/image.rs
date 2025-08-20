use std::{
    fs::DirEntry,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    time::Instant,
};

use image::{DynamicImage, ImageDecoder, ImageReader, RgbImage};
use log::debug;

#[derive(Debug)]
pub struct Image {
    pub id: u32,
    pub file_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub size: u64,
    pub thumbnail: RgbImage,
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
            debug!("{:?} {:?}", path.as_ref(), start.elapsed());

            return rgb_image;
        }

        panic!("Image is not an RGB8 image");
    }
}
