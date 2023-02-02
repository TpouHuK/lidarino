use lidarino::sphere;

fn main() {
    let points = lidarino::sphere::generate_points(100, 1.0, 0.5, 0.5, 0, 180);
    for point in points {
        println!("{} {} {}", point.x, point.y, point.z);
        let (phi, theta) = point.get_phi_theta();
        //println!("{} {}", phi.to_degrees(), theta.to_degrees());
    }
}
