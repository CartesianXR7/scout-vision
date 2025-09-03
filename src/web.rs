// src/web.rs
use warp::Filter;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use futures_util::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use image::{RgbImage, Rgb, DynamicImage};
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;
use base64;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::os::unix::process::ExitStatusExt;

use crate::vision::{VisionSystem, Detection, NavigationAction, VisionTelemetry};
use crate::pathfinding::PathPlanner;
use crate::motor_control::MotorController;

#[derive(Serialize)]
struct FrameData {
    image: String, 
    detections: Vec<Detection>,
    telemetry: VisionTelemetry,
    path_status: PathStatus,
    navigation: NavigationInfo,
    timestamp: u64,
}

#[derive(Serialize)]
struct PathStatus {
    status: String,  
    color: String, 
    obstacles: usize,
}

#[derive(Serialize)]
struct NavigationInfo {
    action: String,
    speed: f32,
    heading: f32,
    distance_to_goal: Option<f32>,
}

pub struct WebServer {
    vision: Arc<RwLock<VisionSystem>>,
    path_planner: Arc<RwLock<PathPlanner>>,
    motor_controller: Arc<RwLock<MotorController>>,
    clients: Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<warp::ws::Message>>>>,
    next_client_id: Arc<RwLock<usize>>,
}

impl WebServer {
    pub fn new(
        vision: Arc<RwLock<VisionSystem>>,
        path_planner: Arc<RwLock<PathPlanner>>,
        motor_controller: Arc<RwLock<MotorController>>,
    ) -> Self {
        Self {
            vision,
            path_planner,
            motor_controller,
            clients: Arc::new(RwLock::new(HashMap::new())),
            next_client_id: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn run(self: Arc<Self>) {
        let static_files = warp::fs::dir("static");

        let ws_route = warp::path("ws")
            .and(warp::ws())
            .map({
                let server = self.clone();
                move |ws: warp::ws::Ws| {
                    let server = server.clone();
                    ws.on_upgrade(move |websocket| {
                        let server = server.clone();
                        async move {
                            server.handle_websocket(websocket).await
                        }
                    })
                }
            });

        let routes = static_files.or(ws_route);

        self.clone().start_frame_broadcaster();

        println!(" Web server ready at http://0.0.0.0:8080");
        warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
    }

    async fn handle_websocket(&self, ws: warp::ws::WebSocket) {
        let (mut ws_tx, mut ws_rx) = ws.split();
        let (tx, rx) = mpsc::unbounded_channel();

        let client_id = {
            let mut next_id = self.next_client_id.write();
            let id = *next_id;
            *next_id += 1;
            id
        };

        self.clients.write().insert(client_id, tx);

        let mut rx = UnboundedReceiverStream::new(rx);
        tokio::spawn(async move {
            while let Some(msg) = rx.next().await {
                if ws_tx.send(msg).await.is_err() {
                    break;
                }
            }
        });

        while let Some(msg) = ws_rx.next().await {
            if let Ok(msg) = msg {
            } else {
                break;
            }
        }

        self.clients.write().remove(&client_id);
    }

    fn start_frame_broadcaster(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                let frame_data = {
                    let vision = self.vision.read();
                    let detections = vision.get_last_detections();
                    let image_base64 = vision.get_last_frame_base64();
                    
                    serde_json::json!({
                        "image": image_base64,
                        "detections": detections,
                        "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
                        "status": "success",
                        "detection_count": detections.len(),
                    })
                };
                
                if let Ok(json) = serde_json::to_string(&frame_data) {
                    let msg = warp::ws::Message::text(json);
                    let clients = self.clients.read();
                    for tx in clients.values() {
                        let _ = tx.send(msg.clone());
                    }
                }
                
                tokio::time::sleep(Duration::from_millis(33)).await;
            }
        });
    }

    async fn create_frame_data(&self) -> FrameData {
        let (image_base64, detections, telemetry, nav_action) = {
            let vision = self.vision.read();
            (
                vision.get_last_frame_base64(),
                vision.get_last_detections(),
                vision.get_telemetry(),
                vision.get_navigation_command()
            )
        };

        let path_status = if detections.is_empty() {
            PathStatus {
                status: "CLEAR".to_string(),
                color: "green".to_string(),
                obstacles: 0,
            }
        } else {
            let closest = &detections[0];
            let (status, color) = match nav_action {
                NavigationAction::EmergencyStop => ("BLOCKED", "red"),
                NavigationAction::Stop => ("BLOCKED", "red"),
                NavigationAction::SlowDown => ("CAUTION", "yellow"),
                NavigationAction::Continue => ("CLEAR", "green"),
            };
            
            PathStatus {
                status: status.to_string(),
                color: color.to_string(),
                obstacles: detections.len(),
            }
        };

        let navigation = NavigationInfo {
            action: format!("{:?}", nav_action),
            speed: match nav_action {
                NavigationAction::EmergencyStop => 0.0,
                NavigationAction::Stop => 0.0,
                NavigationAction::SlowDown => 0.3,
                NavigationAction::Continue => 0.5,
            },
            heading: self.motor_controller.read().get_heading(),
            distance_to_goal: self.path_planner.read().get_distance_to_goal(),
        };

        FrameData {
            image: image_base64,
            detections,
            telemetry,
            path_status,
            navigation,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
        }
    }
}

use serde_json::json;
