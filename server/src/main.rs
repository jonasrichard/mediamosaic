use std::{fs::File, sync::Arc};

use axum::{
    Router,
    routing::{get, post},
};
use log::info;
use scanner::handler::{self, SyncCommand};
use serde::Deserialize;
use tokio::{net::TcpListener, sync::mpsc};

mod scanner;

#[derive(Deserialize)]
pub struct Config {
    pub gallery_index: String,
    logfile: String,
    port: u16,
    pub root_directory: String,
}

pub struct AppState {
    pub command_tx: mpsc::Sender<SyncCommand>,
    pub config: Config,
}

// TODO
// create index_dir.html and index_thumbs.html
// if in the directory there is a thumbs.json we use the index_thumbs file,
// if there is not, let us make a directory listing with the other file.
// We can always use fetch('./thumbs.json') to get the file from the current
// directory

#[tokio::main]
async fn main() {
    let config = read_config();

    init_logger(&config.logfile);

    let bind_addr = format!("0.0.0.0:{}", config.port);
    let (cmd_tx, cmd_rx) = mpsc::channel(16);

    tokio::spawn(async move {
        scanner::handler::sync_directory(cmd_rx).await;
    });

    let state = Arc::new(AppState {
        command_tx: cmd_tx,
        config,
    });

    let app = Router::new()
        .route(
            "/sync/{*path}",
            get({
                let shared_state = Arc::clone(&state);
                move |path| handler::directory_sync_handler(path, shared_state)
            }),
        )
        .route(
            "/serve{*path}",
            get({
                let shared_state = Arc::clone(&state);
                move |path| handler::serve_content(path, shared_state)
            }),
        )
        .route(
            "/delete/{*path}",
            get({
                let shared_state = Arc::clone(&state);
                move |path| scanner::handler::delete_image(path, shared_state)
            }),
        )
        .route(
            "/delete",
            post({
                let shared_state = Arc::clone(&state);
                move |body| scanner::handler::delete_images(shared_state, body)
            }),
        );

    let listener = TcpListener::bind(bind_addr).await.unwrap();

    info!("Starting HTTP serve on :3000");

    axum::serve(listener, app).await.unwrap();
}

fn read_config() -> Config {
    let cfg_file = std::fs::read_to_string("mosaic.toml").expect("Cannot find mosaic.toml");

    toml::from_str(&cfg_file).expect("Error parsing mosaic.toml")
}

fn init_logger(logfile: &str) {
    use env_logger::Target;

    let mut builder = env_logger::builder();

    builder.filter_level(log::LevelFilter::Debug);

    if logfile != "stdout" {
        let logfile = File::create("./mosaic.log").expect("Failed to open logfile");

        builder.target(Target::Pipe(Box::new(logfile)));
    }

    builder.init();
}
