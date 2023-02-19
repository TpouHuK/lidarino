use std::time::Duration;

use lidarino::hardware::distance::DistanceReading;
use lidarino::hardware::{DISTANCE_CONTROLLER, PITCH_CONTROLLER, YAW_CONTROLLER, ORIENTATION_CONTROLLER};
use serde::{Deserialize, Serialize};
use serde_json::json;
use warp::Filter;
use warp::ws::{WebSocket, Message};
use lidarino::config::{Config, CONFIG_PATH};
use lazy_static::lazy_static;
use std::sync::Mutex;
use lidarino::hardware::mpu::*;
use lidarino::hardware::mpu::OrientationController;

lazy_static! {
    static ref CONFIG: Mutex<Config> = Mutex::new(Config::default());
}

fn init_orientation() {
    let mut orientation_controller = ORIENTATION_CONTROLLER.lock().unwrap();
    if orientation_controller.is_none() {
        let mpu_config = CONFIG.lock().unwrap().mpu_config.unwrap();
        let mpu = Mpu::new(mpu_config);
        let new_c = OrientationController::new(mpu);
        *orientation_controller = Some(new_c);
        println!("Done initialization, pls dont access MPU using other means. FIXME");
    }
}

fn main() {
    println!("WELCOME TO LIDARINO WEB SERVER");
    if CONFIG.lock().unwrap().load_from_file(CONFIG_PATH).is_ok() {
        println!("Succesfully loaded config from \"{CONFIG_PATH}\"");
    } else {
        println!("Failed loading config from \"{CONFIG_PATH}\"");
    };
    init_orientation();
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
use futures_util::{FutureExt, StreamExt, SinkExt};
use tokio::time::sleep;

async fn orientation_connected(ws: WebSocket) {
    let (mut tx, mut rx) = ws.split();
    tokio::task::spawn( async move {
        let mut dt = 0.0f32;
        loop {
            sleep(Duration::from_secs(1) / 60).await;
            let (roll, pitch, yaw) = ORIENTATION_CONTROLLER
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .get_quat()
                .euler_angles();

            let message = Message::text(format!("{roll},{pitch},{yaw}"));
            if let Err(e) = tx.send(message).await {
                break;
            }
        }
    });
    while let Some(result) = rx.next().await {
    };
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

    let orientation_websocket = warp::path("orientation")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(orientation_connected)
        });

    let tree = orientation_websocket.or(command).or(status).or(measure_distance).with(cors);

    warp::serve(tree).run(([0, 0, 0, 0], 8000)).await;
}
