//! Managing 4-phase unipolar stepper motor.
//! # Example
//! ```
//! let motor = StepMotor::new(pins);
//! let step_delay_ms: u32 = 5;
//! let controller = StepMotorController::new(motor, step_delay_ms);
//!
//! controller.set_pos(100);
//! ```

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::mcp23s17::*;

/// Current phase of a stepper motor.
#[derive(Clone, Copy, Debug)]
enum MotorPhase {
    /// May be used as an initial startup state.
    Unknown,
    /// Full-step phases are represented as even integers,
    /// and half-step as odd integers.
    OnStep(i8),
}

/// Physical step motor.
pub struct StepMotor<T: OutputPin> {
    /// Current phase of a motor
    state: MotorPhase,
    /// `true` if pins are high according to current [`MotorPhase`], `false` otherwise.
    coils_powered: bool,
    /// Pins for controlling motor coils.
    pins: [T; 4],
}

/// Motor phase shift direction.
#[derive(Clone, Copy)]
pub enum StepDirection {
    Forward = 1,
    Nothing = 0,
    Backward = -1,
}

impl<T: OutputPin> StepMotor<T> {
    /// Create a new stepper motor.
    /// * `pins` - pins for controlling state of motor coils (in the correct order).
    pub fn new(pins: [T; 4]) -> StepMotor<T> {
        StepMotor {
            state: MotorPhase::Unknown,
            coils_powered: false,
            pins,
        }
    }

    /// Make stepper motor go a single full-step phase in chosen direction.
    pub fn full_step(&mut self, dir: StepDirection) {
        match self.state {
            MotorPhase::Unknown => {
                self.state = MotorPhase::OnStep(0);
                self.pins[0].set_high();
                self.pins[1].set_high();
                self.coils_powered = true;
            }

            MotorPhase::OnStep(step) => {
                let full_steps = [
                    [true, true, false, false], // 0
                    [true, true, false, false], // 0
                    [false, true, true, false], // 1
                    [false, true, true, false], // 1
                    [false, false, true, true], // 2
                    [false, false, true, true], // 2
                    [true, false, false, true], // 3
                    [true, false, false, true], // 3
                ];
                let new_step = (step + (dir as i8) * 2).rem_euclid(8);
                self.state = MotorPhase::OnStep(new_step);
                for i in 0..4 {
                    if full_steps[new_step as usize][i] {
                        self.pins[i].set_high();
                    } else {
                        self.pins[i].set_low();
                    }
                }
                self.coils_powered = true;
            }
        }
    }

    /// Make stepper motor go a single half-step phase in chosen direction.
    pub fn half_step(&mut self, dir: StepDirection) {
        match self.state {
            MotorPhase::Unknown => {
                self.state = MotorPhase::OnStep(0);
                self.pins[0].set_high();
                self.pins[1].set_high();
                self.coils_powered = true;
            }

            MotorPhase::OnStep(step) => {
                let half_steps = [
                    [true, true, false, false],  // 0
                    [false, true, false, false], // 0
                    [false, true, true, false],  // 1
                    [false, false, true, false], // 1
                    [false, false, true, true],  // 2
                    [false, false, false, true], // 2
                    [true, false, false, true],  // 3
                    [true, false, false, false], // 3
                ];
                let new_step = (step + (dir as i8) * 2).rem_euclid(8);
                self.state = MotorPhase::OnStep(new_step);
                for i in 0..4 {
                    if half_steps[new_step as usize][i] {
                        self.pins[i].set_high();
                    } else {
                        self.pins[i].set_low();
                    }
                }
                self.coils_powered = true;
            }
        }
    }

    /// Disables all the coils on the motor
    pub fn disable_power(&mut self) {
        self.coils_powered = false;
        for i in 0..4 {
            self.pins[i].set_low();
        }
    }

    /// Enables coils according to last used [`MotorPhase`]
    pub fn enable_power(&mut self) {
        self.coils_powered = true;
        self.full_step(StepDirection::Nothing);
    }
}

