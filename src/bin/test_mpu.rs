use lidarino::mpu::*;
use linux_embedded_hal::{Delay, I2cdev};
use mpu9250::*;
use std::thread;
use std::time::Duration;

const I2C_ADDR: &str = "/dev/i2c-1";

pub fn test_madwick_mpu() {
    let mut mpu = Mpu::new(Duration::from_millis(10));
    println!("Calibration started.");
    mpu.calibrate();
    println!("Calibration finished.");
    println!();

    loop {
        let quat = mpu.update();
        let (roll, pitch, yaw) = quat.euler_angles();
        print!("\rpitch={pitch:>6.2}, roll={roll:>6.2}, yaw={yaw:>6.2}");
    }
}

pub fn test_raw_mpu() {
    let i2c = I2cdev::new(I2C_ADDR).unwrap();
    let mut mpu9250 = Mpu9250::marg_default(i2c, &mut Delay).expect("unable to make MPU9250");

    let who_am_i = mpu9250.who_am_i().expect("could not read WHO_AM_I");
    let mag_who_am_i = mpu9250
        .ak8963_who_am_i()
        .expect("could not read magnetometer's WHO_AM_I");

    println!("WHO_AM_I: 0x{who_am_i:x}");
    println!("AK8963 WHO_AM_I: 0x{mag_who_am_i:x}");
    assert_eq!(who_am_i, 0x71);

    println!("   Accel XYZ(m/s^2)  |   Gyro XYZ (rad/s)  |  Mag Field XYZ(uT)  | Temp (C)");
    loop {
        let all: MargMeasurements<[f32; 3]> = mpu9250.all().expect("unable to read from MPU!");
        print!("\r{:>6.2} {:>6.2} {:>6.2} |{:>6.1} {:>6.1} {:>6.1} |{:>6.1} {:>6.1} {:>6.1} | {:>4.1} ",
               all.accel[0],
               all.accel[1],
               all.accel[2],
               all.gyro[0],
               all.gyro[1],
               all.gyro[2],
               all.mag[0],
               all.mag[1],
               all.mag[2],
               all.temp);
        thread::sleep(Duration::from_millis(100));
    }
}

fn main() {
    test_madwick_mpu();
    test_raw_mpu();
}
