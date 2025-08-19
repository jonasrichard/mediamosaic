use std::sync::Arc;

use axum::{Router, routing::get};
use scanner::handler::{self, SyncCommand};
use serde::Deserialize;
use tokio::{net::TcpListener, sync::mpsc};

mod repo;
mod scanner;

#[derive(Deserialize)]
pub struct Config {
    port: u16,
    pub root_directory: String,
}

pub struct AppState {
    pub command_tx: mpsc::Sender<SyncCommand>,
    pub config: Config,
}

#[tokio::main]
async fn main() {
    let config = read_config();
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
            "/sync",
            get({
                let shared_state = Arc::clone(&state);
                move |query| handler::directory_sync_handler(query, shared_state)
            }),
        )
        .route(
            "/serve/{*path}",
            get({
                let shared_state = Arc::clone(&state);
                move |path| handler::serve_content(path, shared_state)
            }),
        );

    let listener = TcpListener::bind(bind_addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

fn read_config() -> Config {
    let cfg_file = std::fs::read_to_string("mosaic.toml").expect("Cannot find mosaic.toml");

    toml::from_str(&cfg_file).expect("Error parsing mosaic.toml")
}
