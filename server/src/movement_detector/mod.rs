use chrono::Local;
use crossbeam_channel::{Receiver, Sender, unbounded};
use opencv::{
    core::{BORDER_CONSTANT, BORDER_DEFAULT, Point, Point_, Size_, VecN, Vector, no_array},
    imgproc::{
        self, ADAPTIVE_THRESH_GAUSSIAN_C, CHAIN_APPROX_TC89_L1, COLOR_BGR2GRAY, INTER_LINEAR,
        MORPH_CLOSE, RETR_EXTERNAL, THRESH_BINARY_INV, bounding_rect,
    },
    prelude::*,
    videoio,
};
use rand::distr::SampleString;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self},
    io::{self},
    process::{Command, Stdio},
    thread,
    time::{self, Duration},
};

pub fn start_movement_detect_thread(mov_detect_tx: Sender<bool>) {
    thread::spawn(move || {
        // hardcoding a delay is bad
        // TODO: Detect when enough .m4s have been added to the stream folder and start once that is reached.
        thread::sleep(time::Duration::from_millis(5000));
        println!("movement detection thread starting...");
        let mut cam =
            videoio::VideoCapture::from_file("./static/stream/stream.m3u8", videoio::CAP_ANY)
                .unwrap();
        let mut frame = Mat::default(); // This array will store the web-cam data
        let mut prev_frame = Mat::default();
        let mut is_first_frame = true;

        let mut detection_count = 0;
        let mut frame_count = 0;
        // Read the camera
        // and display in the window
        loop {
            cam.read(&mut frame).unwrap();
            match frame.size() {
                Ok(_) => {}
                Err(_) => {
                    continue;
                }
            }
            let mut rescaled: Mat = Mat::default();
            match imgproc::resize(
                &frame,
                &mut rescaled,
                Size_ {
                    width: 640,
                    height: 360,
                },
                0.5,
                0.5,
                INTER_LINEAR,
            ) {
                Ok(_) => {}
                Err(_) => continue,
            }

            let mut first_pass: Mat = Mat::default();
            imgproc::cvt_color(&rescaled, &mut first_pass, COLOR_BGR2GRAY, 0).unwrap();
            let mut final_frame: Mat = Mat::default();
            imgproc::blur(
                &first_pass,
                &mut final_frame,
                Size_ {
                    width: 10,
                    height: 10,
                },
                Point_ { x: -1, y: -1 },
                BORDER_DEFAULT,
            )
            .unwrap();

            frame_count += 1;

            if is_first_frame {
                is_first_frame = false;
                prev_frame = final_frame;
                continue;
            }

            let mut diff_frame = Mat::default();
            opencv::core::subtract(&final_frame, &prev_frame, &mut diff_frame, &no_array(), -1)
                .unwrap();
            let mut mask_frame = Mat::default();
            imgproc::adaptive_threshold(
                &diff_frame,
                &mut mask_frame,
                255.0,
                ADAPTIVE_THRESH_GAUSSIAN_C,
                THRESH_BINARY_INV,
                11,
                3.0,
            )
            .unwrap();
            let kernel = imgproc::get_structuring_element(
                imgproc::MORPH_RECT,
                Size_ {
                    width: 5,
                    height: 5,
                },
                Point { x: -1, y: -1 },
            )
            .unwrap();
            let mut mask_frame_2 = Mat::default();
            imgproc::morphology_ex(
                &mask_frame,
                &mut mask_frame_2,
                MORPH_CLOSE,
                &kernel,
                Point { x: -1, y: -1 },
                2,
                BORDER_CONSTANT,
                VecN::new(0.0, 0.0, 0.0, 0.0),
            )
            .unwrap();
            let mut contours: Vector<Vector<Point_<i32>>> = Vector::new();
            imgproc::find_contours(
                &mask_frame_2,
                &mut contours,
                RETR_EXTERNAL,
                CHAIN_APPROX_TC89_L1,
                Point_ { x: 0, y: 0 },
            )
            .unwrap();

            for countour in contours {
                let bb = bounding_rect(&countour).unwrap();
                if bb.area() > 250 {
                    detection_count += 1;
                    if detection_count > 10 {
                        mov_detect_tx.send(true).unwrap();
                        detection_count = 0;
                    }
                }
            }

            if frame_count >= 30 {
                frame_count = 0;
                detection_count = 0;
            }

            prev_frame = final_frame;
        }
    });

    // handle.join().unwrap();
    println!("mov thread start");
}

#[derive(Serialize, Deserialize, Clone)]
struct MovementEvent {
    start: String,
    end: String,
    filename: String,
}

#[derive(Serialize, Deserialize)]
struct MovementEventLogs {
    events: Vec<MovementEvent>,
}

fn write_movements_logs(records: Vec<MovementEvent>) {
    thread::spawn(move || {
        let records_list = MovementEventLogs { events: records };
        let contents = serde_json::to_string(&records_list);
        match contents {
            Ok(raw_json) => match fs::write("./static/clips/index.json", raw_json) {
                Ok(_) => {
                    println!("updated clips index")
                }
                Err(_) => {
                    println!("ERROR: Failed to update clips index")
                }
            },
            Err(_) => {
                println!("ERROR: Unexpected parsing error when updating clips index")
            }
        }
    });
}

