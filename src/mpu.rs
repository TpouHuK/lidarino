use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::sync::Arc;

use ahrs::{ Madgwick, Ahrs } ;
use linux_embedded_hal::{I2cdev, Delay};
use mpu9250::*;

const I2C_ADDR: &str = "/dev/i2c-1";

pub struct Mpu {
    mpu9250: Mpu9250<I2cDevice<I2cdev>, mpu9250::Marg>,
    madgwick: Madgwick<f32>,
    sample_period: Duration,
}

use nalgebra::base::Vector3;

impl Mpu {
    pub fn new(sample_period: Duration) -> Self {
        let i2c = I2cdev::new(I2C_ADDR).unwrap();
        let mpu9250 = Mpu9250::marg_default(i2c, &mut Delay).expect("unable to make MPU9250");
        let filter_gain = 0.1;
        let madgwick = Madgwick::new(sample_period.as_secs_f32(), filter_gain);


        Mpu { mpu9250 , madgwick, sample_period }
    }

    pub fn calibrate(&mut self) {
        // Accelerometer average is uleses, cause we need to do it for every direction
        let _accelerometer_avg: [f32; 3] = self.mpu9250.calibrate_at_rest(&mut Delay)
            .expect("calibration failed");
    }

    pub fn update(&mut self) -> &nalgebra::Unit<nalgebra::Quaternion<f32>> {
        let all: MargMeasurements<Vector3<f32>>  = self.mpu9250.all()
            .expect("unable to read from MPU");
        
        // TODO use magnetometer for god's sake
        //self.madgwick.update(&all.gyro, &all.accel, &all.mag)
        self.madgwick.update_imu(&all.gyro, &all.accel)
            .expect("Madgwick update succeeded")
    }
}

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
    let mag_who_am_i = mpu9250.ak8963_who_am_i()
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
