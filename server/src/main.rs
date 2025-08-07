use std::env;

use scanner::{bundle::ImageBundle, directory::Directory};

mod repo;
mod scanner;

fn main() {
    let mut args = env::args();

    if let Some(arg) = args.nth(1) {
        println!("{arg}");

        let dir = Directory::scan(arg);

        let bundles = ImageBundle::from_directory(&dir);

        println!("{} bundles created", bundles.len());

        dir.save("first.json", &bundles);
    }

    //let repo = Repository::open();
    //repo.create_schema();
}
