use std::{
    fs::DirEntry,
    io::{BufWriter, Cursor, Write},
    ops::Deref,
    path::PathBuf,
    sync::Arc,
};

use axum::{body::Body, extract::Path, response::Response};
use http::{HeaderValue, header};
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

pub async fn directory_sync_handler(Path(dir): Path<String>, state: Arc<AppState>) {
    debug!("Request to sync dir {dir}");

    let base_path = std::path::Path::new(&state.config.root_directory);
    let full_dir = base_path.join(dir).to_str().unwrap().to_owned();

    state
        .command_tx
        .send(SyncCommand::SyncDirectory(full_dir))
        .await
        .expect("Failed to send internal command");
}

pub async fn serve_content(Path(dir): Path<String>, state: Arc<AppState>) -> Response<Body> {
    debug!("Serving path: {dir}");
    debug!("Root directory: {}", state.config.root_directory);

    let base_path = std::path::Path::new(&state.config.root_directory);
    let mut rel_path = std::path::Path::new(&dir);

    if rel_path.is_absolute() {
        rel_path = rel_path.strip_prefix("/").expect("Cannot join file paths");
    }

    let full_dir = base_path.join(rel_path);

    debug!("  To filepath: {full_dir:?}");
    debug!("  Is dir?: {}", full_dir.is_dir());

    if full_dir.is_dir() {
        if full_dir.join("bundles.json").exists() {
            let body: Body = Body::new(state.index_page.deref().to_owned());
            let mut response: Response<Body> = Response::builder().body(body).unwrap();

            response
                .headers_mut()
                .insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));

            response
        } else {
            list_directory(&base_path, &full_dir)
        }
    } else {
        debug!("  Serving file: {full_dir:?}");

        serve_file(&full_dir)
    }
}

fn list_directory(base: &std::path::Path, dir: &PathBuf) -> Response<Body> {
    let mut buffer = Cursor::new(Vec::new());

    let mut writer = BufWriter::new(&mut buffer);

    let mut entries: Vec<_> = dir.read_dir().unwrap().map(Result::unwrap).collect();

    entries.sort_by(|e1: &DirEntry, e2: &DirEntry| {
        e1.file_name().partial_cmp(&e2.file_name()).unwrap()
    });

    let base_prefix = base.to_str().unwrap();

    writer.write(format!("<html><body>").as_bytes()).unwrap();

    for entry in &entries {
        let entry_path = entry.path();
        let entry_link = entry_path.strip_prefix(base_prefix).unwrap();

        let serve_link = std::path::Path::new("/serve").join(entry_link);

        writer
            .write_fmt(format_args!(
                "<a href=\"{}\">{:?}</a><br/>",
                serve_link.to_str().unwrap(),
                entry.file_name()
            ))
            .unwrap();
    }

    drop(writer);

    let content = buffer.into_inner();

    //

    let body = Body::new(String::from_utf8(content).unwrap());
    Response::builder().body(body).unwrap()
}

fn serve_file(path: &std::path::Path) -> Response<Body> {
    let mut response = Response::builder();

    response = match path.extension().unwrap().to_str().unwrap() {
        "jpg" => response.header("Content-Type", "image/jpeg"),
        "jpeg" => response.header("Content-Type", "image/jpeg"),
        "json" => response.header("Content-Type", "application/json"),
        _ => todo!(),
    };

    let content = std::fs::read(path).unwrap();

    response.body(content.into()).unwrap()
}
