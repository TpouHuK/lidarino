use lidarino::hardware::mpu::*;
use linux_embedded_hal::{Delay, I2cdev};
use mpu9250::*;
use std::thread;
use std::time::Duration;
use std::time::Instant;

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
    mpu9250.mag_scale(mpu9250::MagScale::_16BITS).unwrap();
    eprintln!("MPU onboard accel bias: {:?}", mpu9250.get_accel_bias::<[f32; 3]>());
    eprintln!("MPU onboard gyro bias: {:?}", mpu9250.get_gyro_bias::<[f32; 3]>());
    eprintln!("MPU onboard mag sensitivity adjustments: {:?}", mpu9250.mag_sensitivity_adjustments::<[f32; 3]>());
    eprintln!("Trying to test update frequency (unscaled version)");

    use mpu9250::*;
    //mpu9250.gyro_temp_data_rate(GyroTempDataRate::DlpfConf(Dlpf::_6)).unwrap();
    mpu9250.gyro_temp_data_rate(GyroTempDataRate::DlpfConf(Dlpf::_7)).unwrap();
    let n = 5000;
    let mut readings = Vec::with_capacity(n);
    let start_time = Instant::now();
    for _ in 0..n { 
        let r_start = Instant::now();
        readings.push((mpu9250.accel::<[f32; 3]>().expect("unable to read from MPU!"), r_start.elapsed()));
    }
    let duration = start_time.elapsed();
    eprintln!("{n} reading were made in {duration:?}");
    let readings_per_second = n as f32 / duration.as_secs_f32() ;
    eprintln!("readings per second: {readings_per_second}");
    for r in  readings {
        println!("{} {} {} {}", r.0[0], r.0[1], r.0[2], r.1.as_secs_f32());
    }
    
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
    //test_madwick_mpu();
    test_raw_mpu();
}
