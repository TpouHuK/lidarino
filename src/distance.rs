//! HI50 laser measurement sensor.
//!
//! # Example
//! ```
//! let sensor = DistanceSensor::new();
//! let controller = DistanceController::new(sensor);
//!
//! // Request and wait for measurement, blocks thread
//! let measurement = controller.get_measurement();
//!
//! // Or...
//! controller.request_measurement();
//! // ... do some stuff ...
//! controller.wait_until_done();
//! let measurement = controller.get_last_measurement();
//!
//! ```
use mio_serial::*;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Condvar;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Request status of DistanceController
/// Used to request distance readings and wait on them.
type Status = Arc<(Mutex<bool>, Condvar)>;

/// Result of measuring distance with DistanceSensor.
type DistanceReading = Arc<(AtomicU32, AtomicU32)>;

// struct DistanceReading // TODO as a separate type

/// Separate thread control loop for [`DistanceController`]
fn distance_sensor_control_loop(
    mut distance_sensor: DistanceSensor,
    status: Status,
    distance_reading: DistanceReading,
    kill_switch: Arc<AtomicBool>,
) {
    let (lock, cvar) = &*status;
    let mut is_done = lock.lock().unwrap();

    loop {
        is_done = cvar.wait(is_done).unwrap();
        if kill_switch.load(Ordering::Relaxed) {
            break;
        }
        if !*is_done {
            drop(is_done);
            let reading = distance_sensor.read_distance().unwrap(); // TODO FIX UNWRAP ADD RETRIES
                                                                    //let reading = (42u32, 00u32);
            thread::sleep(Duration::from_secs(3));

            let (dist, qual) = &*distance_reading;
            dist.store(reading.0, Ordering::Relaxed);
            qual.store(reading.1, Ordering::Relaxed);

            is_done = lock.lock().unwrap();
            *is_done = true;
            cvar.notify_all();
        }
    }
    *is_done = true;
    cvar.notify_all();
}

/// Controller for HI50 distance measurement sensor.
pub struct DistanceController {
    status: Status,
    distance_reading: DistanceReading,

    thread_handle: Option<thread::JoinHandle<()>>,
    kill_switch: Arc<AtomicBool>,
}

impl DistanceController {
    /// Create new [`DistanceController`] for `distance_sensor`
    pub fn new(distance_sensor: DistanceSensor) -> Self {
        let distance_reading = Arc::new((AtomicU32::new(0), AtomicU32::new(0)));
        let status = Arc::new((Mutex::new(true), Condvar::new()));
        let kill_switch = Arc::new(AtomicBool::new(false));

        let distance_reading_clone = distance_reading.clone();
        let status_clone = status.clone();
        let kill_switch_clone = kill_switch.clone();

        let thread_handle = thread::spawn(move || {
            distance_sensor_control_loop(
                distance_sensor,
                status_clone,
                distance_reading_clone,
                kill_switch_clone,
            )
        });
        DistanceController {
            status,
            distance_reading,
            thread_handle: Some(thread_handle),
            kill_switch,
        }
    }

    /// Blocks thread untill current measurement request is complete
    /// Instantly returns if theres no request pending.
    pub fn wait_until_done(&self) {
        let (lock, cvar) = &*self.status;
        let mut is_done = lock.lock().unwrap();
        while !*is_done {
            is_done = cvar.wait(is_done).unwrap();
        }
    }

    /// Non-blocking request to measure distance.
    pub fn request_measurement(&self) {
        let (lock, cvar) = &*self.status;
        let mut is_done = lock.lock().unwrap();
        *is_done = false;
        cvar.notify_all();
    }

    /// Blocking request for measurement. Returns result of measurement
    pub fn get_measurement(&self) -> (u32, u32) {
        self.request_measurement();
        self.wait_until_done();
        let (distance, quality) = &*self.distance_reading;
        (
            distance.load(Ordering::Relaxed),
            quality.load(Ordering::Relaxed),
        )
    }

