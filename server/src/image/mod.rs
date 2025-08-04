use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
    time::Instant,
};

use image::{DynamicImage, ImageReader, RgbImage};

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
    file_path: PathBuf,
    width: u32,
    height: u32,
    size: u64,
}

pub struct ImageBundle<'dir> {
    total_width: u32,
    height: u32,
    images: Vec<&'dir Image>,
}

impl Directory {
    pub fn scan(path: impl AsRef<Path>) -> Self {
        let mut images = Vec::new();

        for f in path.as_ref().read_dir().unwrap() {
            let entry = f.unwrap();

            println!("{entry:?}, {}", entry.path().ends_with("jpg"));

            if Directory::is_image(&entry) {
                images.push(Image::peek_into(&entry));
            }
        }

        images.first().map(Image::create_thumbnail);

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

    pub fn print(&self) {
        for img in &self.images {
            println!("{img:?}");
        }
    }
}

impl Image {
    pub fn peek_into(entry: &DirEntry) -> Self {
        let dim = ImageReader::open(entry.path())
            .unwrap()
            .into_dimensions()
            .unwrap();

        println!("{entry:?} {dim:?}");

        Image {
            file_path: entry.path(),
            width: dim.0,
            height: dim.1,
            size: entry.metadata().unwrap().len(),
        }
    }

    pub fn create_thumbnail(&self) -> RgbImage {
        let start = Instant::now();
        let img = ImageReader::open(&self.file_path)
            .unwrap()
            .decode()
            .unwrap();

        let thumb = img.thumbnail(256, 256);

        if let DynamicImage::ImageRgb8(rgb_image) = thumb {
            println!("{:?} {:?}", self.file_path, start.elapsed());

            return rgb_image;
        }

        panic!("Image is not an RGB8 image");
    }
}

impl<'dir> ImageBundle<'dir> {
    pub fn from_directory(dir: &'dir Directory) -> Vec<ImageBundle<'dir>> {
        let mut bundles: Vec<ImageBundle> = vec![];

        for image in &dir.images {
            for bundle in bundles.iter_mut() {
                if bundle.height == image.height && bundle.images.len() < 8 {
                    bundle.images.push(image);
                    continue;
                }
            }

            let bundle = ImageBundle {
                total_width: image.width,
                height: image.height,
                images: Vec::from(&[image]),
            };

            bundles.push(bundle);
        }

        bundles
    }

    pub fn create_thumbnails(&self) {
        let thumbnails: Vec<RgbImage> = self
            .images
            .iter()
            .map(|img: &&Image| Image::create_thumbnail(img))
            .collect();
        let total_width = thumbnails
            .iter()
            .map(|i| i.width())
            .reduce(|acc, e| acc + e)
            .unwrap();
        let height = thumbnails.first().unwrap().height();

        let start = Instant::now();

        let mut thumbs = RgbImage::new(total_width, height);

        let mut x_offset = 0u32;

        for thumbnail in &thumbnails {
            for y in 0..thumbnail.height() {
                for x in 0..thumbnail.width() {
                    thumbs.put_pixel(x + x_offset, y, *thumbnail.get_pixel(x, y));
                }
            }

            println!("  thumb {:?}", start.elapsed());

            x_offset += thumbnail.width();
        }

        thumbs.save("./thumb.jpg").unwrap();

        println!("Saved {:?}", start.elapsed());
    }
}
