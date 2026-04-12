use std::{fs::DirEntry, path::{Path, PathBuf}};

pub mod directory;

pub fn to_absolute_path(base_dir: impl AsRef<Path>, path: impl AsRef<Path>) -> PathBuf {
    if path.as_ref().is_absolute() {
        base_dir.as_ref().to_path_buf().join(path.as_ref().strip_prefix("/").unwrap())
    } else {
        base_dir.as_ref().to_path_buf().join(path)
    }
}

pub fn to_relative_path(base_dir: impl AsRef<Path>, path: impl AsRef<Path>) -> PathBuf {
    path.as_ref()
        .strip_prefix(base_dir)
        .expect("Path is outside of base dir")
        .to_path_buf()
}

pub fn is_image(entry: &DirEntry) -> bool {
    if entry.file_type().unwrap().is_file()
        && let Some(ext) = entry.path().extension()
    {
        return ext.eq_ignore_ascii_case("jpg");
    }

    false
}
