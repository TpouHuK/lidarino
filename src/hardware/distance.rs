//! HI50 laser measurement sensor.
//!
//! # Example
//! ```//! let sensor = DistanceSensor::new();
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

use crate::shared::{IsDead, SharedState};
use mio_serial::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Distance {
    millimeters: u32,
}

impl Distance {
    pub fn from_mm(millimeters: u32) -> Self {
        Distance { millimeters }
    }

    pub fn as_mm(&self) -> u32 {
        self.millimeters
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub enum DistanceReading {
    #[default]
    NoReading,
    Ok {
        distance: Distance,
        quality: u16,
        measuring_time: Duration,
    },
    Err {
        error: DistanceReadingError,
        measuring_time: Duration,
    },
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum ReadingState {
    Ready,
    Pending,
    Dead,
}

impl IsDead for ReadingState {
    fn is_dead(&self) -> bool {
        self == &ReadingState::Dead
    }
}

impl Default for ReadingState {
    fn default() -> Self {
        ReadingState::Ready
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum DistanceReadingError {
    UnknownError = 0,
    /// VBAT too low, power boltage should >= 2.0V
    VbatTooLow = 1,
    /// Internal error, don't care
    InternalError = 2,
    /// Module temperature is too low(< -20C)
    TempTooLow = 3,
    /// Module temperature is too high(> +40C)
    TempTooHigh = 4,
    /// Target out of measure range
    TargetOutOfMeasureRange = 5,
    /// Invalid measure result
    InvalidMeasureResult = 6,
    /// Background light is too strong
    BackgroundLightIsTooStrong = 7,
    /// Laser signal is too weak
    LaserSignalIsTooWeak = 8,
    /// Laser signal is too strong
    LaserSignalIsTooStrong = 9,
    /// Hardware fault 1
    HardwareFault1 = 10,
    /// Hardware fault 2
    HardwareFault2 = 11,
    /// Hardware fault 3
    HardwareFault3 = 12,
    /// Hardware fault 4
    HardwareFault4 = 13,
    /// Hardware fault 5
    HardwareFault5 = 14,
    /// Laser signal is not stable
    LaserSignalIsNotStable = 15,
    /// Hardware fault 6
    HardwareFault6 = 16,
    /// Hardware fault 7
    HardwareFault7 = 17,
    ParsingError,
}

impl DistanceReadingError {
    pub fn new(code: u8) -> Self {
        use DistanceReadingError::*;
        match code {
            x if x == VbatTooLow as u8 => VbatTooLow,
            x if x == InternalError as u8 => InternalError,
            x if x == TempTooLow as u8 => TempTooLow,
            x if x == TempTooHigh as u8 => TempTooHigh,
            x if x == TargetOutOfMeasureRange as u8 => TargetOutOfMeasureRange,
            x if x == InvalidMeasureResult as u8 => InvalidMeasureResult,
            x if x == BackgroundLightIsTooStrong as u8 => BackgroundLightIsTooStrong,
            x if x == LaserSignalIsTooWeak as u8 => LaserSignalIsTooWeak,
            x if x == LaserSignalIsTooStrong as u8 => LaserSignalIsTooStrong,
            x if x == HardwareFault1 as u8 => HardwareFault1,
            x if x == HardwareFault2 as u8 => HardwareFault2,
            x if x == HardwareFault3 as u8 => HardwareFault3,
            x if x == HardwareFault4 as u8 => HardwareFault4,
            x if x == HardwareFault5 as u8 => HardwareFault5,
            x if x == LaserSignalIsNotStable as u8 => LaserSignalIsNotStable,
            x if x == HardwareFault6 as u8 => HardwareFault6,
            x if x == HardwareFault7 as u8 => HardwareFault7,
            _ => UnknownError, // Todo change to unknown error code
        }
    }
}

/// Separate thread control loop for [`DistanceController`]
fn distance_sensor_control_loop(
    mut distance_sensor: DistanceSensor,
    state: Arc<SharedState<ReadingState>>,
    mode: Arc<Mutex<ReadingMode>>,
    distance_reading: Arc<Mutex<DistanceReading>>,
) {
    loop {
        state.await_state(ReadingState::Pending);

        if state.get_state().is_dead() {
            break;
        }

        let mut reading_m = distance_reading.lock().unwrap();
        *reading_m = distance_sensor.read_distance_mode(*mode.lock().unwrap());
        state.set_state(ReadingState::Ready);
    }
}

/// Controller for HI50 distance measurement sensor.
pub struct DistanceController {
    state: Arc<SharedState<ReadingState>>,
    reading: Arc<Mutex<DistanceReading>>,
    reading_mode: Arc<Mutex<ReadingMode>>,
    _thread_handle: Option<thread::JoinHandle<()>>,
}

impl DistanceController {
    /// Create new [`DistanceController`] for `distance_sensor`
    pub fn new(distance_sensor: DistanceSensor) -> Self {
        let state: Arc<SharedState<ReadingState>> = Default::default();
        let reading: Arc<Mutex<DistanceReading>> = Default::default();

        let state_clone = state.clone();
        let reading_clone = reading.clone();

        let reading_mode = Arc::new(Mutex::new(ReadingMode::Default));
        let reading_mode_clone = reading_mode.clone();

        let thread_handle = thread::spawn(move || {
            distance_sensor_control_loop(
                distance_sensor,
                state_clone,
                reading_mode_clone,
                reading_clone,
            )
        });

        DistanceController {
            state,
            reading,
            reading_mode,
            _thread_handle: Some(thread_handle),
        }
    }

    pub fn set_mode(&self, mode: ReadingMode) {
        *self.reading_mode.lock().unwrap() = mode;
    }

    pub fn get_mode(&self) -> ReadingMode {
        *self.reading_mode.lock().unwrap()
    }

    /// Blocks thread untill current measurement request is complete
    /// Instantly returns if theres no request pending.
    pub fn await_measurement(&self) {
        self.state.await_state(ReadingState::Ready);
    }

    /// Non-blocking request to measure distance.
    pub fn request_measurement(&self) {
        self.state.set_state(ReadingState::Pending);
    }

    /// Blocking request for measurement. Returns result of measurement
    pub fn get_measurement(&self) -> DistanceReading {
        self.request_measurement();
        self.await_measurement();
        *self.reading.lock().unwrap()
    }

    /// Non-blocking get of last measurement.
    pub fn get_last_measurement(&self) -> DistanceReading {
        *self.reading.lock().unwrap()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReadingMode {
    Default,
    Fast,
    Slow,
}

impl ReadingMode {
    pub fn as_u8(&self) -> &'static [u8] {
        match self {
            ReadingMode::Default => b"D",
            ReadingMode::Fast => b"F",
            ReadingMode::Slow => b"M",
        }
    }
}

pub use sensor::*;

//TODO, add propper logging for errors, maybe improve errors
/* Real world sensor */
#[cfg(not(feature = "mock_hardware"))]
mod sensor {
    use super::*;

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

        pub fn read_distance_mode(&mut self, mode: ReadingMode) -> DistanceReading {
            let start = Instant::now();

            self.tty_port
                .write_all(mode.as_u8())
                .expect("enabled laser");
            self.tty_port.flush().expect("enabled laser");

            // HI50 returns following messages:
            // Where [D/F/M] is D or F or M, the same as in the request. (Maybe serial just echoes
            // back input, don't know for sure)
            //
            // On success: `[D/F/M]: 5.614m,1211\r\n`
            // On error: `[D/F/M]:Er08!\r\n`
            let mut buf: Vec<u8> = vec![0; 16];
            let mut filled_len = 0;
            loop {
                let reading_result = self.tty_port.read(&mut buf[filled_len..]);
                match reading_result {
                    Ok(read_bytes) => {
                        filled_len += read_bytes;

                        if filled_len == buf.len() {
                            break;
                        }

                        if filled_len >= 9 && &buf[2..=3] == b"Er" {
                            let error_code: anyhow::Result<u8> = std::str::from_utf8(&buf[4..=5])
                                .map_err(|e| e.into())
                                .and_then(|err_code_str| {
                                    err_code_str.parse::<u8>().map_err(|e| e.into())
                                });
                            if let Ok(error_code) = error_code {
                                return DistanceReading::Err {
                                    error: DistanceReadingError::new(error_code),
                                    measuring_time: start.elapsed(),
                                };
                            }
                        }
                    }
                    Err(_err) => {
                        //error!(IO distance sensor error)
                        eprintln!("Oopsy io error, {_err:?}");
                        return DistanceReading::Err {
                            error: DistanceReadingError::UnknownError,
                            measuring_time: start.elapsed(),
                        };
                    }
                }
            }

            // `D: 5.614m,1211\r\n`
            let unkown_error = DistanceReading::Err {
                error: DistanceReadingError::UnknownError,
                measuring_time: start.elapsed(),
            };

            let number: u32 = {
                let range = [&buf[2..=3], &buf[5..=7]].concat();
                let string = String::from_utf8(range);
                if string.is_err() {
                    eprintln!("Oopsy, bad string, {string:?}");
                    eprintln!("Sensor message: {buf:?}");
                    return unkown_error;
                }
                let number = string.unwrap().trim().parse();
                if number.is_err() {
                    eprintln!("Oopsy, bad number, {number:?}");
                    eprintln!("Sensor message: {buf:?}");
                    return unkown_error;
                }
                number.unwrap()
            };

            let q_number: u16 = {
                let q_range = &buf[10..=13];
                let string = String::from_utf8(q_range.to_vec());
                if string.is_err() {
                    eprintln!("Oopsy, bad quality string, {string:?}");
                    eprintln!("Sensor message: {buf:?}");
                    return unkown_error;
                }
                let q_number = string.unwrap().parse();
                if q_number.is_err() {
                    eprintln!("Oopsy, bad quality number, {number:?}");
                    eprintln!("Sensor message: {buf:?}");
                    return unkown_error;
                }
                q_number.unwrap()
            };

            DistanceReading::Ok {
                distance: Distance::from_mm(number),
                quality: q_number,
                measuring_time: start.elapsed(),
            }
        }

        /// Make "default" measurement. Sends `b"D"` on serial.
        pub fn read_distance(&mut self) -> DistanceReading {
            self.read_distance_mode(ReadingMode::Default)
        }

        /// Make "fast" measurement. Sends `b"F"` on serial.
        pub fn read_distance_fast(&mut self) -> DistanceReading {
            self.read_distance_mode(ReadingMode::Fast)
        }

        /// Make "slow" measurement. Sends `b"M"` on serial.
        pub fn read_distance_slow(&mut self) -> DistanceReading {
            self.read_distance_mode(ReadingMode::Slow)
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
}

/* Mock of a sensor */
#[cfg(feature = "mock_hardware")]
mod sensor {
    use super::*;
    /// HI50 Distance sensor.
    pub struct DistanceSensor {}

    impl Default for DistanceSensor {
        fn default() -> Self {
            Self::new()
        }
    }

    impl DistanceSensor {
        /// Create new HI50 Distance sensor with hardcoded values.
        pub fn new() -> Self {
            DistanceSensor {}
        }

        /// Enable laser. Sends `b"O"` on serial.
        pub fn start(&mut self) -> Result<()> {
            Ok(())
        }

        /// Make "default" measurement. Sends `b"D"` on serial.
        pub fn read_distance(&mut self) -> DistanceReading {
            let start = Instant::now();

            DistanceReading::Ok {
                distance: Distance::from_mm(123456),
                quality: 42,
                measuring_time: start.elapsed(),
            }
        }

        /// Make "fast" measurement. Sends `b"F"` on serial.
        pub fn read_distance_fast(&mut self) -> DistanceReading {
            let start = Instant::now();

            DistanceReading::Ok {
                distance: Distance::from_mm(41414),
                quality: 12341,
                measuring_time: start.elapsed(),
            }
        }

        pub fn read_distance_mode(&mut self, mode: ReadingMode) -> DistanceReading {
            let start = Instant::now();

            DistanceReading::Ok {
                distance: Distance::from_mm(41414),
                quality: 12341,
                measuring_time: start.elapsed(),
            }
        }

        /// Close laser. Sends `b"C"` on serial.
        pub fn stop(&mut self) -> Result<()> {
            Ok(())
        }
    }
}
