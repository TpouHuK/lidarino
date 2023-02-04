use lidarino::sphere::*;
use std::time::Duration;

fn main() {
    let points = lidarino::sphere::generate_points(1000, 1.0, 0.4, 1.0, 0, 360);
    let waypoints: Vec<Waypoint> = points.into_iter().map(|p| p.into()).collect();
    let waypoints = optimize_path(waypoints, Duration::from_secs(10));
    for waypoint in waypoints {
        println!("{} {}", waypoint.yaw, waypoint.pitch);
    }
    //let waypoints = nearest_neightbour_solve(waypoints);
}
