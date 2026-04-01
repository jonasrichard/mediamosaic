use std::{
    ffi::OsString,
    fs::DirEntry,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    time::Instant,
};

use image::{DynamicImage, ImageDecoder, ImageReader, RgbImage, buffer::ConvertBuffer};
use log::debug;

#[derive(Debug)]
pub struct Image {
    pub id: OsString,
    pub file_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub size: u64,
    pub thumbnail: RgbImage,
}

impl Image {
    pub fn from_path(entry: &DirEntry) -> Self {
        let path = entry.path();
        let thumbnail = Image::create_thumbnail(&path);

        Image {
            id: path.file_name().unwrap().to_os_string(),
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

        debug!("{:?} {:?}", path.as_ref(), start.elapsed());

        match thumb {
            DynamicImage::ImageLuma8(gray_image) => gray_image.convert(),
            DynamicImage::ImageRgb8(rgb_image) => rgb_image,
            _ => {
                panic!("Image is not an RGB8 image");
            }
        }
    }
}
