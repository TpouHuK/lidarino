use lidarino::sphere::*;

fn main() {
    let points = lidarino::sphere::generate_points(100, 1.0, 0.4, 1.0, 0, 360);
    let waypoints: Vec<Waypoint> = points.into_iter().map(|p| p.into()).collect();
    let waypoints = nearest_neightbour_solve(waypoints);
}
