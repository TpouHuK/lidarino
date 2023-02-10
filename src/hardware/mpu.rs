//! MPU9250 with rotation tracking.

use std::time::Duration;
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;
// use std::thread;

use ahrs::{Ahrs, Madgwick};
use linux_embedded_hal::{Delay, I2cdev};
use mpu9250::*;

const I2C_ADDR: &str = "/dev/i2c-1";

/// MPU9250 with rotation tracking.
pub struct Mpu {
    pub mpu9250: Mpu9250<I2cDevice<I2cdev>, mpu9250::Marg>,
    pub gyro_bias: [f32; 3],
    pub madgwick: Madgwick<f32>,
    pub raw_accel: [f32; 3],
    pub raw_gyro: [f32; 3],
    pub _sample_period: Duration,
}

impl Mpu {
    /// Create new MPU9250.
    pub fn new(sample_period: Duration) -> Self {
        let i2c = I2cdev::new(I2C_ADDR).unwrap();
        let mpu9250 = Mpu9250::marg_default(i2c, &mut Delay).expect("unable to make MPU9250");
        let filter_gain = 0.5;
        let madgwick = Madgwick::new(sample_period.as_secs_f32(), filter_gain);

        let gyro_bias = [0.0; 3];

        Mpu {
            mpu9250,
            madgwick,
            _sample_period: sample_period,
            raw_accel: [0.0; 3],
            raw_gyro: [0.0; 3],
            gyro_bias,
        }
    }

    pub fn calibrate(&mut self) {
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

        for i in 0..3 {
            self.gyro_bias[i] /= (amount as f32) * -1.0;
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
        // let all: MargMeasurements<Vector3<f32>> =
        // self.mpu9250.all().expect("unable to read from MPU");
        let mut gyro: [f32; 3] = self.mpu9250.gyro().unwrap();
        let accel: [f32; 3] = self.mpu9250.accel().unwrap();

        // TODO use magnetometer for god's sake
        //self.madgwick.update(&all.gyro, &all.accel, &all.mag)

        //let mut gyro = all.gyro;
        for (i, gyro) in gyro.iter_mut().enumerate() {
            *gyro += self.gyro_bias[i];
        }

        self.raw_accel = accel;
        self.raw_gyro = gyro;

        self.madgwick
            .update_imu(&gyro.into(), &accel.into())
            .expect("Madgwick update succeeded")
    }
}

use nalgebra::UnitQuaternion;
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn control_loop(mut mpu: Mpu, quaternion: Arc<Mutex<UnitQuaternion<f32>>>) {
    let rate_hz = 1000;
    let tick_time = Duration::from_secs(1) / rate_hz;

    let mut prev_measurement = Instant::now();
    loop {
        let mut wait_amount = tick_time.checked_sub(prev_measurement.elapsed());

        while wait_amount.is_some() {
            wait_amount = tick_time.checked_sub(prev_measurement.elapsed());
        }

        prev_measurement = Instant::now();
        let quat = mpu.update();
        let mut lock = quaternion.lock().unwrap();
        *lock = *quat;
    }
}

pub struct OrientationController {
    quat: Arc<Mutex<UnitQuaternion<f32>>>,
}

impl Default for OrientationController {
    fn default() -> Self {
        Self::new()
    }
}

impl OrientationController {
    pub fn new() -> Self {
        let quat = Arc::new(Mutex::new(UnitQuaternion::default()));
        let quat_clone = quat.clone();
        std::thread::spawn(|| {
            let mut mpu = Mpu::new(Duration::from_millis(1));
            mpu.calibrate();
            control_loop(mpu, quat_clone);
        });
        OrientationController { quat }
    }

    pub fn get_quat(&self) -> UnitQuaternion<f32> {
        *self.quat.lock().unwrap()
    }
}
