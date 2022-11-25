use std::time::{Duration};
use std::thread;
use std::sync::atomic::{AtomicU32, AtomicI32, AtomicBool, Ordering};
use std::sync::{Arc};

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

pub struct StepMotorController {
    pub cur_pos: Arc<AtomicI32>,
    pub tgt_pos: Arc<AtomicI32>,
    pub step_delay_ms: Arc<AtomicU32>,

    thread_handle: Option<thread::JoinHandle<()>>,
    kill_switch: Arc<AtomicBool>
    // condvar + atomicBool(was_changed) for perfomance ?
}

fn control_loop(mut motor: StepMotor, cur_pos: Arc<AtomicI32>, tgt_pos: Arc<AtomicI32>, step_delay_ms: Arc<AtomicU32>, kill_switch: Arc<AtomicBool>) {
    loop {
        if kill_switch.load(Ordering::Relaxed) { break; }

        let diff = tgt_pos.load(Ordering::Relaxed) - cur_pos.load(Ordering::Relaxed);
        if diff > 0 {
            motor.do_full_step(StepDirection::Forward);
            cur_pos.fetch_add(1, Ordering::Relaxed);
            thread::sleep(Duration::from_millis(step_delay_ms.load(Ordering::Relaxed) as u64));
        } else if diff < 0 {
            motor.do_full_step(StepDirection::Backward);
            cur_pos.fetch_add(-1, Ordering::Relaxed);
            thread::sleep(Duration::from_millis(step_delay_ms.load(Ordering::Relaxed) as u64));
        } else {
            // Condvar.wait()
        }
    }
}

impl StepMotorController {
    pub fn new(motor: StepMotor, step_delay_ms: u32) -> Self {
        let cur_pos = Arc::new(AtomicI32::new(0));
        let cur_pos2 = cur_pos.clone();

        let tgt_pos = Arc::new(AtomicI32::new(0));
        let tgt_pos2 = tgt_pos.clone();

        let step_delay_ms: Arc<AtomicU32> = Arc::new(AtomicU32::new(step_delay_ms));
        let step_delay_ms2 = step_delay_ms.clone();
        
        let kill_switch = Arc::new(AtomicBool::new(false));
        let kill_switch2 = kill_switch.clone();

        let thread_handle = thread::spawn(move || {
            control_loop(motor, cur_pos2, tgt_pos2, step_delay_ms2, kill_switch2)
        });
        let thread_handle = Some(thread_handle);

        return StepMotorController { cur_pos, tgt_pos, step_delay_ms, thread_handle, kill_switch}
    }

    pub fn set_pos(&self, pos: i32) {
        self.tgt_pos.store(pos, Ordering::Relaxed);
    }

    pub fn reset(&self) {
        self.tgt_pos.store(0, Ordering::Relaxed);
        self.cur_pos.store(0, Ordering::Relaxed);
    }

    pub fn stop(&self) {
        self.tgt_pos.store(self.cur_pos.load(Ordering::Relaxed), Ordering::Relaxed);
    }

    pub fn set_step_delay_ms(&self, step_delay_ms: u32) {
        self.step_delay_ms.store(step_delay_ms, Ordering::Relaxed);
    }

    pub fn is_stopped(&self) -> bool {
        self.tgt_pos.load(Ordering::Relaxed) == self.cur_pos.load(Ordering::Relaxed)
    }

    pub fn wait_stop(&self) {
        loop { if self.is_stopped() { break; } }
    }

    pub fn move_on(&self, delta_pos: i32) {
        self.tgt_pos.store(self.cur_pos.load(Ordering::Relaxed) + delta_pos, Ordering::Relaxed);
    }
}

impl Drop for StepMotorController {
    fn drop(&mut self){
        self.kill_switch.store(true, Ordering::Relaxed);
        if let Some(thread_handle) = self.thread_handle.take() {
            thread_handle.join().expect("Control thread did not panic"); // Maybe fuck it who cares anyway
        }
    }
}
