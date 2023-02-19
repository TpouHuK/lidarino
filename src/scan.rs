#![allow(clippy::new_without_default)] // TODO remove after finished developing

use crate::shared::*;
use crate::sphere::*;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use crate::hardware::distance::DistanceReading;
use crate::hardware::{
    DISTANCE_CONTROLLER, MPU_CONTROLLER, ORIENTATION_CONTROLLER, PITCH_CONTROLLER, YAW_CONTROLLER,
};

#[derive(Serialize, Deserialize)]
pub struct ScannedCheckpoint {
    x: f32,
    y: f32,
    z: f32,
    waypoint_yaw: i32,
    waypoint_pitch: i32,
    current_yaw: i32,
    current_pitch: i32,
    roll: f32,
    pitch: f32,
    yaw: f32,
    distance: u32,
    quality: u32,
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum ScanState {
    Scanning,
    Paused,
    Dead,
}

impl IsDead for ScanState {
    fn is_dead(&self) -> bool {
        *self == ScanState::Dead
    }
}

struct ScanJobData {
    waypoints: Vec<Waypoint>,
    scanned_points: Vec<ScannedCheckpoint>,
}

impl ScanJobData {
    pub fn new() -> Self {
        ScanJobData {
            waypoints: Vec::new(), 
            scanned_points: Vec::new(), 
        }
    }
}

use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::thread;
use std::time::{Instant, Duration};
use spinners::{Spinner, Spinners};

impl ScanJobData {
    pub fn generate_path(&mut self, opts: ScanOptions) {
        let start = Instant::now();
        let mut sp = Spinner::new(Spinners::Dots9, "Building a path.".into());

        let points = crate::sphere::generate_points(opts);
        let waypoints: Vec<Waypoint> = points.into_iter().map(|p| p.into()).collect();
        self.waypoints = optimize_path(waypoints, Duration::from_secs(30));
        sp.stop_and_persist(
            "✔",
            format!("Done path building in {:?}", start.elapsed()),
            );
    }
}

use std::sync::mpsc;
use std::sync::Arc;

enum ScanJobMsg {
    GeneratePath(ScanOptions),
    StartScan,
    PauseScan,
    SaveFile,
}

pub struct ScanJob {
    data: Arc<Mutex<ScanJobData>>,
    tx: SyncSender<ScanJobMsg>,
}

impl ScanJob {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::sync_channel(1); // FIXME maybe 0?
        let data = Arc::new(Mutex::new(ScanJobData::new()));
        let data_clone = data.clone();
        thread::spawn( move || {
            scan_job(rx, data_clone);
        });
        ScanJob{data, tx}
    }

    pub fn generate_path(&self, opts: ScanOptions) {
        self.tx.send(ScanJobMsg::GeneratePath(opts)).unwrap();
    }

    pub fn start_scan(&self) {
        self.tx.send(ScanJobMsg::StartScan).unwrap();
    }

    pub fn pause_scan(&self) {
        self.tx.send(ScanJobMsg::PauseScan).unwrap();
    }

    pub fn save_file(&self) {
        self.tx.send(ScanJobMsg::SaveFile).unwrap();
    }

    // There are no way to reset scanned points for now
    pub fn reset(&self) {
        todo!()
    }
}

fn scan_job(rx: Receiver<ScanJobMsg>, data: Arc<Mutex<ScanJobData>>) {
    while let Ok(msg) = rx.recv() {
        match msg {
            ScanJobMsg::GeneratePath(opts) => data.lock().unwrap().generate_path(opts),
            ScanJobMsg::StartScan => {
                /* Main scan loop */
                loop {
                    let mut data = data.lock().unwrap();
                    let point_number = data.scanned_points.len();
                    let waypoint = data.waypoints.get(point_number);

                    if let Some(waypoint) = waypoint {
                        if let Ok(msg) = rx.try_recv() {
                            match msg {
                                ScanJobMsg::PauseScan => {
                                    eprintln!("Pausing a scan");
                                    break;
                                }
                                _ => { eprintln!("I got a bullshit request while scanning. ._.")}
                            }
                        }

                        eprintln!("Going to point {point_number}.");
                        YAW_CONTROLLER.set_target_pos(waypoint.yaw);
                        YAW_CONTROLLER.wait_stop();
                        PITCH_CONTROLLER.set_target_pos(waypoint.pitch);
                        PITCH_CONTROLLER.wait_stop();

                        let measurement = DISTANCE_CONTROLLER.get_measurement();
                        match measurement {
                            DistanceReading::Ok {
                                distance,
                                quality,
                                measuring_time,
                            } => {
                                let p =
                                    Point::from_yaw_pitch_distance(waypoint.yaw, waypoint.pitch, distance.as_mm());
                                let (roll, pitch, yaw) = ORIENTATION_CONTROLLER
                                    .lock()
                                    .unwrap()
                                    .as_ref()
                                    .unwrap()
                                    .get_quat()
                                    .euler_angles();
                                let scanned_checkpoint = ScannedCheckpoint {
                                    x: p.x,
                                    y: p.y,
                                    z: p.z,
                                    waypoint_yaw: waypoint.yaw,
                                    waypoint_pitch: waypoint.pitch,
                                    current_yaw: YAW_CONTROLLER.get_current_pos(),
                                    current_pitch: PITCH_CONTROLLER.get_current_pos(),
                                    roll,
                                    pitch,
                                    yaw,
                                    distance: distance.as_mm(),
                                    quality: quality as u32,
                                };
                                data.scanned_points.push(scanned_checkpoint);
                            }
                            DistanceReading::Err {
                                measuring_time,
                                error,
                            } => {
                                eprintln!("Error measuring point. {error:?}");
                            }
                            DistanceReading::NoReading => {
                                unreachable!()
                            }
                        }
                    } else {
                        /* Finished scan */
                        break;
                    }
                }

            },
            ScanJobMsg::PauseScan => { /* Doing nothing, cause this arm will be matched only while in a paused state*/},
            ScanJobMsg::SaveFile => {
                let data = data.lock().unwrap();
                let json_string = serde_json::to_string(&data.scanned_points).unwrap();
                std::fs::write("points.json", json_string).unwrap();
            },

            #[allow(unreachable_patterns)]
            _ => unreachable!()
        }
    }
}
