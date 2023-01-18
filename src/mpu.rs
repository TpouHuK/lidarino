extern crate mpu6050;
use std::thread;
use std::time::Duration;
use linux_embedded_hal::{I2cdev, Delay};
use i2cdev::linux::LinuxI2CError;
use mpu6050::*;

pub fn test_mpu() {
    let i2c = I2cdev::new("/dev/i2c-1").unwrap();
    let mut delay = Delay;
    let mut mpu = Mpu6050::new(i2c);
    mpu.init(&mut delay);

loop {
    thread::sleep(Duration::from_secs_f32(1f32));
    // get roll and pitch estimate
    let acc = mpu.get_acc_angles().unwrap();
    println!("r/p: {:?}", acc);

    // get temp
    let temp = mpu.get_temp().unwrap();
    println!("temp: {:?}c", temp);

    // get gyro data, scaled with sensitivity 
    let gyro = mpu.get_gyro().unwrap();
    println!("gyro: {:?}", gyro);

    // get accelerometer data, scaled with sensitivity
    let acc = mpu.get_acc().unwrap();
    println!("acc: {:?}", acc);
  }
}
