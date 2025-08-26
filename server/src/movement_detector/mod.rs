use std::{sync::mpsc::Sender, thread, time};
use opencv::{
 core::{no_array, Point, Point_, Size_, VecN, Vector, BORDER_CONSTANT, BORDER_DEFAULT}, highgui::{self}, imgproc::{self, bounding_rect, ADAPTIVE_THRESH_GAUSSIAN_C, CHAIN_APPROX_TC89_L1, COLOR_BGR2GRAY, INTER_LINEAR, MORPH_CLOSE, RETR_EXTERNAL, THRESH_BINARY_INV}, prelude::*, sys::cv_utils_logging_setLogLevel_LogLevel, videoio
};

pub fn start_movement_detect_thread(mov_detect_tx: Sender<bool>) {

    thread::spawn(move || {
    // hardcoding a delay is bad
    // TODO: Detect when enough .m4s have been added to the stream folder and start once that is reached.
    thread::sleep(time::Duration::from_millis(10000));
    println!("movement detection thread starting...");
    let mut cam = videoio::VideoCapture::from_file("http://localhost:8080/protected/stream/stream.m3u8", videoio::CAP_ANY).unwrap();
    let mut frame = Mat::default(); // This array will store the web-cam data
    let mut prev_frame = Mat::default();
    let mut is_first_frame = true;

    let mut detection_count = 0;
    let mut frame_count = 0;
    // Read the camera
    // and display in the window
    loop {
        cam.read(&mut frame).unwrap();
        let mut rescaled:Mat = Mat::default();
        imgproc::resize(&frame, &mut rescaled, Size_ { width: 640, height: 360 }, 0.5, 0.5, INTER_LINEAR).unwrap();
        let mut first_pass: Mat = Mat::default();
        imgproc::cvt_color(&rescaled, &mut first_pass, COLOR_BGR2GRAY, 0).unwrap();
        let mut final_frame: Mat = Mat::default();
        imgproc::blur(&first_pass, &mut final_frame, Size_ { width: 10, height: 10 }, Point_ { x: -1, y: -1 }, BORDER_DEFAULT).unwrap();

        
        frame_count+=1;

        if is_first_frame {
            is_first_frame = false;
            prev_frame = final_frame;
            continue;
        }


        let mut diff_frame = Mat::default();
        opencv::core::subtract(&final_frame, &prev_frame, &mut diff_frame, &no_array(), -1).unwrap();
        let mut mask_frame = Mat::default();
        imgproc::adaptive_threshold(&diff_frame, &mut mask_frame, 255.0, ADAPTIVE_THRESH_GAUSSIAN_C, THRESH_BINARY_INV, 11, 3.0).unwrap();
        let kernel = imgproc::get_structuring_element(
            imgproc::MORPH_RECT,
            Size_ { width: 5, height: 5 },
            Point { x: -1, y: -1 },
        ).unwrap();
        let mut mask_frame_2 = Mat::default();
        imgproc::morphology_ex(&mask_frame, &mut mask_frame_2, MORPH_CLOSE, &kernel, Point { x: -1, y: -1 }, 2, BORDER_CONSTANT, VecN::new(0.0, 0.0, 0.0, 0.0)).unwrap();
        let mut contours: Vector<Vector<Point_<i32>>> = Vector::new();
        imgproc::find_contours(&mask_frame_2, &mut contours, RETR_EXTERNAL, CHAIN_APPROX_TC89_L1, Point_ { x: 0, y: 0 }).unwrap();

        for countour in contours {
            let bb = bounding_rect(&countour).unwrap();
            if bb.area() > 250 {
                //TODO: Count potential detection up
                detection_count+=1;
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