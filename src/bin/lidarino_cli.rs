#![allow(dead_code, unused_imports)]
use std::ops::Range;
use lidarino::hardware::{YAW_CONTROLLER, PITCH_CONTROLLER, DISTANCE_CONTROLLER};

use warp::Filter;

fn sensors() -> String {
    let (dist, qual) = DISTANCE_CONTROLLER.get_measurement();
    format!("{{{dist}, {qual}}}")
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
    let (distance, quality) = DISTANCE_CONTROLLER.get_measurement();

    (pitch_control, yaw_control, distance, quality)
}

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
}

fn main() {
    println!("WELCOME TO LIDARINO");
    manual_control();
    start_scan();
}
