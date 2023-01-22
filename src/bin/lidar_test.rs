extern crate rplidar_drv;

use mio_serial::*;
use rplidar_drv::RplidarDevice;
use std::time::Duration;

fn main() {
    let serial_port = mio_serial::new("/dev/ttyS0", 19200)
            .timeout(Duration::from_millis(3500))
            .data_bits(DataBits::Eight)
            .open().expect("Failed to open ttyS0 port.");
    let mut rplidar = RplidarDevice::with_stream(serial_port);

    let device_info = rplidar.get_device_info().unwrap();
    rplidar.start_scan().unwrap();

    loop {
        let scan_point = rplidar.grab_scan_point().unwrap();
    }
}
