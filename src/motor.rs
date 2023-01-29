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

impl MotorPhase {
    fn phase_num_or_0(&self) -> usize {
        match self {
            MotorPhase::Unknown => { 0 }
            MotorPhase::OnStep(phase) => { *phase as usize }
        }
    }

    fn fullstep_pins(&self) -> &'static [bool; 4] {
        const FULLSTEP_PINS: [[bool; 4]; 8] = [
            [true, true, false, false], // 0 0 
            [true, true, false, false], // 0 1
            [false, true, true, false], // 2 2 
            [false, true, true, false], // 2 3
            [false, false, true, true], // 4 4
            [false, false, true, true], // 4 5
            [true, false, false, true], // 6 6
            [true, false, false, true], // 6 7
        ];
        &FULLSTEP_PINS[self.phase_num_or_0()]
    }

    fn halfstep_pins(&self) -> &'static [bool; 4] {
        const HALFSTEP_PINS: [[bool; 4]; 8] = [
            [true, true, false, false],  // 0
            [false, true, false, false], // 1
            [false, true, true, false],  // 2
            [false, false, true, false], // 3
            [false, false, true, true],  // 4
            [false, false, false, true], // 5
            [true, false, false, true],  // 6
            [true, false, false, false], // 7
        ];
        &HALFSTEP_PINS[self.phase_num_or_0()]
    }

    fn next_fullstep(&mut self){
        *self = match self {
            MotorPhase::Unknown => { MotorPhase::OnStep(0) }
            MotorPhase::OnStep(step) => { MotorPhase::OnStep((*step + 2).rem_euclid(8)) }
        }
    }

    fn prev_fullstep(&mut self){
        *self = match self {
            MotorPhase::Unknown => { MotorPhase::OnStep(0) }
            MotorPhase::OnStep(step) => { MotorPhase::OnStep((*step - 2).rem_euclid(8)) }
        }
    }

    fn init_phase(&mut self) {
        *self = MotorPhase::OnStep(0);
    }

    /// Unimplemented
    fn next_halfstep(&mut self) {
        unimplemented!();
    }

    /// Unimplemented
    fn prev_halfstep(&mut self) {
        unimplemented!();
    }
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

    fn set_pins(&mut self, levels: &[bool; 4]) {
        for (level, pin) in levels.iter().zip(self.pins.iter_mut()) {
            pin.write(*level);
        }
    }

    /// Make stepper motor go a single full-step phase in chosen direction.
    pub fn full_step(&mut self, dir: StepDirection) {
        match dir {
            StepDirection::Forward => { self.state.next_fullstep(); }
            StepDirection::Backward => { self.state.prev_fullstep(); }
            StepDirection::Nothing => { self.state.init_phase(); }
        }
        self.set_pins(self.state.fullstep_pins());
        self.coils_powered = true;
    }

    /// Make stepper motor go a single half-step phase in chosen direction.
    pub fn half_step(&mut self, dir: StepDirection) {
        match dir {
            StepDirection::Forward => { self.state.next_halfstep(); }
            StepDirection::Backward => { self.state.prev_halfstep(); }
            StepDirection::Nothing => { self.state.init_phase(); }
        }
        self.set_pins(self.state.halfstep_pins());
        self.coils_powered = true;
    }

    /// Disables all the coils on the motor
    pub fn disable_power(&mut self) {
        self.coils_powered = false;
        self.set_pins(&[false, false, false, false])
    }

    /// Enables coils according to last used [`MotorPhase`]
    pub fn enable_power(&mut self) {
        self.coils_powered = true;
        self.set_pins(self.state.fullstep_pins());
    }
}

impl<T: OutputPin> Drop for StepMotor<T> {
    fn drop(&mut self) {
        for i in 0..4 {
            self.pins[i].set_low();
        }
    }
}

use std::sync::{Condvar, Mutex};

#[derive(Default, Clone)]
struct ControllerSharedData {
    current_pos: Arc<AtomicI32>,
    target_pos: Arc<AtomicI32>,
    step_delay_ms: Arc<AtomicU32>,
    update_status: Arc<(Mutex<bool>, Condvar)>,
    kill_switch: Arc<AtomicBool>,
}

impl ControllerSharedData {
    fn set_step_delay_ms(&self, step_delay_ms: u32) {
        self.step_delay_ms.store(step_delay_ms, Ordering::Relaxed);
    }

    fn set_current_pos(&self, current_pos: i32) {
        self.current_pos.store(current_pos, Ordering::Relaxed);
        self.notify_update();
    }

    fn set_target_pos(&self, target_pos: i32) {
        self.current_pos.store(target_pos, Ordering::Relaxed);
        self.notify_update();
    }

    fn get_current_pos(&self) -> i32 {
        self.current_pos.load(Ordering::Relaxed)
    }

    fn get_target_pos(&self) -> i32 {
        self.target_pos.load(Ordering::Relaxed)
    }

    fn notify_update(&self) {
        let (lock, cvar) = &*self.update_status;
        let mut update = lock.lock().unwrap();
        *update = true;
        cvar.notify_all();
    }

