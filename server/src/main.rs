use std::{collections::HashMap, fs, process::Command, sync::{mpsc::{self}, Mutex}, thread::{self}, time::{self, Duration, Instant}};
use actix_web::{get, post, web::{self, Data}, App, HttpResponse, HttpServer, Responder};
use actix_files::Files;
use actix_cors::Cors;
use argon2::password_hash::{Salt, SaltString, rand_core::OsRng};
use chrono::{Local};
use rand::distr::SampleString;
use serde::{Serialize, Deserialize};

use crate::routes::auth::{get_check_token, create_account};
pub mod movement_detector;
pub mod routes;

const HOSTNAME: &str = "nephtys.local";

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    port: u16,
    camera_path: String,
    username: String,
    pass_hash: String,
    salt: String
}

struct AppState {
    config: Config,
    tokens: Mutex<HashMap<String, time::Instant>>
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
            println!("{:?} movement detected",  now.to_rfc3339());
        }
    }
    });

    println!("starting web server");
    env_logger::init();
    HttpServer::new(move || {
        let auth_protected_scope = web::scope("/protected")
        .service(Files::new("/stream", "./static/stream").show_files_listing());
    
        App::new()
        .app_data(Data::new(Mutex::new(AppState {config: config.clone(), tokens: Mutex::new(HashMap::<String, Instant>::new())})))
            .service(create_account)
            .service(get_check_token)
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
    HttpResponse::Ok().body("Hello world!")
}

fn load_config() -> Config {
    let salt = SaltString::generate(&mut OsRng);
    let mut config = Config {
        camera_path: "/dev/video0".to_string(),
        port: 8080,
        username: "".to_string(),
        pass_hash: "".to_string(),
        salt: salt.to_string()
    };
    match fs::read_to_string("./config.toml") {
        Ok(s) => {
            match toml::from_str(s.as_str()) {
                Ok(conf) => config = conf,
                Err(_) => panic!("Couldn't parse config.toml please check the file.")
            }
        },
        Err(_) => {
            fs::write("./config.toml", 
            toml::to_string_pretty(&config)
                        .expect("Couldn't parse default configuration")
            ).expect("Missing config.toml & Couldn't write the default config to it. Check permissions.")
        }
    }

    return config;
}

pub enum WriteConfigError {
    FileSystemError,
    ParsingError
}

pub fn write_config(config: &Config) -> Result<(), WriteConfigError> {
    let parsed = toml::to_string_pretty(&config);
    match parsed {
        Ok(parsed) => {
            match fs::write("./config.toml", parsed) {
                Ok(_) => Ok(()),
                Err(_) => Err(WriteConfigError::FileSystemError)
            }
        }, Err(_) => {
            return Err(WriteConfigError::ParsingError)
        }
    }

}

fn start_ffmpeg_webcam_streaming(input: String) {
    thread::spawn(move || {
        Command::new("ffmpeg")
            .args(["-f", "v4l2", 
                    "-input_format", "mjpeg", 
                    "-video_size", "1280x720", 
                    // "-framerate", "30",
                    "-vsync", "0",
                    "-i", input.as_str(),
                    "-c:v", "libx264", 
                    "-preset", "ultrafast" ,
                    "-tune", "zerolatency" ,
                    "-f", "hls", 
                    "-hls_flags", "delete_segments+independent_segments+split_by_time",
                    "-hls_segment_type", "fmp4",
                    "-hls_time", "3", 
                    "static/stream/stream.m3u8"]
                )
        .output()
        .expect("FATAL: Couldn't start FFMPEG");
        });

}