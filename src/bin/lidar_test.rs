extern crate rplidar_drv;

use rplidar_drv::*;
use rpos_drv::Channel;

fn main() {
    let mut serial_port = mio_serial::new("/dev/ttyUSB0", 115200)
        .open()
        .expect("Failed to open ttyUSB0 port.");

    serial_port
        .write_data_terminal_ready(false)
        .expect("failed to clear DTR");

    let channel = Channel::<RplidarHostProtocol, dyn mio_serial::SerialPort>::new(
        RplidarHostProtocol::new(),
        serial_port,
    );

    let mut rplidar = RplidarDevice::new(channel);

    let device_info = rplidar
        .get_device_info()
        .expect("failed to get device info");

    println!("Connected to LIDAR: ");
    println!("    Model: {}", device_info.model);
    println!(
        "    Firmware Version: {}.{}",
        device_info.firmware_version >> 8,
        device_info.firmware_version & 0xff
    );
    println!("    Hardware Version: {}", device_info.hardware_version);
    println!("    Serial Number: {:?}", device_info.serialnum);

    let device_health = rplidar
        .get_device_health()
        .expect("failed to get device health");

    match device_health {
        Health::Healthy => {
            println!("LIDAR is healthy.");
        }
        Health::Warning(error_code) => {
            println!("LIDAR is unhealthy, warn: {error_code:04X}");
        }
        Health::Error(error_code) => {
            println!("LIDAR is unhealthy, error: {error_code:04X}");
        }
    }

    let all_supported_scan_modes = rplidar
        .get_all_supported_scan_modes()
        .expect("failed to get all supported scan modes");

    println!("All supported scan modes:");
    for scan_mode in all_supported_scan_modes {
        println!(
            "    {:2} {:16}: Max Distance: {:6.2}m, Ans Type: {:02X}, Us per sample: {:.2}us",
            scan_mode.id,
            scan_mode.name,
            scan_mode.max_distance,
            scan_mode.ans_type,
            scan_mode.us_per_sample
        );
    }

    let typical_scan_mode = rplidar
        .get_typical_scan_mode()
        .expect("failed to get typical scan mode");

    println!("Typical scan mode: {typical_scan_mode}");

    match rplidar.check_motor_ctrl_support() {
        Ok(support) if support => {
            println!("Accessory board is detected and support motor control, starting motor...");
            rplidar.set_motor_pwm(600).expect("failed to start motor");
        }
        Ok(_) => {
            println!("Accessory board is detected, but doesn't support motor control");
        }
        Err(_) => {
            println!("Accessory board isn't detected");
        }
    }

    println!("Starting LIDAR in typical mode...");

    let actual_mode = rplidar
        .start_scan()
        .expect("failed to start scan in standard mode");

    println!("Started scan in mode `{}`", actual_mode.name);

    let start_time = std::time::Instant::now();

    loop {
        match rplidar.grab_scan() {
            Ok(scan) => {
                println!(
                    "[{:6}s] {} points per scan",
                    start_time.elapsed().as_secs(),
                    scan.len()
                );

                /*
                 for scan_point in scan {
                    println!(
                        "    Angle: {:5.2}, Distance: {:8.4}, Valid: {:5}, Sync: {:5}",
                        scan_point.angle(),
                        scan_point.distance(),
                        scan_point.is_valid(),
                        scan_point.is_sync()
                    )
                }*/
            }
            Err(err) => {
                if let rplidar_drv::Error::OperationTimeout = err {
                    continue;
                } else {
                    println!("Error: {err:?}");
                    break;
                }
            }
        }
    }
}
