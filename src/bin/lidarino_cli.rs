#![allow(dead_code, unused_imports)]
use lidarino::hardware::distance::DistanceReading;
use lidarino::hardware::{DISTANCE_CONTROLLER, PITCH_CONTROLLER, YAW_CONTROLLER};
use lidarino::sphere::*;
use spinners::{Spinner, Spinners};
use std::ops::Range;
use std::time::{Duration, Instant};

use warp::Filter;

fn sensors() -> String {
    match DISTANCE_CONTROLLER.get_measurement() {
        DistanceReading::Ok {
            distance, quality, ..
        } => {
            format!("{{{}, {quality}}}", distance.as_mm())
        }
        _ => "some error idk".to_string(),
    }
}

fn manual_control() {
    use std::io;
    use std::io::Write;
    let stdin = io::stdin();
    let mut user_input = String::with_capacity(100);

    loop {
        print!("[LIDARINO_manual]-> ");
        io::stdout().flush().unwrap();

        stdin.read_line(&mut user_input).unwrap();
        let split: Vec<&str> = user_input.trim().split(' ').collect();
        match split[..] {
            ["state" | "t"] => {
                let yaw = YAW_CONTROLLER.get_current_pos();
                let pitch = PITCH_CONTROLLER.get_current_pos();
                println!("current_yaw: {yaw}, current_pitch: {pitch}");
            }
            ["yaw" | "y", angle] => {
                let angle: i32 = angle.parse().unwrap();
                println!("setting yaw to {angle}");
                YAW_CONTROLLER.set_target_pos(angle);
            }
            ["pitch" | "p", angle] => {
                let angle: i32 = angle.parse().unwrap();
                println!("setting pitch to {angle}");
                PITCH_CONTROLLER.set_target_pos(angle);
            }
            ["stop" | "s"] => {
                println!("stopping motors");
                YAW_CONTROLLER.stop();
                PITCH_CONTROLLER.stop();
            }
            ["exit"] => {
                println!("bye!");
                break;
            }
            ["measure" | "m"] => {
                let measurement = make_measurement();
                println!("measurement: {measurement:?}");
            }
            ["scan"] => {
                do_scan();
            }
            ["reset" | "r"] => {
                println!("Yaw and Pitch set as 0.");
                YAW_CONTROLLER.reset();
                PITCH_CONTROLLER.reset();
            }
            _ => {
                println!("{split:?}, {user_input}")
            }
        }
        user_input.clear();
    }
}

// Yaw, pitch, distance
fn make_measurement() -> (i32, i32, u32, u32) {
    use std::sync::atomic::Ordering;
    let pitch_control: i32 = PITCH_CONTROLLER.get_target_pos();
    let yaw_control: i32 = YAW_CONTROLLER.get_target_pos();
    let (distance, quality) = match DISTANCE_CONTROLLER.get_measurement() {
        DistanceReading::Ok {
            distance, quality, ..
        } => (distance.as_mm(), quality),
        _ => (0, 0),
    };

    (pitch_control, yaw_control, distance, quality as u32)
}

fn do_scan() {
    println!("Scanning started");

    let path_start = Instant::now();
    let mut sp = Spinner::new(Spinners::Dots9, "Building a path.".into());

    let opts = ScanOptions {
        amount_of_points: 10,
        pitch_start: 0.0,
        pitch_end: 120.0,
        yaw_start: 180.0 - 90.0,
        yaw_end: 180.0 + 90.0,
    };
    let points = lidarino::sphere::generate_points(opts);
    let waypoints: Vec<Waypoint> = points.into_iter().map(|p| p.into()).collect();
    let waypoints = optimize_path(waypoints, Duration::from_secs(1));
    sp.stop_and_persist(
        "✔",
        format!("Done path building in {:?}", path_start.elapsed()),
    );

    let mut scanned_points = Vec::new();
    let scan_start = Instant::now();
    for (i, waypoint) in waypoints.iter().enumerate() {
        let elapsed = scan_start.elapsed();
        let msg = format!("Going to point #{i}. ELAPSED: {elapsed:?}");
        sp = Spinner::new(Spinners::Line, msg);

        YAW_CONTROLLER.set_target_pos(waypoint.yaw);
        YAW_CONTROLLER.wait_stop();
        PITCH_CONTROLLER.set_target_pos(waypoint.pitch);
        PITCH_CONTROLLER.wait_stop();

        let measurement = DISTANCE_CONTROLLER.get_measurement();
        match measurement {
            DistanceReading::Ok {
                distance, quality, ..
            } => {
                sp.stop_and_persist(
                    "✔",
                    format!(
                        "Done #{i}. Distance: {}, Quality: {}",
                        distance.as_mm(),
                        quality
                    ),
                );
                let point =
                    Point::from_yaw_pitch_distance(waypoint.yaw, waypoint.pitch, distance.as_mm());
                scanned_points.push(point);
            }
            DistanceReading::Err { .. } => {
                sp.stop_and_persist("❌", format!("Failed #{i}."));
            }
            DistanceReading::NoReading => {
                unreachable!()
            }
        }
    }
    println!("Scanning finished.");

    let json_string = serde_json::to_string(&scanned_points).unwrap();
    std::fs::write("points.json", json_string).unwrap();
}

/*
const YAW_RANGE: Range<i32> = -2000..2000;
const PITCH_RANGE: Range<i32> = -1800..1000;
fn start_scan() {
    for yaw in YAW_RANGE.step_by(20) {
        YAW_CONTROLLER.set_target_pos(yaw);
        YAW_CONTROLLER.wait_stop();
        for pitch in PITCH_RANGE.step_by(20) {
            PITCH_CONTROLLER.set_target_pos(pitch);
            PITCH_CONTROLLER.wait_stop();

            let m = make_measurement();
            println!(
                "{{ \"pitch\": {}, \"yaw\": {}, \"distance_mm\": {}, \"quality\": {}}},",
                m.0, m.1, m.2, m.3
            );
        }
    }
} */

fn main() {
    println!("WELCOME TO LIDARINO");
    manual_control();
}
