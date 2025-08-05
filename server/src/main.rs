use std::env;

use image::{Directory, ImageBundle};

mod image;
mod repo;

fn main() {
    let mut args = env::args();

    if let Some(arg) = args.nth(1) {
        println!("{arg}");

        let dir = Directory::scan(arg);

        let bundles = ImageBundle::from_directory(&dir);

        println!("{} bundles created", bundles.len());

        for (i, bundle) in bundles.iter().enumerate() {
            bundle.create_thumbnails(format!("thumb_{i}.jpg"));
        }

        dir.save("first.json", &bundles);
    }

    //let repo = Repository::open();
    //repo.create_schema();
}
