//! MPU9250 with rotation tracking.

use anyhow::Result;
use linux_embedded_hal::{Delay, I2cdev};
use mpu9250::*;
use nalgebra::UnitQuaternion;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;

const I2C_ADDR: &str = "/dev/i2c-1";

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct MpuConfig {
    pub gyro_bias: [f32; 3],
    pub accel_bias: [f32; 3],
    pub accel_scale: [f32; 3],
}

impl Default for MpuConfig {
    fn default() -> Self {
        MpuConfig {
            gyro_bias: [0.0; 3],
            accel_bias: [0.0; 3],
            accel_scale: [1.0; 3],
        }
    }
}

pub struct Mpu {
    pub mpu9250: Mpu9250<I2cDevice<I2cdev>, mpu9250::Marg>,
    pub config: MpuConfig,
}

impl Mpu {
    /// Create new MPU9250.
    pub fn new(config: MpuConfig) -> Self {
        let i2c = I2cdev::new(I2C_ADDR).unwrap();
        let mpu9250 = Mpu9250::marg_default(i2c, &mut Delay).expect("unable to make MPU9250");
        Mpu { mpu9250, config }
    }

    pub fn get_accel_gyro(&mut self) -> Result<([f32; 3], [f32; 3])> {
        let measurements = self
            .mpu9250
            .all()
            .map_err(|_e| anyhow::format_err!("I2C is ded"))?;
        let mut gyro: [f32; 3] = measurements.gyro;
        for (gyro, bias) in gyro.iter_mut().zip(self.config.gyro_bias) {
            *gyro -= bias;
        }
        let mut accel: [f32; 3] = measurements.accel;
        for ((accel, bias), scale) in accel
            .iter_mut()
            .zip(self.config.accel_bias)
            .zip(self.config.accel_scale)
        {
            *accel -= bias;
            *accel *= scale;
        }
        Ok((accel, gyro))
    }
}

#[must_use]
pub fn get_magnetometer_data(mpu: &mut Mpu, duration: &Duration) -> Vec<[f32; 3]> {
    let start_time = Instant::now();
    let mut data = Vec::new();
    while start_time.elapsed() < *duration {
        let reading = mpu.mpu9250.mag();
        if let Ok(reading) = reading {
            match data.last() {
                None => data.push(reading),
                Some(last) => {
                    if last != &reading {
                        data.push(reading)
                    }
                }
            }
        } else {
            eprintln!("Error reading from mag: {reading:?}");
        }
    }
    data
}

#[must_use]
pub fn calculate_gyro_bias(mpu: &mut Mpu, duration: &Duration) -> [f32; 3] {
    let start_time = Instant::now();
    let mut gyro_biases: [f32; 3] = [0.0, 0.0, 0.0];
    let mut amount_of_readings = 0;
    while start_time.elapsed() < *duration {
        let reading: [f32; 3] = mpu.mpu9250.gyro().expect("unable to make gyro reading");
        for (bias, reading) in gyro_biases.iter_mut().zip(reading) {
            *bias += reading;
        }
        amount_of_readings += 1;
    }
    gyro_biases.map(|b| b / amount_of_readings as f32)
}

#[must_use]
pub fn calculate_accel_bias_and_scale(mpu: &mut Mpu) -> ([f32; 3], [f32; 3]) {
    eprintln!("Put mpu into 6 different positions, every time pointing some axis to ground.");
    let stdin = std::io::stdin();
    let mut zero_values: [Vec<f32>; 3] = Default::default();
    let mut axis_ranges: [Vec<f32>; 3] = Default::default();

    for i in 0..6 {
        eprintln!("Put mpu into position {i} and press <Enter>.");
        let mut input = String::new();
        let _ = stdin.read_line(&mut input);

        let accel: [f32; 3] = mpu.mpu9250.accel().expect("unable to make accel reading");
        eprintln!("got reading: {accel:?}");

        for (i, reading) in accel.iter().enumerate() {
            if reading.abs() > 6.0 {
                eprintln!("that was {i} axis");
                axis_ranges[i].push(*reading);
            } else {
                zero_values[i].push(*reading);
            }
        }
    }

    for zero_array in zero_values.iter() {
        assert!(zero_array.len() == 4);
    }

    for axis_array in axis_ranges.iter() {
        assert!(axis_array.len() == 2);
    }

    use std::convert::TryInto;
    let accel_bias: [f32; 3] = zero_values
        .iter()
        .map(|v| v.iter().sum::<f32>() / 4.0)
        .collect::<Vec<f32>>()
        .try_into()
        .unwrap();

    let accel_scale: [f32; 3] = axis_ranges
        .iter()
        .map(|v| (2.0 * mpu9250::G) / (v[0].abs() + v[1].abs()))
        .collect::<Vec<f32>>()
        .try_into()
        .unwrap();

    (accel_bias, accel_scale)
}

fn control_loop(mut mpu: Mpu, quaternion: Arc<Mutex<UnitQuaternion<f32>>>) {
    let rate_hz = 1000;
    let sample_period = Duration::from_secs(1) / rate_hz;
    let filter_gain = 0.1;

    let mut prev_measurement = Instant::now();
    let mut ahrs = ahrs::Madgwick::new(sample_period.as_secs_f32(), filter_gain);
    loop {
        let mut wait_amount = sample_period.checked_sub(prev_measurement.elapsed());

        while wait_amount.is_some() {
            wait_amount = sample_period.checked_sub(prev_measurement.elapsed());
        }

        prev_measurement = Instant::now();
        use ahrs::Ahrs;
        use nalgebra::Vector3;

        let measurement = mpu.get_accel_gyro();
        if let Ok((accel, gyro)) = measurement {
            let quat = ahrs
                .update_imu(
                    &Vector3::new(gyro[0], gyro[1], gyro[2]),
                    &Vector3::new(accel[0], accel[1], accel[2]),
                )
                .unwrap();
            let mut lock = quaternion.lock().unwrap();
            *lock = *quat;
        }
    }
}

pub struct OrientationController {
    quat: Arc<Mutex<UnitQuaternion<f32>>>,
}

impl OrientationController {
    pub fn new(mpu: Mpu) -> Self {
        let quat = Arc::new(Mutex::new(UnitQuaternion::default()));
        let quat_clone = quat.clone();
        std::thread::spawn(|| {
            control_loop(mpu, quat_clone);
        });
        OrientationController { quat }
    }

    pub fn get_quat(&self) -> UnitQuaternion<f32> {
        *self.quat.lock().unwrap()
    }
}