impl<T: OutputPin> Drop for StepMotor<T> {
    fn drop(&mut self) {
        for i in 0..4 {
            self.pins[i].set_low();
        }
    }
}

/// Controller for managing stepper motor asynchronously in a separate thread.
///
/// Spawn's a separate thread which reacts to change in atomic variables.
/// Controlled throught writing `tgt_pos` and `cur_pos`. Motor make's steps
/// to match `cur_pos` with `tgt_pos`, with delay equal to `step_delay_ms` milliseconds between
/// each step.
pub struct StepMotorController {
    /// Current position
    pub cur_pos: Arc<AtomicI32>,
    /// Target position
    pub tgt_pos: Arc<AtomicI32>,
    /// Delay between each motor step
    pub step_delay_ms: Arc<AtomicU32>,

    /// Thread handle of a separate thread which manages the motor.
    thread_handle: Option<thread::JoinHandle<()>>,
    /// Variable to signal termination of a thread
    kill_switch: Arc<AtomicBool>,
}

/// Thread for managing a stepper motor.
fn control_loop<T: OutputPin>(
    mut motor: StepMotor<T>,
    cur_pos: Arc<AtomicI32>,
    tgt_pos: Arc<AtomicI32>,
    step_delay_ms: Arc<AtomicU32>,
    kill_switch: Arc<AtomicBool>,
) {
    // TODO: remove CPU consumption on empty cycle
    loop {
        if kill_switch.load(Ordering::Relaxed) {
            break;
        }

        let diff = tgt_pos.load(Ordering::Relaxed) - cur_pos.load(Ordering::Relaxed);
        match diff {
            1.. => {
                // Positive integer
                motor.full_step(StepDirection::Forward);
                cur_pos.fetch_add(1, Ordering::Relaxed);
                thread::sleep(Duration::from_millis(
                    step_delay_ms.load(Ordering::Relaxed) as u64
                ));
            }
            i32::MIN..=-1 => {
                // Negative integer
                motor.full_step(StepDirection::Backward);
                cur_pos.fetch_add(-1, Ordering::Relaxed);
                thread::sleep(Duration::from_millis(
                    step_delay_ms.load(Ordering::Relaxed) as u64
                ));
            }
            0 => {} // Zero
        }
    }
}

impl StepMotorController {
    /// Creates a new [`StepMotorController`].
    /// * `motor`: motor to controll
    /// * `step_delay_ms`: delay in millisecond between each step
    pub fn new<T: OutputPin + Send + 'static>(motor: StepMotor<T>, step_delay_ms: u32) -> Self {
        let cur_pos = Arc::new(AtomicI32::new(0));
        let tgt_pos = Arc::new(AtomicI32::new(0));
        let step_delay_ms: Arc<AtomicU32> = Arc::new(AtomicU32::new(step_delay_ms));
        let kill_switch = Arc::new(AtomicBool::new(false));

        let cur_pos_clone = cur_pos.clone();
        let tgt_pos_clone = tgt_pos.clone();
        let step_delay_ms_clone = step_delay_ms.clone();
        let kill_switch_clone = kill_switch.clone();
        let thread_handle = thread::spawn(move || {
            control_loop(
                motor,
                cur_pos_clone,
                tgt_pos_clone,
                step_delay_ms_clone,
                kill_switch_clone,
            )
        });
        let thread_handle = Some(thread_handle);

