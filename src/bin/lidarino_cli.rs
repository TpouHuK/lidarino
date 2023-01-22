#![allow(dead_code, unused_imports)]
mod motor;
use std::ops::Range;

use motor::*;

mod mpu;
use mpu::*;

mod distance;
use distance::*;

fn wait_enter() {
    use std::io::stdin;
    let mut _s = String::new();
    stdin().read_line(&mut _s).expect("Did not ented a string? wtf");
    println!("Bye!...");
}

fn test_motors() {
    let pitch_motor = StepMotor::new([4, 17, 27, 22]);
    let yaw_motor = StepMotor::new([10, 9, 11, 5]);

    let pitch_control = StepMotorController::new(pitch_motor, 3);
    let yaw_control = StepMotorController::new(yaw_motor, 3);
    yaw_control.stop();

    loop {
       pitch_control.set_pos(-300);
        yaw_control.set_pos(-400);

        pitch_control.wait_stop();
        yaw_control.wait_stop();

        pitch_control.set_pos(400);
        yaw_control.set_pos(300);

        pitch_control.wait_stop();
        yaw_control.wait_stop();
    }
}

use warp::Filter;

fn sensors() -> String {
    let mut sensor = DistanceSensor::new();

    let data = sensor.read_distance();
    if let Ok((dist, qual)) = data {
        format!("{{{}, {}}}", dist, qual)
    } else {
        "Error".to_string()
    }
}


fn manual_control(pitch_control: &StepMotorController, yaw_control: &StepMotorController, distance_sensor: &mut DistanceSensor) {
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
            ["exit"] => { println!("bye!"); break }
            ["measure" | "m"] => {
                let measurement = make_measurement(pitch_control, yaw_control, distance_sensor);
                println!("measurement: {:?}", measurement);
                }
            _ => { println!("{:?}, {}", split, user_input)}
        }
        user_input.clear();
    }
}

// Yaw, pitch, distance
fn make_measurement(pitch_control: &StepMotorController, yaw_control: &StepMotorController, distance_sensor: &mut DistanceSensor) -> (i32, i32, i32, i32) {
    use std::sync::atomic::Ordering;
    let pitch_control: i32 = pitch_control.tgt_pos.load(Ordering::Relaxed);
    let yaw_control: i32 = yaw_control.tgt_pos.load(Ordering::Relaxed);

    let mut distance = distance_sensor.read_distance_fast();
    //let mut distance = distance_sensor.read_distance();

    let mut retry_count = 3;
    while distance.is_err() {
        retry_count -= 1;
        if retry_count == 0 { return (pitch_control, yaw_control, 0, -1) }
        distance = distance_sensor.read_distance();
        println!("error reading distance");
    }
    let data = distance.unwrap();
    let distance = data.0 as i32;
    let quality = data.1 as i32;

    (pitch_control, yaw_control, distance, quality)
}

const YAW_RANGE: Range<i32> = -1000..1000;
const PITCH_RANGE: Range<i32> = -1500..500;

fn start_scan(pitch_control: &StepMotorController, yaw_control: &StepMotorController, distance_sensor: &mut DistanceSensor) {
    for yaw in YAW_RANGE.step_by(100) {
        for pitch in PITCH_RANGE.step_by(100) {
            pitch_control.set_pos(pitch);
            yaw_control.set_pos(yaw);

            pitch_control.wait_stop();
            yaw_control.wait_stop();

            let m = make_measurement(pitch_control, yaw_control, distance_sensor);
            println!("{{ \"pitch\": {}, \"yaw\": {}, \"distance_mm\": {} }},", m.0, m.1, m.2);
        }
    }
}

fn main() {
    println!("WELCOME TO LIDARINO");

    let pitch_motor = StepMotor::new([4, 17, 27, 22]);
    let yaw_motor = StepMotor::new([10, 9, 11, 5]);

    let pitch_control = StepMotorController::new(pitch_motor, 15);
    let yaw_control = StepMotorController::new(yaw_motor, 15);
    let mut distance_sensor = DistanceSensor::new();
    distance_sensor.start().unwrap();
    manual_control(&pitch_control, &yaw_control, &mut distance_sensor);
    start_scan(&pitch_control, &yaw_control, &mut distance_sensor);
}

/* #[tokio::main]
async fn main() {
    let yaw = warp::path!("yaw" / i64).map(| yaw | { format!("yaw {yaw}!") } );
    let pitch = warp::path!("pitch" / i64).map( | pitch | { format!("pitch {pitch}!") } );
    let sensors = warp::path!("sensors").map( sensors );

    println!("we are running!");
    warp::serve(yaw.or(pitch).or(sensors))
        .run(([127, 0, 0, 1], 80))
        .await;
    //test_mpu();
    //test_motors();
} */
