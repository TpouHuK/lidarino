extern crate rplidar_drv;

use std::{time::Duration, thread::sleep};

//use mio_serial::*;
use rplidar_drv::RplidarDevice;

fn main() {
    let serial_port = mio_serial::new("/dev/ttyUSB0", 115200)
            .open().expect("Failed to open ttyUSB0 port.");
    let mut rplidar = RplidarDevice::with_stream(serial_port);
    rplidar.stop();
    drop(rplidar);
    let serial_port = mio_serial::new("/dev/ttyUSB0", 115200)
            .open().expect("Failed to open ttyUSB0 port.");
    let mut rplidar = RplidarDevice::with_stream(serial_port);


    let device_info = rplidar.get_device_info().unwrap();
    println!("{device_info:?}");

    let health = rplidar.get_device_health().expect("no health");
    dbg!(health);

     match rplidar.check_motor_ctrl_support() {
        Ok(support) if support == true => {
            println!("Accessory board is detected and support motor control, starting motor...");
            rplidar.set_motor_pwm(600).expect("failed to start motor");
        },
        Ok(_) => {
            println!("Accessory board is detected, but doesn't support motor control");
        },
        Err(_) => {
            println!("Accessory board isn't detected");
        }
    }

    dbg!(rplidar.get_all_supported_scan_modes());

    dbg!(rplidar.get_typical_scan_mode().unwrap());
    let options = rplidar_drv::ScanOptions::force_scan_with_mode(0);
    rplidar.start_motor().unwrap();

    loop {
        let a =  rplidar.start_scan_with_options(&options).unwrap();
        dbg!(a);
        sleep(Duration::from_secs(10));
        let scan_point = rplidar.grab_scan();
        match scan_point {
            Ok(scan_point) => { println!("{:?}", scan_point); }
            Err(err) => { println!("{:?}", err); } 
        }
    }
}
