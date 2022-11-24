use std::time::{Duration};
use rppal::gpio::{Gpio, OutputPin};

#[derive(Clone, Copy, Debug)]
enum MotorState {
    Disabled,
    OnStep(i8),
}

pub struct StepMotor {
    state: MotorState,
    pins: [OutputPin; 4],
}

#[derive(Clone, Copy)]
pub enum StepDirection {
    Forward = 1,
    Backward = -1,
}

impl StepMotor {
    pub fn new(pins: [u8; 4]) -> StepMotor {
        let gpio = Gpio::new().expect("access to gpio exists");
        let dev_pins: [OutputPin; 4] = core::array::from_fn(|i| gpio.get(pins[i]).expect("got_pin").into_output_low() );
        return StepMotor{ state: MotorState::Disabled, pins: dev_pins };
    }

    pub fn do_full_step(&mut self, dir: StepDirection) {
        match self.state {
            MotorState::Disabled => {
                self.state = MotorState::OnStep(0);
                self.pins[0].set_high();
                self.pins[1].set_high();
            },

            MotorState::OnStep(step) => {
                let full_steps = [
                    [true,  true,  false, false], // 0
                    [true,  true,  false, false], // 0
                    [false, true,  true,  false], // 1
                    [false, true,  true,  false], // 1
                    [false, false, true,  true], // 2 
                    [false, false, true,  true], // 2 
                    [true,  false, false, true], // 3
                    [true,  false, false, true], // 3
                ];
                let new_step = (step + (dir as i8)*2).rem_euclid(8);
                self.state = MotorState::OnStep(new_step);
                for i in 0..4 {
                    if full_steps[new_step as usize][i] { self.pins[i].set_high(); } else {self.pins[i].set_low(); }
                }
            }
        }
    }

    pub fn do_half_step(&mut self, dir: StepDirection) {
        match self.state {
            MotorState::Disabled => {
                self.state = MotorState::OnStep(0);
                self.pins[0].set_high();
                self.pins[1].set_high();
            },

            MotorState::OnStep(step) => {
                let half_steps = [
                    [true,  true,  false, false], // 0
                    [false,  true,  false, false], // 0
                    [false, true,  true,  false], // 1
                    [false, false,  true,  false], // 1
                    [false, false, true,  true], // 2 
                    [false, false, false,  true], // 2 
                    [true,  false, false, true], // 3
                    [true,  false, false, false], // 3
                ];
                let new_step = (step + (dir as i8)*2).rem_euclid(8);
                self.state = MotorState::OnStep(new_step);
                for i in 0..4 {
                    if half_steps[new_step as usize][i] { self.pins[i].set_high(); } else {self.pins[i].set_low(); }
                }
            }
        }
    }
}

impl Drop for StepMotor {
    fn drop(&mut self){
        for i in 0..4 {
            self.pins[i].set_low();
        }
    }
}
