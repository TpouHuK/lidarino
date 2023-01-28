use lidarino::mcp23s17::*;
use std::io;
use std::io::Write;

fn main() {
    let mcp23s17_controller = Mcp23s17Controller::new();
    let c = mcp23s17_controller;
    let stdin = io::stdin();
    let mut input_buf = String::with_capacity(100);

    println!("USAGE: [pin_number][t|f], for e.x.: '0f' or '7t' '5t'");
    loop {
        print!("[MANUAL_PIN_CONTROL]-> ");
        io::stdout().flush().unwrap();

        stdin.read_line(&mut input_buf).unwrap();
        let cmd = input_buf.trim();
        let pin_num: u8 = cmd[0..1].parse().unwrap();
        let pin_value: bool = &cmd[1..2] == "t";
        c.get_pin(pin_num).set_level(pin_value);
        /*
        match cmd {
            "0t" => { c.get_pin(0).set_high(); }
            "1t" => { c.get_pin(1).set_high(); }
            "2t" => { c.get_pin(2).set_high(); }
            "3t" => { c.get_pin(3).set_high(); }
            "4t" => { c.get_pin(4).set_high(); }
            "5t" => { c.get_pin(5).set_high(); }
            "6t" => { c.get_pin(6).set_high(); }
            "7t" => { c.get_pin(7).set_high(); }

            "0f" => { c.get_pin(0).set_low(); }
            "1f" => { c.get_pin(1).set_low(); }
            "2f" => { c.get_pin(2).set_low(); }
            "3f" => { c.get_pin(3).set_low(); }
            "4f" => { c.get_pin(4).set_low(); }
            "5f" => { c.get_pin(5).set_low(); }
            "6f" => { c.get_pin(6).set_low(); }
            "7f" => { c.get_pin(7).set_low(); }
            _ => { println!("Invalid input")}
        }
    }*/
}
