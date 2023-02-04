use std::f32::consts::{PI, TAU};
use core::ops::Range;
use std::time::Duration;

pub struct Point {
    pub x: f32, pub y: f32, pub z: f32,
}

impl Point {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Point {x, y, z}
    }

    pub fn as_phi_theta(&self) -> (f32, f32) {
        let (x, y, z) = (self.x, self.y, self.z);
        let phi = y.atan2(x);
        let theta = (z / (x*x + y*y + z*z).sqrt()).acos();
        (phi, theta)
    }

    pub fn as_pitch_yaw(&self) -> (i32, i32) {
        let (phi, theta) = self.as_phi_theta();
        let pitch = (theta / PI * 2000.0).round() as i32 - 2000;
        let yaw = (phi / PI * 2000.0).round() as i32 - 4000;
        (pitch, yaw)
    }
}


pub fn generate_points(n: u32, radius: f32, minimum: f32, maximum: f32, angle_start: u32, angle_range: u32) -> Vec<Point> {
let phi: f32 = PI * (3.0 - 5f32.sqrt());
    (1..n).map(|index| {
        let index = index as f32;
        let y = ((index / (n - 1) as f32) * (maximum - minimum) + minimum) * 2.0 - 1.0;
        let mut theta = phi * index;
        if angle_start != 0 || angle_range != 360 {
            theta %= TAU;
            theta = theta * (angle_range as f32).to_radians() / TAU + (angle_start as f32).to_radians();
        }

        let r_y = (1.0 - y*y).sqrt();
        let x = theta.cos() * r_y;
        let z = theta.sin() * r_y;

        let (x, y, z) = (x, z, y);
        Point::new(x * radius, y * radius, z * radius)
    }).collect()
}

#[derive(Clone, Copy, Debug)]
pub struct Waypoint {
    pub pitch: i32, pub yaw: i32,
}

impl Waypoint {
    fn manhattan_distance(&self, other: &Self) -> u32 {
        ((self.pitch - other.pitch).abs() + (self.yaw - other.yaw).abs()) as u32
    }
}

impl From<Point> for Waypoint {
    fn from(point: Point) -> Self {
        let (pitch, yaw) = point.as_pitch_yaw();
        Waypoint {pitch, yaw}
    }
}

impl tsp_rs::Metrizable for Waypoint {
    fn cost(&self, other: &Self) -> f64 {
        self.manhattan_distance(other) as f64
    }
}

pub fn optimize_path(path: Vec<Waypoint>, duration: Duration) -> Vec<Waypoint> {
    let mut tour = tsp_rs::Tour::new();
    tour.path = path;

    tour.optimize_nn();
    tour.optimize_kopt(duration);
    
    tour.path
}