        StepMotorController {
            cur_pos,
            tgt_pos,
            step_delay_ms,
            thread_handle,
            kill_switch,
        }
    }

    /// Set desired/target position of a step motor.
    pub fn set_pos(&self, pos: i32) {
        self.tgt_pos.store(pos, Ordering::Relaxed);
    }

    /// Set current position of a step motor as 0.
    pub fn reset(&self) {
        self.tgt_pos.store(0, Ordering::Relaxed);
        self.cur_pos.store(0, Ordering::Relaxed);
    }

    /// Stop motor if it's moving, do nothing otherwise.
    // Sets target position as current, making difference zero and stopping the motor.
    pub fn stop(&self) {
        self.tgt_pos
            .store(self.cur_pos.load(Ordering::Relaxed), Ordering::Relaxed);
    }

    /// Set delay between motor steps.
    pub fn set_step_delay_ms(&self, step_delay_ms: u32) {
        self.step_delay_ms.store(step_delay_ms, Ordering::Relaxed);
    }

    /// Checks if motor is running or not.
    // If target positon and current position are matching, then we conclude that motor is not
    // stepping anywhere.
    pub fn is_stopped(&self) -> bool {
        self.tgt_pos.load(Ordering::Relaxed) == self.cur_pos.load(Ordering::Relaxed)
    }

    /// Blocks current thread untill motor is finished rotating to target position.
    pub fn wait_stop(&self) {
        loop {
            if self.is_stopped() {
                break;
            }
        }
    }

    /// Change target position on `delta_pos` step.
    pub fn move_on(&self, delta_pos: i32) {
        self.tgt_pos.store(
            self.cur_pos.load(Ordering::Relaxed) + delta_pos,
            Ordering::Relaxed,
        );
    }
}

impl Drop for StepMotorController {
    fn drop(&mut self) {
        self.kill_switch.store(true, Ordering::Relaxed);
        if let Some(thread_handle) = self.thread_handle.take() {
            thread_handle.join().expect("Control thread did not panic"); // Maybe fuck it who cares anyway
        }
    }
}

/// Mock of a MotorController, which does nothing and implements all the methods.
pub struct ControllerMock {
    pub cur_pos: Arc<AtomicI32>,
    pub tgt_pos: Arc<AtomicI32>,
    pub step_delay_ms: Arc<AtomicU32>,
}

impl Default for ControllerMock {
    fn default() -> Self {
        Self::new()
    }
}

impl ControllerMock {
    pub fn new() -> Self {
        let cur_pos = Arc::new(AtomicI32::new(0));
        let tgt_pos = Arc::new(AtomicI32::new(0));
        let step_delay_ms: Arc<AtomicU32> = Arc::new(AtomicU32::new(3));
        Self {
            cur_pos,
            tgt_pos,
            step_delay_ms,
        }
    }

    pub fn get_cur_pos(&self) -> i32 {
        self.cur_pos.load(Ordering::Relaxed)
    }

    pub fn get_tgt_pos(&self) -> i32 {
        self.cur_pos.load(Ordering::Relaxed)
    }

    pub fn get_step_delay_ms(&self) -> u32 {
        self.step_delay_ms.load(Ordering::Relaxed)
    }

    pub fn set_pos(&self, pos: i32) {
        self.tgt_pos.store(pos, Ordering::Relaxed);
        self.cur_pos.store(pos, Ordering::Relaxed);
    }

    pub fn reset(&self) {
        self.tgt_pos.store(0, Ordering::Relaxed);
        self.cur_pos.store(0, Ordering::Relaxed);
    }

    pub fn stop(&self) {
        self.tgt_pos
            .store(self.cur_pos.load(Ordering::Relaxed), Ordering::Relaxed);
    }

    pub fn set_step_delay_ms(&self, step_delay_ms: u32) {
        self.step_delay_ms.store(step_delay_ms, Ordering::Relaxed);
    }

    pub fn is_stopped(&self) -> bool {
        self.tgt_pos.load(Ordering::Relaxed) == self.cur_pos.load(Ordering::Relaxed)
    }

    pub fn wait_stop(&self) {
        loop {
            if self.is_stopped() {
                break;
            }
        }
    }

    pub fn move_on(&self, delta_pos: i32) {
        self.tgt_pos.store(
            self.cur_pos.load(Ordering::Relaxed) + delta_pos,
            Ordering::Relaxed,
        );
    }
}
