#![allow(dead_code, unused_imports)]
mod motor;
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

#[tokio::main]
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
}
