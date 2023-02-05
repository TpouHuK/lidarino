#![allow(dead_code, unused_imports)]
use lidarino::hardware::distance::DistanceReading;
use lidarino::hardware::{DISTANCE_CONTROLLER, PITCH_CONTROLLER, YAW_CONTROLLER};
use lidarino::sphere::*;
use serde::{Deserialize, Serialize};
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
                println!("Yaw and Pitch set as 0.");
                YAW_CONTROLLER.reset();
                PITCH_CONTROLLER.reset();

                do_scan();
                println!("Going to 0, 0");
                PITCH_CONTROLLER.set_target_pos(0);
                PITCH_CONTROLLER.wait_stop();
                YAW_CONTROLLER.set_target_pos(0);
                YAW_CONTROLLER.wait_stop();
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

#[derive(Serialize, Deserialize)]
struct ScannedCheckpoint {
    x: f32,
    y: f32,
    z: f32,
    waypoint_yaw: i32,
    waypoint_pitch: i32,
    current_yaw: i32,
    current_pitch: i32,
    distance: u32,
    quality: u32,
}

fn do_scan() {
    println!("Scanning started");

    let path_start = Instant::now();
    let mut sp = Spinner::new(Spinners::Dots9, "Building a path.".into());

    let opts = ScanOptions {
        amount_of_points: 100,
        pitch_start: 0.0,
        pitch_end: 120.0,
        yaw_start: 180.0 - 90.0,
        yaw_end: 180.0 + 90.0,
    };
    let points = lidarino::sphere::generate_points(opts);
    let waypoints: Vec<Waypoint> = points.into_iter().map(|p| p.into()).collect();
    let waypoints = optimize_path(waypoints, Duration::from_secs(10));
    sp.stop_and_persist(
        "✔",
        format!("Done path building in {:?}", path_start.elapsed()),
    );

    let mut scanned_points = Vec::new();
    let scan_start = Instant::now();

    let starting_point = Waypoint {
        yaw: YAW_CONTROLLER.get_current_pos(),
        pitch: PITCH_CONTROLLER.get_current_pos(),
    };
    let mut estimated_distance_to_travel: u32 = {
        waypoints
            .iter()
            .fold((0u32, Some(starting_point)), |(acc, prev), e| {
                if let Some(prev) = prev {
                    (
                        acc + ((prev.yaw - e.yaw).abs() + (prev.pitch - e.pitch).abs()) as u32,
                        Some(*e),
                    )
                } else {
                    (acc, Some(*e))
                }
            })
            .0
    };
    //eprintln!("starting estimated_distance_to_travel: {estimated_distance_to_travel}");

    let mut measurements = Vec::new();

    for (i, waypoint) in waypoints.iter().enumerate() {
        let elapsed = scan_start.elapsed();
        let average_measuring_duration: Duration = {
            if !measurements.is_empty() {
                measurements.iter().sum::<Duration>() / measurements.len() as u32
            } else {
                Duration::from_millis(300)
            }
        };

        let estimated_traveling_time = estimated_distance_to_travel
            * Duration::from_millis(YAW_CONTROLLER.get_step_delay_ms() as u64);
        let estimated_measuring_time = average_measuring_duration * (waypoints.len() - i) as u32;
        let estimated_time = estimated_measuring_time + estimated_traveling_time;

        let msg =
            format!("Going to point #{i}. Elapsed: {elapsed:?} Time left: {estimated_time:?}");
        sp = Spinner::new(Spinners::Line, msg);

        let gonna_travel = ((YAW_CONTROLLER.get_current_pos() - waypoint.yaw).abs()
            + (PITCH_CONTROLLER.get_current_pos() - waypoint.pitch).abs())
            as u32;
        estimated_distance_to_travel -= gonna_travel;
        //eprintln!("gonna_travel: {gonna_travel}");
        //eprintln!("estimated_distance_to_travel: {estimated_distance_to_travel}");

        YAW_CONTROLLER.set_target_pos(waypoint.yaw);
        YAW_CONTROLLER.wait_stop();
        PITCH_CONTROLLER.set_target_pos(waypoint.pitch);
        PITCH_CONTROLLER.wait_stop();

        let measurement = DISTANCE_CONTROLLER.get_measurement();

        match measurement {
            DistanceReading::Ok {
                distance,
                quality,
                measuring_time,
            } => {
                measurements.push(measuring_time);
                sp.stop_and_persist(
                    "✔",
                    format!(
                        "Done #{i}. Distance: {}, Quality: {}",
                        distance.as_mm(),
                        quality
                    ),
                );
                let p =
                    Point::from_yaw_pitch_distance(waypoint.yaw, waypoint.pitch, distance.as_mm());
                let scanned_checkpoint = ScannedCheckpoint {
                    x: p.x,
                    y: p.y,
                    z: p.z,
                    waypoint_yaw: waypoint.yaw,
                    waypoint_pitch: waypoint.pitch,
                    current_yaw: YAW_CONTROLLER.get_current_pos(),
                    current_pitch: PITCH_CONTROLLER.get_current_pos(),
                    distance: distance.as_mm(),
                    quality: quality as u32,
                };
                scanned_points.push(scanned_checkpoint);
            }
            DistanceReading::Err { measuring_time, .. } => {
                measurements.push(measuring_time);
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