pub fn start_movement_logger(mov_detect_rx: Receiver<bool>) {
    match fs::create_dir_all("./static/clips") {
        Ok(_) => println!("Warning: (re)created ./static/clips"),
        Err(_) => {
            fs::exists("./static/clips")
                .expect("FATAL: Couldn't create ./static/clips please check permissions");
        }
    }
    thread::spawn(move || {
        let mut records: Vec<MovementEvent> = vec![];
        let mut last_record_start = Local::now();
        let mut in_event = false;
        let (move_end_tx, move_end_rx) = unbounded();
        let mut filename = generate_name();
        loop {
            match mov_detect_rx.recv_timeout(Duration::from_secs(5)) {
                Ok(_) => {
                    let now = Local::now();
                    println!("movement detected at {}", now.to_rfc3339());
                    if in_event {
                        continue;
                    }
                    in_event = true;
                    last_record_start = now;
                    start_recording_clip(move_end_rx.clone(), filename.clone());
                }
                Err(_) => {
                    if !in_event {
                        continue;
                    }
                    println!("5s without movement... stopping recroding");
                    in_event = false;
                    let now = Local::now();
                    let _ = move_end_tx.send(()); // We end the record there
                    records.push(MovementEvent {
                        start: last_record_start.to_rfc3339(),
                        end: now.to_rfc3339(),
                        filename: filename.clone(),
                    });
                    filename = generate_name();
                    write_movements_logs(records.clone());
                }
            }
        }
    });
}

fn start_recording_clip(stop_signal: Receiver<()>, filename: String) {
    thread::spawn(move || {
        fs::create_dir(format!("./static/clips/{}/", filename)).expect("Couldn't record clip");
        println!("Recording started");
        loop {
            match stop_signal.recv_timeout(Duration::from_millis(1000)) {
                Ok(_) => {
                    println!("Recording stopped");

                    generate_mp4_from_chunks(filename);
                    return;
                }
                Err(_) => {
                    let recording_stream = format!("./static/clips/{}", filename);
                    match fs::read_dir("./static/stream") {
                        Ok(stream_files) => {
                            for file in stream_files {
                                match file {
                                    Ok(file_entry) => {
                                        let filename =
                                            file_entry.file_name().into_string().unwrap();
                                        fs::copy(
                                            file_entry.path(),
                                            format!("{}/{}", &recording_stream, &filename),
                                        )
                                        .expect("CANNOT COPY FILE");
                                    }
                                    Err(_) => {
                                        println!("Skipping file in stream")
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            println!("ERROR: Couldn't read ./static/stream")
                        }
                    }
                }
            }
        }
    });
}

fn generate_name() -> String {
    rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 32)
}

fn concat_mp4_fragments(filename: String) {
    let mut output_file =
        fs::File::create_new(format!("./static/clips/{}/concat.m4s", filename)).unwrap();
    let mut init_file = fs::File::open(format!("./static/clips/{}/init.mp4", filename)).unwrap();
    io::copy(&mut init_file, &mut output_file).unwrap();
    match fs::read_dir(format!("./static/clips/{}", filename)) {
        Ok(stream_files) => {
            let mut stream_files_sorted = stream_files
                .map(|res| res.map(|e| e.file_name()))
                .into_iter()
                .collect::<Result<Vec<_>, io::Error>>()
                .unwrap();
            stream_files_sorted.sort();
            for file_path_ostr in stream_files_sorted {
                let file_name_str = &file_path_ostr.into_string().unwrap();
                let file_path_str = format!("./static/clips/{}/{}", filename, file_name_str);
                let file_path = std::path::Path::new(&file_path_str);

                match fs::File::open(format!("./static/clips/{}/{}", filename, file_name_str)) {
                    Ok(mut file_entry) => {
                        let filename = file_path.file_name().unwrap().to_str().unwrap();
                        if filename.ends_with(".m4s") && filename != "concat.m4s" {
                            io::copy(&mut file_entry, &mut output_file).unwrap();
                        }
                    }
                    Err(err) => {
                        println!("OPENFILERROR: {}", err);
                        println!("Skipping file {} in stream", file_name_str)
                    }
                }
            }
        }
        Err(_) => {
            println!("ERROR: Couldn't read ./static/stream")
        }
    }
}

fn generate_mp4_from_chunks(filename: String) {
    thread::spawn(move || {
        concat_mp4_fragments(filename.clone());
        let _ffmpeg_proc: Result<std::process::Child, std::io::Error> = Command::new("ffmpeg")
            .args([
                "-v",
                "0",
                "-i",
                format!("./static/clips/{}/concat.m4s", filename).as_str(),
                "-c:v",
                "copy",
                format!("./static/clips/{}.mkv", filename).as_str(),
            ])
            .stdout(Stdio::piped())
            .spawn();
    });
}
