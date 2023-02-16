//! Hardcoded hardware items for our "head" module.
use super::distance::*;
use super::mcp23s17::*;
use super::motor::*;
use super::mpu::{Mpu, MpuConfig, OrientationController};
use std::sync::Mutex;

use lazy_static::lazy_static;

const DEFAULT_MOTOR_DELAY_MS: u32 = 4;
const PITCH_PINS: [u8; 4] = [3, 2, 1, 0];
const YAW_PINS: [u8; 4] = [4, 5, 6, 7];

lazy_static! {
    pub static ref MCP23S17: Mcp23s17Controller = Mcp23s17Controller::new();
}

lazy_static! {
    pub static ref YAW_CONTROLLER: StepMotorController = {
        let pins = MCP23S17.step_motor_pins(YAW_PINS);
        StepMotorController::from_pins(pins, DEFAULT_MOTOR_DELAY_MS)
    };
}

lazy_static! {
    pub static ref PITCH_CONTROLLER: StepMotorController = {
        let pins = MCP23S17.step_motor_pins(PITCH_PINS);
        StepMotorController::from_pins(pins, DEFAULT_MOTOR_DELAY_MS)
    };
}

lazy_static! {
    pub static ref DISTANCE_CONTROLLER: DistanceController = {
        let distance_sensor = DistanceSensor::new();
        DistanceController::new(distance_sensor)
    };
}

lazy_static! {
    pub static ref ORIENTATION_CONTROLLER: Mutex<Option<OrientationController>> =
         Mutex::new(None);
}

lazy_static! {
    pub static ref MPU_CONTROLLER: Mutex<Mpu> = Mutex::new(Mpu::new(MpuConfig::default()));
}
