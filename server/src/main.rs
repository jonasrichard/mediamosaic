use std::env;

use image::{Directory, ImageBundle};

mod image;
mod repo;

fn main() {
    let mut args = env::args();

    if let Some(arg) = args.nth(1) {
        println!("{arg}");

        let dir = Directory::scan(arg);

        dir.print();

        let bundles = ImageBundle::from_directory(&dir);

        bundles.first().unwrap().create_thumbnails();
    }

    //let repo = Repository::open();
    //repo.create_schema();
}