    /// Non-blocking get of last measurement.
    pub fn get_last_measurement(&self) -> (u32, u32) {
        let (distance, quality) = &*self.distance_reading;
        (
            distance.load(Ordering::Relaxed),
            quality.load(Ordering::Relaxed),
        )
    }
}

impl Drop for DistanceController {
    // Never used (yet) as DistanceController is static
    fn drop(&mut self) {
        self.kill_switch.store(true, Ordering::Relaxed);
        self.status.1.notify_all();
        if let Some(thread_handle) = self.thread_handle.take() {
            thread_handle.join().expect("Control thread did not panic");
        }
    }
}

/// HI50 Distance sensor.
pub struct DistanceSensor {
    tty_port: Box<dyn SerialPort>,
}

impl Default for DistanceSensor {
    fn default() -> Self {
        Self::new()
    }
}

impl DistanceSensor {
    /// Create new HI50 Distance sensor with hardcoded values.
    pub fn new() -> Self {
        let tty_port = mio_serial::new("/dev/ttyS0", 19200)
            .timeout(Duration::from_millis(3500))
            .data_bits(DataBits::Eight)
            .open()
            .expect("Failed to open ttyS0 port.");
        DistanceSensor { tty_port }
    }

    /// Enable laser. Sends `b"O"` on serial.
    pub fn start(&mut self) -> Result<()> {
        self.tty_port.write_all(b"O").expect("enabled laser");
        self.tty_port.flush().expect("enabled laser");
        let mut buf: Vec<u8> = vec![0; 7];
        self.tty_port.read_exact(&mut buf).unwrap();
        assert_eq!(buf, b"O,OK!\r\n");
        Ok(())
    }

    /// Make "slow" measurement. Sends `b"D"` on serial.
    pub fn read_distance(&mut self) -> Result<(u32, u32)> {
        self.tty_port.write_all(b"D").expect("enabled laser");
        self.tty_port.flush().expect("enabled laser");

        // TODO add error handling
        let mut buf: Vec<u8> = vec![0; 16];
        self.tty_port.read_exact(&mut buf)?; // TOOD FIX ERROR
                                             // 'D: 5.614m,1211\r\n'
        let range = [&buf[3..=3], &buf[5..=7]].concat();
        let string = String::from_utf8(range).unwrap();
        let number: u32 = string.parse().unwrap();

        let q_range = &buf[10..=13];
        let q_string = String::from_utf8(q_range.to_vec()).unwrap();
        let q_number: u32 = q_string.parse().unwrap();

        Ok((number, q_number))
    }

    /// Make "fast" measurement. Sends `b"F"` on serial.
    pub fn read_distance_fast(&mut self) -> Result<(u32, u32)> {
        self.tty_port.write_all(b"F").expect("enabled laser");
        self.tty_port.flush().expect("enabled laser");

        // TODO add error handling
        let mut buf: Vec<u8> = vec![0; 16];
        self.tty_port.read_exact(&mut buf)?; // TOOD FIX ERROR
                                             // 'D: 5.614m,1211\r\n'
        let range = [&buf[3..=3], &buf[5..=7]].concat();
        let string = String::from_utf8(range).unwrap();
        let number: u32 = string.parse().unwrap();

        let q_range = &buf[10..=13];
        let q_string = String::from_utf8(q_range.to_vec()).unwrap();
        let q_number: u32 = q_string.parse().unwrap();

        Ok((number, q_number))
    }

    /// Close laser. Sends `b"C"` on serial.
    pub fn stop(&mut self) -> Result<()> {
        self.tty_port.write_all(b"C").expect("enabled laser");
        self.tty_port.flush().expect("enabled laser");
        let mut buf: Vec<u8> = vec![0; 7];
        self.tty_port.read_exact(&mut buf).unwrap();
        assert_eq!(buf, b"C,OK!\r\n");
        Ok(())
    }
}
