use std::{fs::File, sync::Arc};

use axum::{
    Router,
    response::Redirect,
    routing::{get, post},
};
use log::info;
use mosaic_media::thumbnail;
use serde::Deserialize;
use tokio::net::TcpListener;

mod api;

#[derive(Deserialize)]
pub struct Config {
    pub gallery_index: String,
    logfile: String,
    port: u16,
    pub root_directory: String,
}

pub struct AppState {
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
    let config = read_args();

    init_logger(&config.logfile);

    let bind_addr = format!("0.0.0.0:{}", config.port);

    let state = Arc::new(AppState {
        config,
    });

    let app = Router::new()
        .route("/", get(|| async { Redirect::permanent("/serve/") }))
        .route(
            "/delete/{*path}",
            get({
                let shared_state = Arc::clone(&state);
                move |path| api::delete_image(path, shared_state)
            }),
        )
        .route(
            "/delete",
            post({
                let shared_state = Arc::clone(&state);
                move |body| api::delete_images(shared_state, body)
            }),
        );
    let app = api::routes(app, Arc::clone(&state));

    let listener = TcpListener::bind(bind_addr).await.unwrap();

    info!("Starting HTTP serve on :{}", state.config.port);

    axum::serve(listener, app).await.unwrap();
}

fn read_args() -> Config {
    let mut serve_path = None;
    let mut config_path = Some("mosaic.toml".to_string());
    let mut args = std::env::args();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => {
                if let Some(arg) = args.next() {
                    config_path = Some(arg);
                } else {
                    eprintln!("Expected argument after --config");
                    std::process::exit(1);
                }
            }
            "--path" => {
                if let Some(arg) = args.next() {
                    serve_path = Some(arg);
                } else {
                    eprintln!("Expected argument after --path");
                    std::process::exit(1);
                }
            }
            _ => {}
        }
    }

    println!("Reading config from: {}", config_path.as_ref().unwrap());

    let cfg_file = std::fs::read_to_string(config_path.unwrap()).expect("Cannot find mosaic.toml");
    let mut config: Config = toml::from_str(&cfg_file).expect("Error parsing mosaic.toml");

    if serve_path.is_some() {
        config.root_directory = serve_path.unwrap();
    }

    config
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
