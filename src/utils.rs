//! TODO: This module should be removed and refactored into another modules
pub fn pos_to_xyz(yaw: i32, pitch: i32, distance: u32) -> (f32, f32, f32) {
    let yaw = yaw as f32 / 2000.0 * 90.0 * -1.0;
    let pitch = pitch as f32 / 2000.0 * 90.0;
    let distance = distance as f32 / 1000.0;

    let x = yaw.sin() * pitch.cos() * distance;
    let y = pitch.sin() * distance;
    let z = yaw.cos() * pitch.cos() * distance;

    (x, y, z)
}
