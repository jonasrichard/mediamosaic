use std::env;

use image::Directory;
use repo::Repository;

mod image;
mod repo;

fn main() {
    let mut args = env::args();

    if let Some(arg) = args.nth(1) {
        println!("{arg}");

        Directory::scan(arg);
    }

    let repo = Repository::open();

    repo.create_schema();
}
