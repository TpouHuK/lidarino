mod motor;
use motor::*;

#[allow(dead_code)]
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

fn main() {
    test_motors();
}
