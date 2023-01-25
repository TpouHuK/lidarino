#![allow(dead_code, unused_imports)]
use std::ops::Range;

use lidarino::distance::*;
use lidarino::motor::*;
use lidarino::mpu::*;
use lidarino::mcp23s17::*;
use warp::Filter;

fn sensors() -> String {
    let mut sensor = DistanceSensor::new();

    let data = sensor.read_distance();
    if let Ok((dist, qual)) = data {
        format!("{{{dist}, {qual}}}")
    } else {
        "Error".to_string()
    }
}

fn manual_control(
    pitch_control: &StepMotorController,
    yaw_control: &StepMotorController,
    //distance_sensor: &mut DistanceSensor,
) {
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
                yaw_control.set_pos(angle);
            }
            ["pitch" | "p", angle] => {
                let angle: i32 = angle.parse().unwrap();
                println!("setting pitch to {angle}");
                pitch_control.set_pos(angle);
            }
            ["stop" | "s"] => {
                println!("stopping motors");
                yaw_control.stop();
                pitch_control.stop();
            }
            ["exit"] => {
                println!("bye!");
                break;
            }
            ["measure" | "m"] => {
                let measurement = make_measurement(pitch_control, yaw_control);
                println!("measurement: {:?}", measurement);
            }
            _ => {
                println!("{:?}, {}", split, user_input)
            }
        }
        user_input.clear();
    }
}

// Yaw, pitch, distance
fn make_measurement(
    pitch_control: &StepMotorController,
    yaw_control: &StepMotorController,
    //distance_sensor: &mut DistanceSensor,
) -> (i32, i32, i32, i32) {
    use std::sync::atomic::Ordering;
    let pitch_control: i32 = pitch_control.tgt_pos.load(Ordering::Relaxed);
    let yaw_control: i32 = yaw_control.tgt_pos.load(Ordering::Relaxed);

    //let mut distance = distance_sensor.read_distance_fast();
    //let mut distance = distance_sensor.read_distance();

    let mut retry_count = 3;
    /* 
    while distance.is_err() {
        retry_count -= 1;
        if retry_count == 0 {
            return (pitch_control, yaw_control, 0, -1);
        }
        //distance = distance_sensor.read_distance();
        println!("error reading distance");
    } 
    let data = distance.unwrap();
    let distance = data.0 as i32;
    let quality = data.1 as i32;
    */
    //(pitch_control, yaw_control, distance, quality)
    (0, 0, 0, 0)
}

const YAW_RANGE: Range<i32> = -2000..2000;
const PITCH_RANGE: Range<i32> = -1800..1000;

fn start_scan(
    pitch_control: &StepMotorController,
    yaw_control: &StepMotorController,
    //distance_sensor: &mut DistanceSensor,
) {
    for yaw in YAW_RANGE.step_by(20) {
        yaw_control.set_pos(yaw);
        yaw_control.wait_stop();
        for pitch in PITCH_RANGE.step_by(20) {
            pitch_control.set_pos(pitch);
            pitch_control.wait_stop();

            let m = make_measurement(pitch_control, yaw_control);
            println!(
                "{{ \"pitch\": {}, \"yaw\": {}, \"distance_mm\": {}, \"quality\": {}}},",
                m.0, m.1, m.2, m.3
            );
        }
    }
}

fn main() {
    println!("WELCOME TO LIDARINO");

    let mcp23s17_controller = Mcp23s17Controller::new();
    let pitch_pins: [VirtualPin; 4] = core::array::from_fn(|i| mcp23s17_controller.get_pin(i as u8));
    let yaw_pins: [VirtualPin; 4] = core::array::from_fn(|i| mcp23s17_controller.get_pin(4+i as u8));

    let pitch_motor = StepMotor::new(pitch_pins);
    let yaw_motor = StepMotor::new(yaw_pins);

    let pitch_control = StepMotorController::new(pitch_motor, 3);
    let yaw_control = StepMotorController::new(yaw_motor, 3);
    //let mut distance_sensor = DistanceSensor::new();
    //distance_sensor.start().unwrap();
    manual_control(&pitch_control, &yaw_control);
    start_scan(&pitch_control, &yaw_control);
}
