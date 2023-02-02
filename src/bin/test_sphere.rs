use lidarino::sphere;

fn main() {
    let points = lidarino::sphere::generate_points(100, 1.0, 0.4, 1.0, 0, 360);
    for point in points {
        println!("{} {} {}", point.x, point.y, point.z);
        //let (phi, theta) = point.as_phi_theta();
        //println!("{} {}", phi.to_degrees(), theta.to_degrees());
    }
}
