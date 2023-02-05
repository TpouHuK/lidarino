use serde::{Deserialize, Serialize};
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Point { x, y, z }
    }

    pub fn as_phi_theta(&self) -> (f32, f32) {
        let (x, y, z) = (self.x, self.y, self.z);
        let phi = y.atan2(x);
        let theta = (z / (x * x + y * y + z * z).sqrt()).acos();

        let mut phi = phi - PI;
        if phi < -PI {
            phi += TAU
        }
        let theta = PI - theta;
        assert!((0.0..=PI).contains(&theta));

        (phi, theta)
    }

    pub fn as_pitch_yaw(&self) -> (i32, i32) {
        let (phi, theta) = self.as_phi_theta();
        let pitch = (theta / PI * 4000.0).round() as i32;
        let yaw = (phi / PI * 4000.0).round() as i32;

        //let yaw = (yaw - 4000) % 4000;
        (pitch, yaw)
    }

    pub fn from_yaw_pitch_distance(yaw: i32, pitch: i32, distance: u32) -> Point {
        let yaw = yaw as f32 / 4000.0 * PI * -1.0;
        let pitch = pitch as f32 / 4000.0 * PI;
        let distance = distance as f32 / 1000.0;

        let x = yaw.sin() * pitch.sin() * distance;
        let y = yaw.cos() * pitch.sin() * distance;
        let z = pitch.cos() * distance;

        Point { x, y, z }
    }
}

pub struct ScanOptions {
    pub amount_of_points: u32,
    pub pitch_start: f32,
    pub pitch_end: f32,
    pub yaw_start: f32,
    pub yaw_end: f32,
}

impl ScanOptions {
    pub fn n(&self) -> u32 {
        self.amount_of_points + 1
    }

    pub fn radius(&self) -> f32 {
        1.0
    }

    pub fn minimum(&self) -> f32 {
        self.pitch_start / 180.0
    }

    pub fn maximum(&self) -> f32 {
        self.pitch_end / 180.0
    }

    pub fn angle_start(&self) -> u32 {
        self.yaw_start as u32
    }

    pub fn angle_range(&self) -> u32 {
        //self.yaw_end as u32
        (self.yaw_end - self.yaw_start) as u32
    }
}

pub fn generate_points(opts: ScanOptions) -> Vec<Point> {
    let n = opts.n();
    let minimum = opts.minimum();
    let maximum = opts.maximum();
    let angle_start = opts.angle_start();
    let angle_range = opts.angle_range();
    let radius = opts.radius();
    let phi: f32 = PI * (3.0 - 5f32.sqrt());

    (1..n)
        .map(|index| {
            let index = index as f32;
            let y = ((index / (n - 1) as f32) * (maximum - minimum) + minimum) * 2.0 - 1.0;
            let mut theta = phi * index;
            if angle_start != 0 || angle_range != 360 {
                theta %= TAU;
                theta = theta * (angle_range as f32).to_radians() / TAU
                    + (angle_start as f32).to_radians();
            }

            let r_y = (1.0 - y * y).sqrt();
            let x = theta.cos() * r_y;
            let z = theta.sin() * r_y;

            let (x, y, z) = (x, z, y);
            Point::new(x * radius, y * radius, z * radius)
        })
        .collect()
}

#[derive(Clone, Copy, Debug)]
pub struct Waypoint {
    pub pitch: i32,
    pub yaw: i32,
}

impl Waypoint {
    fn manhattan_distance(&self, other: &Self) -> u32 {
        ((self.pitch - other.pitch).abs() + (self.yaw - other.yaw).abs()) as u32
    }
}

impl From<Point> for Waypoint {
    fn from(point: Point) -> Self {
        let (pitch, yaw) = point.as_pitch_yaw();
        Waypoint { pitch, yaw }
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
