use std::time;
use std::time::{Duration};
use std::thread;
use rppal::gpio::{Gpio, OutputPin};

mod motor;
use motor::*;


fn wait_enter() {
    use std::io::{stdin,stdout,Write};
    let mut _s = String::new();
    stdin().read_line(&mut _s).expect("Did not ented a string? wtf");
    println!("Bye!...");
}

fn main() {
    let mut pitch_motor = StepMotor::new([4, 17, 27, 22]);
    let mut yaw_motor = StepMotor::new([10, 9, 11, 5]);
}
