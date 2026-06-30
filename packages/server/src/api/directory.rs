use axum::{
    Json, extract,
    response::{IntoResponse, Response},
};
use log::{debug, info};
use mosaic_media::{
    scanner::{self, directory::Directory},
    thumbnail::bundle::ImageBundle,
};
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;

use crate::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct DirectoryEntry {
    name: String,
    entry_type: String,
}

#[utoipa::path(
    get,
    path = "/directory/{path}",
    params(
        ("path" = String, Path, description = "Relative directory path")
    ),
    responses(
        (status = 200, description = "Directory entries", body = [DirectoryEntry])
    ),
    tag = "media"
)]
pub async fn list_directory_handler(
    extract::Path(dir): extract::Path<String>,
    state: Arc<AppState>,
) -> Json<Vec<DirectoryEntry>> {
    debug!("Request to list dir {dir}");

    let full_path = scanner::to_absolute_path(&state.config.root_directory, &dir);
    info!("Listing directory: {full_path:?}");

    if let Ok(entries) = full_path.read_dir() {
        let result = entries
            .map(|entry| entry.unwrap())
            .map(|entry| DirectoryEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                entry_type: if entry.path().is_dir() {
                    "directory".to_string()
                } else {
                    "file".to_string()
                },
            })
            .collect();

        Json(result)
    } else {
        Json(vec![])
    }
}

#[utoipa::path(
    get,
    path = "/info/{path}",
    params(
        ("path" = String, Path, description = "Relative directory or bundle path")
    ),
    responses(
        (status = 200, description = "Directory information or bundles file content")
    ),
    tag = "media"
)]
pub async fn info_handler(
    extract::Path(dir): extract::Path<String>,
    state: Arc<AppState>,
) -> Response {
    debug!("Request to get info for {dir}");

    let full_path = scanner::to_absolute_path(&state.config.root_directory, &dir);
    info!("Getting info for: {full_path:?}");

    let bundle = full_path.join("bundles.json");
    if bundle.exists() {
        super::serve_file(&bundle)
    } else {
        if let Ok(entries) = full_path.read_dir() {
            let result: Vec<DirectoryEntry> = entries
                .map(|entry| {
                    let entry2 = entry.unwrap();
                    let name = entry2.file_name().to_string_lossy().to_string();
                    let entry_type = if entry2.path().is_dir() {
                        "directory".to_string()
                    } else {
                        "file".to_string()
                    };

                    DirectoryEntry { name, entry_type }
                })
                .collect();

            Json(result).into_response()
        } else {
            Json(Vec::<DirectoryEntry>::new()).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/directory/thumbnail/{path}",
    params(
        ("path" = String, Path, description = "Relative directory path")
    ),
    responses(
        (status = 200, description = "Thumbnail creation status", body = String, content_type = "text/plain")
    ),
    tag = "media"
)]
pub async fn create_thumbnails_handler(
    extract::Path(dir): extract::Path<String>,
    state: Arc<AppState>,
) -> String {
    debug!("Request to create thumbnails for dir {dir}");

    let full_path = super::relative_to_absolute_path(&state.config.root_directory, &dir);
    info!("Creating thumbnails for directory: {full_path:?}");

    if full_path.is_dir() {
        let mut scan_direcotry = Directory::new(&state.config.root_directory, &full_path);
        let image_enties = scan_direcotry.list_images();
        scan_direcotry.read_par_images(image_enties);

        let bundles = ImageBundle::from_directory(&scan_direcotry);

        scan_direcotry.save(&bundles);

        "Thumbnails created.".to_string()
    } else {
        "Not a directory.".to_string()
    }
}
