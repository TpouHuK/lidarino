# MPU
- [ ] Config files.
- [ ] Make MPU calibration (saving/loading it from file).
	- [ ] Calibrate MPU gyro.
	- [ ] Calibrate MPU magnetometer.
	- [ ] Calibrate MPU accelerometer.

- [ ] Initialize magwick with correct initial state.
- [ ] Implement MPU controller.

# LowPriority
- [ ] Mcp23s17 controller drop.

# Done
- [x] Mocking hardware
- [x] Fix empty `loop` and wasted CPU cycles in motor::MotorController, once motors are proven to be working.
- [x] Add choice between reading modes for `DistanceController`
- [x] Add ability to create scans
- [x] Implement distance sensor error handling and a propper controller.
