use lidarino::mcp23s17::*;
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
        let pin_value: bool = &cmd[1..2] == "t";

        c.get_pin(pin_num).set_level(pin_value);
    }
}
