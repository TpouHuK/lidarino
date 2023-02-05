use lidarino::sphere::*;
use std::time::Duration;

fn main() {
    let opts = ScanOptions {
        amount_of_points: 1000,
        pitch_start: 0.0,
        pitch_end: 120.0,
        yaw_start: 180.0 - 90.0,
        yaw_end: 180.0 + 90.0,
    };
    let points = lidarino::sphere::generate_points(opts);
    for p in &points {
        let (pitch, yaw) = p.as_pitch_yaw();
        let p = Point::from_yaw_pitch_distance(yaw, pitch, 100);
        println!("{} {} {}", p.x, p.y, p.z);
    }
    let waypoints: Vec<Waypoint> = points.into_iter().map(|p| p.into()).collect();
    let waypoints = optimize_path(waypoints, Duration::from_secs(1));
    for waypoint in waypoints {
        //println!("{} {}", waypoint.yaw, waypoint.pitch);
    }
    //let waypoints = nearest_neightbour_solve(waypoints);
}
