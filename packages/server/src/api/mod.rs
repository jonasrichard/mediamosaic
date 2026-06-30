use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{Json, Router, body::Body, extract, response::Response, routing::get};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{AppState, thumbnail::bundle::Thumbnail};

pub mod directory;
pub mod file;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct DeleteImagesRequest(pub Vec<String>);

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiMessage {
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiError {
    pub error: String,
}

pub fn routes(router: Router<()>, state: Arc<AppState>) -> Router<()> {
    router
        .route(
            "/directory{*path}",
            get({
                let shared_state = Arc::clone(&state);
                move |path| directory::list_directory_handler(path, shared_state)
            }),
        )
        .route(
            "/directory/thumbnail{*path}",
            get({
                let shared_state = Arc::clone(&state);
                move |path| directory::create_thumbnails_handler(path, shared_state)
            }),
        )
        .route(
            "/info{*path}",
            get({
                let shared_state = Arc::clone(&state);
                move |path| directory::info_handler(path, shared_state)
            }),
        )
        .route(
            "/file/serve{*path}",
            get({
                let shared_state = Arc::clone(&state);
                move |path| file::serve_file(path, shared_state)
            }),
        )
}

#[utoipa::path(
    get,
    path = "/delete/{path}",
    params(
        ("path" = String, Path, description = "Relative file path to delete")
    ),
    responses(
        (status = 200, description = "File deleted", body = String, content_type = "text/plain"),
        (status = 404, description = "File not found", body = String, content_type = "text/plain"),
        (status = 500, description = "Failed to delete file", body = String, content_type = "text/plain")
    ),
    tag = "media"
)]
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

#[utoipa::path(
    post,
    path = "/delete",
    request_body = DeleteImagesRequest,
    responses(
        (status = 200, description = "Delete operation completed", body = String, content_type = "text/plain")
    ),
    tag = "media"
)]
pub async fn delete_images(
    state: Arc<AppState>,
    Json(payload): Json<DeleteImagesRequest>,
) -> Response<Body> {
    // TODO delete the files from bundles.json at least, so that the UI should
    // not pick up those thumbnails
    {
        let files_to_delete = payload.0;
        let base_path = Path::new(&state.config.root_directory);
        let mut bundle_parent_dir = None;

        info!("Files to delete: {files_to_delete:?}");

        for file in &files_to_delete {
            let full_path = base_path.join(file);

            if bundle_parent_dir.is_none() {
                bundle_parent_dir = full_path.parent().map(|p| p.to_path_buf());
            }

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

        if let Some(mut bundle_parent_dir) = bundle_parent_dir {
            info!("Current dir: {bundle_parent_dir:?}");
            bundle_parent_dir.push("bundles.json");

            update_bundles_file(&bundle_parent_dir, &files_to_delete);
        } else {
            info!("No valid parent directory found for the files to delete.");
        }
    }

    Response::builder().body("".into()).unwrap()
}

fn update_bundles_file(bundles_path: &PathBuf, files_to_delete: &[String]) {
    info!("Updating bundles file: {bundles_path:?}");

    let content = if let Ok(c) = std::fs::read_to_string(bundles_path) {
        c
    } else {
        info!("  No bundles file found, skipping update");
        return;
    };

    let thumbnails: Vec<Thumbnail> = serde_json::from_str(&content).unwrap();
    let mut new_thumbnails = vec![];

    for t in thumbnails {
        let mut path = t.relative_base_path.clone();
        path.push_str(&t.original_name);

        info!("  Check if {path} is in dir list");

        if !files_to_delete.contains(&path) {
            new_thumbnails.push(t);
        }
    }

    let jf = File::create(bundles_path).unwrap();
    let writer = BufWriter::new(jf);

    serde_json::to_writer_pretty(writer, &new_thumbnails).unwrap();
}

pub fn serve_file(path: &std::path::Path) -> Response<Body> {
    let mut response = Response::builder();

    response = match path
        .extension()
        .unwrap()
        .to_ascii_lowercase()
        .to_str()
        .unwrap()
    {
        "jpg" => response.header("Content-Type", "image/jpeg"),
        "jpeg" => response.header("Content-Type", "image/jpeg"),
        "json" => response.header("Content-Type", "application/json"),
        _ => todo!(),
    };

    let content = std::fs::read(path).unwrap();

    response.body(content.into()).unwrap()
}

/// Convert a relative path to an absolute path based on the root directory.
pub fn relative_to_absolute_path(root: &str, relative: &str) -> PathBuf {
    let mut path = PathBuf::from(root);

    let relative_path = if relative.starts_with("/") {
        relative.strip_prefix("/").unwrap()
    } else {
        relative
    };

    path.push(relative_path);
    path
}
