use std::{path::Path, time::Instant};

use image::RgbImage;
use serde::Serialize;

use super::{directory::Directory, image::Image};

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
    width: u32,
    height: u32,
    original_name: String,
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

    pub fn create_thumbnails(&self) {
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

        thumbs.save(&self.file_name).unwrap();

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
                    width: image.width,
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
