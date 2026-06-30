use std::{path::Path, sync::Arc};

use axum::{body::Body, extract, response::Response};
use http::StatusCode;
use log::info;

use crate::AppState;

#[utoipa::path(
    get,
    path = "/file/serve/{path}",
    params(
        ("path" = String, Path, description = "Relative file path to serve")
    ),
    responses(
        (status = 200, description = "File content returned"),
        (status = 400, description = "Path points to a directory", body = String)
    ),
    tag = "media"
)]
pub async fn serve_file(
    extract::Path(file): extract::Path<String>,
    state: Arc<AppState>,
) -> Result<Response<Body>, (StatusCode, String)> {
    let base_path = Path::new(&state.config.root_directory);
    let mut rel_path = Path::new(&file);

    if rel_path.is_absolute() {
        rel_path = rel_path.strip_prefix("/").expect("Cannot join file paths");
    }

    let full_dir = base_path.join(rel_path);

    if full_dir.is_dir() {
        return Err((StatusCode::BAD_REQUEST, "Path is a directory".to_string()));
    }

    info!("Serving file: {full_dir:?}");

    Ok(super::serve_file(&full_dir))
}
