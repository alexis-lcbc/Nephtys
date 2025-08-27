use actix_cors::Cors;
use actix_files::Files;
use actix_web::{
    App, HttpResponse, HttpServer, Responder, get,
    middleware::from_fn,
    post,
    web::{self, Data},
};
use argon2::password_hash::{Salt, SaltString, rand_core::OsRng};
use chrono::Local;
use rand::distr::SampleString;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    process::Command,
    sync::{
        Mutex,
        mpsc::{self},
    },
    thread::{self},
    time::{self, Duration, Instant},
};

use crate::routes::auth::{check_token_middleware, create_account, get_check_token};
pub mod movement_detector;
pub mod routes;

const HOSTNAME: &str = "nephtys.local";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    port: u16,
    camera_path: String,
    username: String,
    pass_hash: String,
    salt: String,
}

#[derive(Debug)]
struct AppState {
    config: Config,
    tokens: HashMap<String, time::Instant>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Loading configuration");
    let config = load_config();
    println!("starting ffmpeg hosting thread");
    start_ffmpeg_webcam_streaming("/dev/video0".to_string());
    let (mov_detect_tx, mov_detect_rx) = mpsc::channel::<bool>();

    println!("starting camera detect thread");
    movement_detector::start_movement_detect_thread(mov_detect_tx);

    thread::spawn(move || {
        loop {
            for _ in &mov_detect_rx {
                let now = Local::now();
                println!("{:?} movement detected", now.to_rfc3339());
            }
        }
    });

    println!("starting web server");
    env_logger::init();
    let mut app_data = Data::new(Mutex::new(AppState {
        config: config.clone(),
        tokens: HashMap::<String, Instant>::new(),
    }));
    HttpServer::new(move || {
        let auth_protected_scope = web::scope("/protected")
            .wrap(from_fn(check_token_middleware))
            .service(Files::new("/stream", "./static/stream").show_files_listing())
            .service(get_check_token);

        App::new()
            .app_data(app_data.clone())
            .service(create_account)
            .service(hello)
            .service(auth_protected_scope)
            .wrap(Cors::permissive())
    })
    .bind(("127.0.0.1", load_config().port))?
    .run()
    .await
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Nephtys server running")
}

fn load_config() -> Config {
    let salt = SaltString::generate(&mut OsRng);
    let mut config = Config {
        camera_path: "/dev/video0".to_string(),
        port: 8080,
        username: "".to_string(),
        pass_hash: "".to_string(),
        salt: salt.to_string(),
    };
    match fs::read_to_string("./config.toml") {
        Ok(s) => match toml::from_str(s.as_str()) {
            Ok(conf) => config = conf,
            Err(_) => panic!("Couldn't parse config.toml please check the file."),
        },
        Err(_) => fs::write(
            "./config.toml",
            toml::to_string_pretty(&config).expect("Couldn't parse default configuration"),
        )
        .expect(
            "Missing config.toml & Couldn't write the default config to it. Check permissions.",
        ),
    }

    return config;
}

pub enum WriteConfigError {
    FileSystemError,
    ParsingError,
}

pub fn write_config(config: &Config) -> Result<(), WriteConfigError> {
    let parsed = toml::to_string_pretty(&config);
    match parsed {
        Ok(parsed) => match fs::write("./config.toml", parsed) {
            Ok(_) => Ok(()),
            Err(_) => Err(WriteConfigError::FileSystemError),
        },
        Err(_) => return Err(WriteConfigError::ParsingError),
    }
}

fn start_ffmpeg_webcam_streaming(input: String) {
    thread::spawn(move || {
        Command::new("ffmpeg")
            .args([
                "-f",
                "v4l2",
                "-input_format",
                "mjpeg",
                "-video_size",
                "1280x720",
                // "-framerate", "30",
                "-vsync",
                "0",
                "-i",
                input.as_str(),
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-tune",
                "zerolatency",
                "-f",
                "hls",
                "-hls_flags",
                "delete_segments+independent_segments+split_by_time",
                "-hls_segment_type",
                "fmp4",
                "-hls_time",
                "1",
                "static/stream/stream.m3u8",
            ])
            .output()
            .expect("FATAL: Couldn't start FFMPEG");
    });
}
