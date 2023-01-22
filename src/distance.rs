use mio_serial::*;
use std::thread::sleep;
use std::time::Duration;

pub struct DistanceSensor {
    tty_port: Box<dyn SerialPort>,
}

impl DistanceSensor {
     pub fn new() -> Self {
        let tty_port = mio_serial::new("/dev/ttyS0", 19200)
            .timeout(Duration::from_millis(3500))
            .data_bits(DataBits::Eight)
            .open().expect("Failed to open ttyS0 port.");
        DistanceSensor { tty_port }
     }

     pub fn start(&mut self) -> Result<()> {
        self.tty_port.write_all(b"O").expect("enabled laser");
        self.tty_port.flush().expect("enabled laser");
        let mut buf: Vec<u8> = vec![0; 7];
        self.tty_port.read_exact(&mut buf).unwrap();
        assert_eq!(buf, b"O,OK!\r\n");
        Ok(())
     }

     pub fn read_distance(&mut self) -> Result<(u32, u32)> {
        self.tty_port.write_all(b"D").expect("enabled laser");
        self.tty_port.flush().expect("enabled laser");


        // TODO add error handling
        let mut buf: Vec<u8> = vec![0; 16];
        self.tty_port.read_exact(&mut buf)?; // TOOD FIX ERROR
        // 'D: 5.614m,1211\r\n'
        let range = [&buf[3..=3], &buf[5..=7]].concat();
        let string = String::from_utf8(range).unwrap();
        let number:u32 = string.parse().unwrap();

        let q_range = &buf[10..=13];
        let q_string = String::from_utf8(q_range.to_vec()).unwrap();
        let q_number:u32 = q_string.parse().unwrap();

        Ok((number, q_number))
     }

     pub fn read_distance_fast(&mut self) -> Result<(u32, u32)> {
        self.tty_port.write_all(b"F").expect("enabled laser");
        self.tty_port.flush().expect("enabled laser");


        // TODO add error handling
        let mut buf: Vec<u8> = vec![0; 16];
        self.tty_port.read_exact(&mut buf)?; // TOOD FIX ERROR
        // 'D: 5.614m,1211\r\n'
        let range = [&buf[3..=3], &buf[5..=7]].concat();
        let string = String::from_utf8(range).unwrap();
        let number:u32 = string.parse().unwrap();

        let q_range = &buf[10..=13];
        let q_string = String::from_utf8(q_range.to_vec()).unwrap();
        let q_number:u32 = q_string.parse().unwrap();

        Ok((number, q_number))
     }

     pub fn stop(&mut self) -> Result<()> {
        self.tty_port.write_all(b"C").expect("enabled laser");
        self.tty_port.flush().expect("enabled laser");
        let mut buf: Vec<u8> = vec![0; 7];
        self.tty_port.read_exact(&mut buf).unwrap();
        assert_eq!(buf, b"C,OK!\r\n");
        Ok(())
     }
}
