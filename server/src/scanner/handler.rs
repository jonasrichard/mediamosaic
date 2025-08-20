use std::{
    collections::HashMap,
    fs::DirEntry,
    io::{BufWriter, Cursor, Write},
    path::PathBuf,
    sync::Arc,
};

use axum::extract::{Path, Query};
use log::debug;
use tokio::sync::mpsc;

use crate::{
    AppState,
    scanner::{bundle::ImageBundle, directory::Directory},
};

#[derive(Debug)]
pub enum SyncCommand {
    SyncDirectory(String),
}

pub async fn sync_directory(mut commands: mpsc::Receiver<SyncCommand>) {
    while let Some(command) = commands.recv().await {
        debug!("Sync command: {command:?}");

        let SyncCommand::SyncDirectory(dir) = command;

        let directory = Directory::scan(dir);
        let bundles = ImageBundle::from_directory(&directory);

        debug!("{} bundles created", bundles.len());

        directory.save(&bundles);
    }
}

pub async fn directory_sync_handler(
    Query(params): Query<HashMap<String, String>>,
    state: Arc<AppState>,
) {
    let dir = params.get("dir").unwrap();

    debug!("Request to sync dir {dir}");

    let base_path = std::path::Path::new(&state.config.root_directory);
    let full_dir = base_path.join(dir).to_str().unwrap().to_owned();

    state
        .command_tx
        .send(SyncCommand::SyncDirectory(full_dir))
        .await
        .expect("Failed to send internal command");
}

pub async fn serve_content(Path(path): Path<String>, state: Arc<AppState>) -> String {
    debug!("Serving path: {path}");
    debug!("Root directory: {}", state.config.root_directory);

    let base_path = std::path::Path::new(&state.config.root_directory);
    let full_dir = base_path.join(path);

    debug!("  To filepath: {full_dir:?}");
    debug!("  Is dir?: {}", full_dir.is_dir());

    if full_dir.is_dir() {
        list_directory(&full_dir)
    } else {
        String::new()
    }
}

fn list_directory(dir: &PathBuf) -> String {
    let mut buffer = Cursor::new(Vec::new());

    let mut writer = BufWriter::new(&mut buffer);

    let mut entries: Vec<_> = dir.read_dir().unwrap().map(Result::unwrap).collect();

    entries.sort_by(|e1: &DirEntry, e2: &DirEntry| {
        e1.file_name().partial_cmp(&e2.file_name()).unwrap()
    });

    for entry in &entries {
        writer
            .write_fmt(format_args!("{:?}", entry.file_name()))
            .unwrap();
    }

    drop(writer);

    let content = buffer.into_inner();

    String::from_utf8(content).unwrap()
}