    fn notify_noupdate(&self) {
        let (lock, cvar) = &*self.update_status;
        let mut update = lock.lock().unwrap();
        *update = false;
        cvar.notify_all();
    }

    fn wait_update(&self) {
        let (lock, cvar) = &*self.update_status;
        let mut update = lock.lock().unwrap();
        while !*update {
            update = cvar.wait(update).unwrap();
        }
        *update = false;
    }

    fn wait_noupdate(&self) {
        let (lock, cvar) = &*self.update_status;
        let mut update = lock.lock().unwrap();
        while *update {
            update = cvar.wait(update).unwrap();
        }
    }

    fn inc_current_pos(&self, value: i32) {
        self.current_pos.fetch_add(value, Ordering::Relaxed);
    }

    fn kill(&self) {
        self.kill_switch.store(true, Ordering::Relaxed);
        self.notify_update();
    }

    fn wait_step_delay(&self) {
        thread::sleep(Duration::from_millis(
            self.step_delay_ms.load(Ordering::Relaxed) as u64,
        ));
    }

    fn is_killed(&self) -> bool {
        self.kill_switch.load(Ordering::Relaxed)
    }
}

/// Controller for managing stepper motor asynchronously in a separate thread.
///
/// Spawn's a separate thread which reacts to change in atomic variables.
/// Controlled throught writing `tgt_pos` and `cur_pos`. Motor make's steps
/// to match `cur_pos` with `tgt_pos`, with delay equal to `step_delay_ms` milliseconds between
/// each step.
pub struct StepMotorController {
    shared: ControllerSharedData,
    /// Thread handle of a control thread which manages the motor.
    thread_handle: Option<thread::JoinHandle<()>>,
}

/// Thread for managing a stepper motor.
fn control_loop<T: OutputPin>(mut motor: StepMotor<T>, shared: ControllerSharedData) {
    loop {
        if shared.is_killed() {
            break;
        }

        let diff = shared.get_target_pos() - shared.get_current_pos();
        match diff.cmp(&0) {
            // Positive integer, step forward
            std::cmp::Ordering::Greater => {
                motor.full_step(StepDirection::Forward);
                shared.inc_current_pos(1);
                shared.wait_step_delay();
            }
            // Negative integer, step backward
            std::cmp::Ordering::Less => {
                motor.full_step(StepDirection::Backward);
                shared.inc_current_pos(-1);
                shared.wait_step_delay();
            }
            // Zero, do nothing
            std::cmp::Ordering::Equal => {
                shared.notify_noupdate();
                shared.wait_update()
            }
        }
    }
}

impl StepMotorController {
    /// Creates a new [`StepMotorController`].
    /// * `motor`: motor to controll
    /// * `step_delay_ms`: delay in millisecond between each step
    pub fn new<T: OutputPin + Send + 'static>(motor: StepMotor<T>, step_delay_ms: u32) -> Self {
        let shared_data: ControllerSharedData = Default::default();
        shared_data.set_step_delay_ms(step_delay_ms);

        let shared_clone = shared_data.clone();
        let thread_handle = thread::spawn(move || control_loop(motor, shared_clone));
        let thread_handle = Some(thread_handle);

        StepMotorController {
            shared: shared_data,
            thread_handle,
        }
    }

    /// Set desired/target position of a step motor.
    #[deprecated(note="Use set_target_pos instead.")]
    pub fn set_pos(&self, pos: i32) {
        self.shared.set_target_pos(pos);
    }

    /// Set current position of a step motor as 0.
    pub fn reset(&self) {
        self.shared.set_target_pos(0);
        self.shared.set_current_pos(0);
    }

    /// Stop motor if it's moving, do nothing otherwise.
    pub fn stop(&self) {
        self.shared.set_target_pos(self.shared.get_current_pos());
    }

    /// Set delay between motor steps.
    pub fn set_step_delay_ms(&self, step_delay_ms: u32) {
        self.shared.set_step_delay_ms(step_delay_ms);
    }

    /// Checks if motor is running or not.
    pub fn is_stopped(&self) -> bool {
        self.shared.get_current_pos() == self.shared.get_target_pos()
    }

    /// Blocks current thread untill motor is finished rotating to target position.
    pub fn wait_stop(&self) {
        self.shared.wait_noupdate();
    }

    pub fn get_target_pos(&self) -> i32 {
        self.shared.get_target_pos()
    }

    pub fn get_current_pos(&self) -> i32 {
        self.shared.get_current_pos()
    }

    pub fn set_target_pos(&self, target_pos: i32) {
        self.shared.set_target_pos(target_pos);
    }

    pub fn set_current_pos(&self, current_pos: i32) {
        self.shared.set_current_pos(current_pos);
    }

    /// Change target position on `delta_pos` step.
    pub fn move_on(&self, delta_pos: i32) {
        self.set_target_pos(self.get_target_pos() + delta_pos);
    }
}

impl Drop for StepMotorController {
    fn drop(&mut self) {
        self.shared.kill();
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

    pub fn get_current_pos(&self) -> i32 {
        self.cur_pos.load(Ordering::Relaxed)
    }

    pub fn get_target_pos(&self) -> i32 {
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
