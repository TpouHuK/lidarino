use lidarino::hardware::distance::DistanceReading;
use lidarino::hardware::{DISTANCE_CONTROLLER, PITCH_CONTROLLER, YAW_CONTROLLER};
use serde::{Deserialize, Serialize};
use serde_json::json;
use warp::Filter;

fn main() {
    env_logger::init();
    start_http();
    unreachable!();
}

fn measure_distance() -> warp::reply::Json {
    let reading = DISTANCE_CONTROLLER.get_measurement();
    let reply = match reading {
        DistanceReading::Ok {
            distance,
            quality,
            measuring_time,
        } => {
            json!({
                "distance_mm": distance.as_mm(),
                "quality": quality,
                "measuring_time_ms": measuring_time.as_millis(),
            })
        }
        _ => {
            json!({
                "err": "some error idk",
            })
        }
    };

    warp::reply::json(&reply)
}

fn send_current_state() -> warp::reply::Json {
    let yaw = YAW_CONTROLLER.get_current_pos();
    let pitch = PITCH_CONTROLLER.get_current_pos();

    let last_measurement = DISTANCE_CONTROLLER.get_last_measurement();
    let (distance, quality) = match last_measurement {
        DistanceReading::Ok {
            distance, quality, ..
        } => (distance.as_mm(), quality),
        _ => (0, 0),
    };

    let reply = json!({
        "yaw": yaw,
        "pitch": pitch,
        "prev_dist_mm": distance,
        "prev_quality": quality,
    });

    warp::reply::json(&reply)
}

#[derive(Serialize, Deserialize, Debug)]
struct SetPosition {
    yaw: Option<i32>,
    pitch: Option<i32>,
}

fn set_position(cmd: SetPosition) -> warp::reply::Json {
    println!("{cmd:?}");
    let reply = json!("Ok");

    if let Some(yaw) = cmd.yaw {
        YAW_CONTROLLER.set_target_pos(yaw);
    }

    if let Some(pitch) = cmd.pitch {
        PITCH_CONTROLLER.set_target_pos(pitch);
    }

    warp::reply::json(&reply)
}

#[tokio::main(worker_threads = 1)]
async fn start_http() {
    use warp::http::Method;
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "Access-Control-Allow-Headers",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
            "Origin",
            "Accept",
            "Accept",
            "X-Requested-With",
            "Content-Type",
        ])
        .allow_methods(&[
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
            Method::HEAD,
        ]);

    let command = warp::post()
        .and(warp::path!("position"))
        .and(warp::filters::body::json())
        .map(set_position);

    let status = warp::get()
        .and(warp::path!("status"))
        .map(send_current_state);

    let measure_distance = warp::post()
        .and(warp::path!("measure_distance"))
        .map(measure_distance);

    let tree = command.or(status).or(measure_distance).with(cors);

    warp::serve(tree).run(([0, 0, 0, 0], 8000)).await;
}
