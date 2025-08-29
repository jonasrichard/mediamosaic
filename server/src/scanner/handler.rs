use std::{
    fs::DirEntry,
    io::{BufWriter, Cursor, Write},
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{Json, body::Body, extract, response::Response};
use http::{HeaderValue, header};
use log::{debug, info};
use tokio::sync::mpsc;

use crate::{
    AppState,
    scanner::{bundle::ImageBundle, directory::ScannerContext},
};

#[derive(Debug)]
pub enum SyncCommand {
    /// Sync the images in the directory. The first is the root path and the
    /// second is the relative path inside the root.
    SyncDirectory(PathBuf, String),
}

pub async fn sync_directory(mut commands: mpsc::Receiver<SyncCommand>) {
    while let Some(command) = commands.recv().await {
        debug!("Sync command: {command:?}");

        let SyncCommand::SyncDirectory(base_dir, relative_dir) = command;
        let context = ScannerContext::new(&base_dir);

        let directory = context.scan(relative_dir);
        let bundles = ImageBundle::from_directory(&directory);

        debug!("{} bundles created", bundles.len());

        directory.save(&bundles);
    }
}

pub async fn directory_sync_handler(
    extract::Path(dir): extract::Path<String>,
    state: Arc<AppState>,
) {
    debug!("Request to sync dir {dir}");

    let context = ScannerContext::new(&state.config.root_directory);
    let full_path = context.to_absolute_path(&dir);

    if full_path.join("bundles.json").exists() {
        let entries = full_path.read_dir().unwrap();

        for entry in entries {
            let entry2 = entry.unwrap();
            let name = entry2.file_name();

            if name == "bundles.json" || name.to_str().unwrap().starts_with("thumbs") {
                std::fs::remove_file(entry2.path()).expect("Cannot remove file");
            }
        }
    }

    state
        .command_tx
        .send(SyncCommand::SyncDirectory(context.base_dir, dir))
        .await
        .expect("Failed to send internal command");
}

pub async fn serve_content(
    extract::Path(dir): extract::Path<String>,
    state: Arc<AppState>,
) -> Response<Body> {
    debug!("Serving path: {dir}");
    debug!("Root directory: {}", state.config.root_directory);

    let base_path = Path::new(&state.config.root_directory);
    let mut rel_path = Path::new(&dir);

    if rel_path.is_absolute() {
        rel_path = rel_path.strip_prefix("/").expect("Cannot join file paths");
    }

    let full_dir = base_path.join(rel_path);

    debug!("  To filepath: {full_dir:?}");
    debug!("  Is dir?: {}", full_dir.is_dir());

    if full_dir.is_dir() {
        if full_dir.join("bundles.json").exists() {
            let gallery_page = std::fs::read_to_string(&state.config.gallery_index).unwrap();
            let body: Body = Body::new(gallery_page);
            let mut response: Response<Body> = Response::builder().body(body).unwrap();

            response
                .headers_mut()
                .insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));

            response
        } else {
            list_directory(base_path, &full_dir)
        }
    } else {
        debug!("  Serving file: {full_dir:?}");

        serve_file(&full_dir)
    }
}

pub async fn delete_image(
    extract::Path(dir): extract::Path<String>,
    state: Arc<AppState>,
) -> Response<Body> {
    let base_path = Path::new(&state.config.root_directory);
    let full_path = base_path.join(dir);

    debug!("Deleting file: {full_path:?}");

    if full_path.exists() && full_path.is_file() {
        match std::fs::remove_file(&full_path) {
            Ok(_) => {
                let body = Body::from(format!("Deleted file: {}", full_path.to_string_lossy()));
                Response::builder().status(200).body(body).unwrap()
            }
            Err(e) => {
                let body = Body::from(format!(
                    "Failed to delete file: {}. Error: {}",
                    full_path.to_string_lossy(),
                    e
                ));
                Response::builder().status(500).body(body).unwrap()
            }
        }
    } else {
        let body = Body::from(format!("File not found: {}", full_path.to_string_lossy()));
        Response::builder().status(404).body(body).unwrap()
    }
}

pub async fn delete_images(
    state: Arc<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response<Body> {
    if let serde_json::Value::Array(files) = payload {
        info!("Files to delete: {files:?}");

        let base_path = Path::new(&state.config.root_directory);

        for file in files {
            if let serde_json::Value::String(file_str) = file {
                let full_path = base_path.join(&file_str);

                if full_path.exists() && full_path.is_file() {
                    match std::fs::remove_file(&full_path) {
                        Ok(_) => info!("Deleted file: {}", full_path.to_string_lossy()),
                        Err(e) => info!(
                            "Failed to delete file: {}. Error: {}",
                            full_path.to_string_lossy(),
                            e
                        ),
                    }
                } else {
                    info!("File not found: {}", full_path.to_string_lossy());
                }
            }
        }
    }

    Response::builder().body("".into()).unwrap()
}

fn list_directory(base: &std::path::Path, dir: &std::path::Path) -> Response<Body> {
    let mut buffer = Cursor::new(Vec::new());

    let mut writer = BufWriter::new(&mut buffer);

    let mut entries: Vec<_> = dir.read_dir().unwrap().map(Result::unwrap).collect();

    entries.sort_by(|e1: &DirEntry, e2: &DirEntry| {
        e1.file_name().partial_cmp(&e2.file_name()).unwrap()
    });

    let base_prefix = base.to_str().unwrap();

    let relative_parent = base.parent().unwrap().to_str().unwrap();

    let _ = writer.write("<html><body>".as_bytes()).unwrap();

    writer
        .write_fmt(format_args!(
            "<a href=\"/serve/{:?}\">Parent</a><br/>",
            relative_parent
        ))
        .unwrap();

    for entry in &entries {
        let entry_path = entry.path();
        let entry_link = entry_path.strip_prefix(base_prefix).unwrap();

        let serve_link = Path::new("/serve").join(entry_link);

        writer
            .write_fmt(format_args!(
                "<a href=\"{}/\">{:?}</a><br/>",
                serve_link.to_str().unwrap(),
                entry.file_name()
            ))
            .unwrap();
    }

    debug!("  Creating index link {dir:?}");

    let sync_link = Path::new("/sync").join(dir.strip_prefix(base_prefix).unwrap());

    writer
        .write_fmt(format_args!(
            "<br><a href=\"{}/\">Index</a></body></html>",
            sync_link.to_str().unwrap()
        ))
        .unwrap();

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
