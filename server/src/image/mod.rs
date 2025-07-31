use std::{path::Path, time::Instant};

pub struct Directory {
    id: u32,
    path: String,
    file_count: u32,
    total_size: u64,
    scanned_at: Instant,
}

impl Directory {
    pub fn scan(path: impl AsRef<Path>) -> Self {
        for f in path.as_ref().read_dir().unwrap() {
            println!("{f:?}");
        }

        Directory {
            id: 0,
            path: path.as_ref().to_str().unwrap().to_string(),
            file_count: 0,
            total_size: 0,
            scanned_at: Instant::now(),
        }
    }
}
