//! MCP23S17 pin multiplexer.
use rppal_mcp23s17::{ ChipSelect, HardwareAddress, Mcp23s17, SpiBus, SpiMode };
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

#[derive(Debug, Clone)]

/// Thread-safe MCP23S17 pin. 
pub struct VirtualPin {
    pin_num: u8,
    pin_req_tx: Sender<PinChangeRequest>,
}


impl VirtualPin {
    /// Set pin to high.
    pub fn set_high(&self) {
        self.set_level(true);
    }

    /// Set pin to low.
    pub fn set_low(&self) {
        self.set_level(false);
    }

    /// Set pin to high if `high: bool`, set pin to low otherwise.
    pub fn set_level(&self, high: bool) {
        self.pin_req_tx
            .send(PinChangeRequest {
                pin_num: self.pin_num,
                high,
            })
            .expect("controller alive");
    }
}

/// MCP23S17 controller with ability to get thread-safe [`VirtualPin`].
pub struct Mcp23s17Controller {
    pin_req_tx: Sender<PinChangeRequest>,
}

#[derive(Debug, Clone, Copy)]
/// Request to set pin `pin_number` to state `high`.
struct PinChangeRequest {
    pin_num: u8,
    high: bool,
}

/// Main thread for controlling MCP23S17.
fn controller_thread(rx: Receiver<PinChangeRequest>, mcp23s17: Mcp23s17) {
    use rppal_mcp23s17::*;
    let pins: [pin::OutputPin; 8] = core::array::from_fn(|i| {
        mcp23s17
            .get(Port::GpioA, i as u8)
            .unwrap()
            .into_output_pin_low()
            .unwrap()
    });

    // TODO clean loop exit
    loop {
        let msg = rx.recv().unwrap(); 
        let pin_num = msg.pin_num as usize;

        if msg.high {
            pins[pin_num].set_high().unwrap();
        } else {
            pins[pin_num].set_low().unwrap();
        }
    }
}

impl Default for Mcp23s17Controller {
    fn default() -> Self {
        Self::new()
    }
}

impl Mcp23s17Controller {
    /// Create a new MCP23S17 controller instance.
    pub fn new() -> Self {
        let (pin_req_tx, rx) = mpsc::channel();

        let _thread_handle = thread::spawn(move || {
            // Hardcoded values
            let mcp23s17 = Mcp23s17::new(
                HardwareAddress::new(0).expect("Invalid hardware address"),
                SpiBus::Spi0,
                ChipSelect::Cs0,
                100_000,
                SpiMode::Mode0,
            )
            .expect("Failed to create MCP23S17");

            controller_thread(rx, mcp23s17);
        });

        Self { pin_req_tx }
    }

    /// Returns [`VirtualPin`] for MPC23S7.
    /// * `pin_num` - pin number (0 to 7) (on GPIOA)
    /// TODO: add support for pins from 0 to 16
    pub fn output_pin(&self, pin_num: u8) -> VirtualPin {
        assert!((0..8).contains(&pin_num));
        VirtualPin {
            pin_num,
            pin_req_tx: self.pin_req_tx.clone(),
        }
    }
}

impl Drop for Mcp23s17Controller {
    fn drop(&mut self) {
        //TODO
    }
}
