#![allow(clippy::new_without_default)] // Annoying during developing, TODO fix after

use rppal_mcp23s17::{ChipSelect, HardwareAddress, Level, Mcp23s17, Port, RegisterAddress, SpiBus, SpiMode};
use std::thread;
use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug, Clone)]
pub struct VirtualPin {
    pin_num: u8,
    pin_req_tx: Sender<PinChangeRequest>,
}

impl VirtualPin {
    pub fn set_high(&self) {
        self.set_level(true);
    }

    pub fn set_low(&self) {
        self.set_level(false);
    }

    pub fn set_level(&self, high: bool) {
        self.pin_req_tx.send(PinChangeRequest{ pin_num: self.pin_num, high } ).unwrap();
    }
}

pub struct Mcp23s17Controller {
    pin_req_tx: Sender<PinChangeRequest>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

#[derive(Debug, Clone, Copy)]
struct PinChangeRequest {
    pin_num: u8,
    high: bool,
}

fn controller_thread(rx: Receiver<PinChangeRequest>, mcp23s17: Mcp23s17) {
    use rppal_mcp23s17::*;
    let pins: [pin::OutputPin; 8] = core::array::from_fn(
        |i| mcp23s17.get(Port::GpioA, i as u8).unwrap()
            .into_output_pin_low().unwrap()
    );

    loop {
        let msg = rx.recv().unwrap(); // TODO, exit instead of panic
        let pin_num = msg.pin_num as usize;

        if msg.high {
            pins[pin_num].set_high().unwrap();
        } else {
            pins[pin_num].set_low().unwrap();
        }
    }
}

impl Mcp23s17Controller {
    pub fn new() -> Self {
        // Create an instance of the driver for the device with the hardware address
        // (A2, A1, A0) of 0b000.
        let (pin_req_tx, rx) = mpsc::channel();

        let thread_handle = thread::spawn(move || {
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

        Self { pin_req_tx, thread_handle: Some(thread_handle) }
    }

    pub fn get_pin(&self, pin_num: u8) -> VirtualPin {
        VirtualPin{ pin_num, pin_req_tx: self.pin_req_tx.clone() }
    }
}
