use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::sync::Arc;

use ahrs::{ Madgwick, Ahrs } ;
//use linux_embedded_hal::{I2cdev, Delay};
use mpu9250::*;

const I2C_ADDR: &str = "/dev/i2c-1";

/*
pub struct Mpu {
    mpu9250: Mpu9250<I2cDevice<I2cdev>, mpu9250::Marg>,
    gyro_bias: [f32; 3],
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

        let gyro_bias = [0.0; 3];

        Mpu { mpu9250 , madgwick, sample_period, gyro_bias }
    }

    pub fn calibrate(&mut self) {
        // Accelerometer average is uleses, cause we need to do it for every direction
        let _accelerometer_avg: [f32; 3] = self.mpu9250.calibrate_at_rest(&mut Delay)
            .expect("calibration failed");
        eprintln!("Internal calibration done");

        let start = std::time::Instant::now();
        self.gyro_bias = [0.0; 3];
        let mut amount = 0;

        while start.elapsed().as_secs() < 1 {
            amount += 1;
            let gyro_reading: [f32; 3] = self.mpu9250.gyro().unwrap();
            for (i, reading) in gyro_reading.iter().enumerate() {
                self.gyro_bias[i] += reading;
            }
        }
        eprintln!("Made {amount} reading during 1 second of averaging");

        for i in 0..3 {
            self.gyro_bias[i] /= -amount as f32;
        }
    }

    pub fn read_accel(&mut self) -> [f32; 3] {
        let mut gyro: [f32; 3] = self.mpu9250.gyro().unwrap();
        for (i, gyro_val) in gyro.iter_mut().enumerate() {
            *gyro_val += self.gyro_bias[i];
        }
        gyro
    }

    pub fn update(&mut self) -> &nalgebra::Unit<nalgebra::Quaternion<f32>> {
        let all: MargMeasurements<Vector3<f32>>  = self.mpu9250.all()
            .expect("unable to read from MPU");
        
        // TODO use magnetometer for god's sake
        //self.madgwick.update(&all.gyro, &all.accel, &all.mag)
        let mut gyro = all.gyro;
        for i in 0..3 {
            gyro[i] += self.gyro_bias[i];
        }
        self.madgwick.update_imu(&gyro, &all.accel)
            .expect("Madgwick update succeeded")
    }
} */
