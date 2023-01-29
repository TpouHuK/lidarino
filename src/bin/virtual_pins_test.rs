use lidarino::mcp23s17::*;
use rppal::gpio::Level;
use std::io;
use std::io::Write;

fn main() {
    let mcp23s17_controller = Mcp23s17Controller::new();
    let c = mcp23s17_controller;
    let stdin = io::stdin();
    let mut input_buf = String::with_capacity(100);

    println!("USAGE: [pin_number][t|f], for e.x.: '0f' or '7t' '5t'");
    println!("Crashes on incorrect input :-)");
    loop {
        print!("[MANUAL MCP23S13 PIN CONTROL]-> ");

        io::stdout().flush().unwrap();
        stdin.read_line(&mut input_buf).unwrap();

        let cmd = input_buf.trim();

        let pin_num: u8 = cmd[0..1].parse().unwrap();
        let pin_value: Level = (&cmd[1..2] == "t").into();
        c.output_pin(pin_num).write(pin_value);
    }
}
