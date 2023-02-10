use lidarino::hardware::mpu::*;
use linux_embedded_hal::{Delay, I2cdev};
use mpu9250::*;

use std::thread;
use std::time::Duration;
use std::time::Instant;

const I2C_ADDR: &str = "/dev/i2c-1";

pub fn test_madwick_mpu() {
    let mut mpu = Mpu::new(Duration::from_millis(1));
    mpu.mpu9250
        .gyro_temp_data_rate(GyroTempDataRate::DlpfConf(Dlpf::_6))
        .unwrap();
    mpu.mpu9250
        .accel_data_rate(AccelDataRate::DlpfConf(Dlpf::_6))
        .unwrap();

    println!("Calibration started.");
    mpu.calibrate();
    println!("Calibration finished.");
    println!("gyro bias: {:?}", mpu.gyro_bias);

    let mut prev_measurement = Instant::now();
    let rate_hz = 1000;
    let tick_time = Duration::from_secs(1) / rate_hz;
    let start = Instant::now();
    let mut ticks = 0u32;

    loop {
        ticks += 1;
        let mut wait_amount = tick_time.checked_sub(prev_measurement.elapsed());

        while wait_amount.is_some() {
            wait_amount = tick_time.checked_sub(prev_measurement.elapsed());
        }

        prev_measurement = Instant::now();
        let quat = mpu.update();
        let (roll, pitch, yaw) = quat.euler_angles();

        let roll = roll.to_degrees();
        let pitch = pitch.to_degrees();
        let yaw = yaw.to_degrees();

        let rate = ticks as f32 / start.elapsed().as_secs_f32();
        if ticks % 1000 == 0 {
            let accel = mpu.raw_accel;
            let gyro = mpu.raw_gyro;
            print!("\rpitch={pitch:>6.4}, roll={roll:>6.4}, yaw={yaw:>6.4} rate={rate:>6.4}, accel={accel:?}, gyro={gyro:?}");
            use std::io::Write;
            std::io::stdout().flush().unwrap();
        }
    }
}

pub fn test_raw_mpu() {
    let i2c = I2cdev::new(I2C_ADDR).unwrap();
    let mut mpu9250 = Mpu9250::marg_default(i2c, &mut Delay).expect("unable to make MPU9250");
    mpu9250.mag_scale(mpu9250::MagScale::_16BITS).unwrap();
    eprintln!(
        "MPU onboard gyro bias: {:?}",
        mpu9250.get_gyro_bias::<[f32; 3]>()
    );
    eprintln!(
        "MPU onboard mag sensitivity adjustments: {:?}",
        mpu9250.mag_sensitivity_adjustments::<[f32; 3]>()
    );
    eprintln!("Trying to test update frequency dlpf0 rate");

    use mpu9250::*;
    //mpu9250.gyro_temp_data_rate(GyroTempDataRate::DlpfConf(Dlpf::_6)).unwrap();
    mpu9250
        .gyro_temp_data_rate(GyroTempDataRate::DlpfConf(Dlpf::_1))
        .unwrap();
    eprintln!(
        "MPU onboard accel bias before calib: {:?}",
        mpu9250.get_accel_bias::<[f32; 3]>()
    );
    let _: Result<[f32; 3], _> = mpu9250.calibrate_at_rest(&mut linux_embedded_hal::Delay);
    eprintln!(
        "MPU onboard accel bias after calib: {:?}",
        mpu9250.get_accel_bias::<[f32; 3]>()
    );

    let n = 5000;
    let mut readings = Vec::with_capacity(n);
    let start_time = Instant::now();
    for _ in 0..n {
        let r_start = Instant::now();
        readings.push((
            mpu9250
                .gyro::<[f32; 3]>()
                .expect("unable to read from MPU!"),
            r_start.elapsed(),
        ));
    }
    let duration = start_time.elapsed();
    eprintln!("{n} reading were made in {duration:?}");
    let readings_per_second = n as f32 / duration.as_secs_f32();
    eprintln!("readings per second: {readings_per_second}");
    for r in readings {
        println!("{} {} {} {}", r.0[0], r.0[1], r.0[2], r.1.as_secs_f32());
    }
}

fn main() {
    test_madwick_mpu();
    //test_raw_mpu();
}
