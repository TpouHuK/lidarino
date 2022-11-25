use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicU32, AtomicI32, AtomicBool, Ordering};
use std::sync::{Arc};

mod motor;
use motor::*;

fn wait_enter() {
    use std::io::{stdin,stdout,Write};
    let mut _s = String::new();
    stdin().read_line(&mut _s).expect("Did not ented a string? wtf");
    println!("Bye!...");
}

fn test_motors() {
    let mut pitch_motor = StepMotor::new([4, 17, 27, 22]);
    let mut yaw_motor = StepMotor::new([10, 9, 11, 5]);

    let pitch_control = StepMotorController::new(pitch_motor, 3);
    let yaw_control = StepMotorController::new(yaw_motor, 3);

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

fn main() {
    test_motors();
}
