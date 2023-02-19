use lazy_static::lazy_static;
use lidarino::config::{Config, CONFIG_PATH};
use lidarino::hardware::distance::DistanceReading;
use lidarino::hardware::mpu::OrientationController;
use lidarino::hardware::mpu::*;
use lidarino::hardware::{
    DISTANCE_CONTROLLER, MPU_CONTROLLER, ORIENTATION_CONTROLLER, PITCH_CONTROLLER, YAW_CONTROLLER,
};
use lidarino::sphere::*;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use lidarino::scan::*;

lazy_static! {
    static ref CONFIG: Mutex<Config> = Mutex::new(Config::default());
}

lazy_static! {
    static ref SCAN_JOB: ScanJob = ScanJob::new();
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
                let (roll_a, pitch_a, yaw_a) = ORIENTATION_CONTROLLER
                    .lock()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .get_quat()
                    .euler_angles();
                println!("current_yaw: {yaw}, current_pitch: {pitch}, roll: {roll_a}, pitch: {pitch_a}, yaw: {yaw_a}");
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
                let measurement = DISTANCE_CONTROLLER.get_measurement();
                println!("measurement: {measurement:?}");
            }
            ["gen_path"] => {
            /*
             let opts = ScanOptions {
                    amount_of_points: 1000,
                    pitch_start: 0.0,
                    pitch_end: 160.0,
                    yaw_start: 180.0 - 90.0,
                    yaw_end: 180.0 + 90.0,
                }; */

             let opts = ScanOptions {
                    amount_of_points: 1000,
                    pitch_start: 0.0,
                    pitch_end: 160.0,
                    yaw_start: 180.0 - 90.0,
                    yaw_end: 180.0 + 90.0,
                };
                SCAN_JOB.generate_path(opts)
            }
            ["start_scan"] => {
                SCAN_JOB.start_scan()
            }
            ["pause_scan"] => {
                SCAN_JOB.pause_scan()
            }
            ["save_scan"] => {
                SCAN_JOB.save_file()
            }
            ["reset" | "r"] => {
                println!("Yaw and Pitch set as 0.");
                YAW_CONTROLLER.reset();
                PITCH_CONTROLLER.reset();
            }
            ["calibrate", "gyro"] | ["cg"] => {
                println!("Gyroscope calibration started. Keep MPU still.");
                let mut mpu = MPU_CONTROLLER.lock().unwrap();
                let gyro_bias =
                    lidarino::hardware::mpu::calculate_gyro_bias(&mut mpu, &Duration::from_secs(3));
                drop(mpu);
                let mut config = CONFIG.lock().unwrap();
                match &mut config.mpu_config {
                    None => {
                        panic!("wtf is this config");
                    }
                    Some(mpu_config) => mpu_config.gyro_bias = gyro_bias,
                }
                match config.save_to_file(CONFIG_PATH) {
                    Ok(_) => {
                        println!("Saved config to file.");
                    }
                    Err(e) => {
                        println!("Error writing a config: {e:?}");
                    }
                }
            }
            ["calibrate", "accel"] | ["ca"] => {
                println!("Accelerometer calibration started.");
                let mut mpu = MPU_CONTROLLER.lock().unwrap();
                let (accel_bias, accel_scale) =
                    lidarino::hardware::mpu::calculate_accel_bias_and_scale(&mut mpu);
                drop(mpu);
                let mut config = CONFIG.lock().unwrap();
                match &mut config.mpu_config {
                    None => {
                        panic!("wtf is this config");
                    }
                    Some(mpu_config) => {
                        mpu_config.accel_bias = accel_bias;
                        mpu_config.accel_scale = accel_scale;
                    }
                }
                match config.save_to_file(CONFIG_PATH) {
                    Ok(_) => {
                        println!("Saved config to file.");
                    }
                    Err(e) => {
                        println!("Error writing a config: {e:?}");
                    }
                }
            }
            ["magdump"] => {
                let mut mpu = MPU_CONTROLLER.lock().unwrap();
                let data = lidarino::hardware::mpu::get_magnetometer_data(
                    &mut mpu,
                    &Duration::from_secs(60),
                );
                let mut dump = String::new();
                for d in data {
                    dump.push_str(&format!("{} {} {}\n", d[0], d[1], d[2]));
                }
                std::fs::write("magnetometer_dump", dump).unwrap();
            }
            ["init_orientation"] => {
                let mut orientation_controller = ORIENTATION_CONTROLLER.lock().unwrap();
                if orientation_controller.is_none() {
                    let mpu_config = CONFIG.lock().unwrap().mpu_config.unwrap();
                    let mpu = Mpu::new(mpu_config);
                    let new_c = OrientationController::new(mpu);
                    *orientation_controller = Some(new_c);
                    println!("Done initialization, pls dont access MPU using other means. FIXME");
                } else {
                    println!("Error, orientation controller allready initialized");
                }
            }
            _ => {
                println!("{split:?}, {user_input}")
            }
        }
        user_input.clear();
    }
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
    roll: f32,
    pitch: f32,
    yaw: f32,
    distance: u32,
    quality: u32,
}

fn main() {
    println!("WELCOME TO LIDARINO");
    if CONFIG.lock().unwrap().load_from_file(CONFIG_PATH).is_ok() {
        println!("Succesfully loaded config from \"{CONFIG_PATH}\"");
    } else {
        println!("Failed loading config from \"{CONFIG_PATH}\"");
    };

    //lazy_static::initialize(&ORIENTATION_CONTROLLER);
    manual_control();
}
