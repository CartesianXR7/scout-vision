// src/main.rs 
use anyhow::Result;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::time::{sleep, Duration};
use std::time::Instant;

mod vision;
mod web;
mod pathfinding;
mod motor_control;

use vision::{VisionSystem, NavigationAction};
use pathfinding::{PathPlanner, NavigationCommand};
use motor_control::MotorController;

#[tokio::main]
async fn main() -> Result<()> {
    println!(" MARS ROVER - RUST POWERED");
    println!(" Pi Zero 2W | IMX500 NPU | YOLOv8");
    println!(" SAFETY-CRITICAL MODE ENABLED\n");

    let vision = Arc::new(RwLock::new(VisionSystem::new()?));
    let path_planner = Arc::new(RwLock::new(PathPlanner::new()));
    let motor_controller = Arc::new(RwLock::new(MotorController::new()?));

    println!(" Starting web server on port 8080");
    let web_server = Arc::new(web::WebServer::new(
        vision.clone(),
        path_planner.clone(),
        motor_controller.clone(),
    ));

    let _web_handle = {
        let server = web_server.clone();
        tokio::spawn(async move {
            server.run().await;
        })
    };

    println!(" Web interface: http://0.0.0.0:8080");
    println!(" All systems initialized");
    println!(" Starting autonomous navigation...\n");

    let mut loop_count = 0u64;
    let mut fps = 0.0f32;
    let mut last_fps_time = Instant::now();
    let mut fps_frame_count= 0;

    loop {
        let frame_start = Instant::now();

        let (detections, nav_action) = {
            let mut vision = vision.write();
            let detections = vision.process_frame()?;
            let action = vision.get_navigation_command();
            (detections, action)
        };

        let nav_command = {
            let mut planner = path_planner.write();
            planner.update_obstacles(&detections);
            planner.get_navigation_command()
        };

        {
            let mut motors = motor_controller.write();

            match nav_action {
                NavigationAction::EmergencyStop => {
                    motors.emergency_stop();
                }
                NavigationAction::Stop => {
                    motors.stop();
                }
                _ => {
                    match nav_command {
                        NavigationCommand::Forward(speed) => {
                            motors.move_forward(speed);
                        }
                        NavigationCommand::TurnLeft(angle) => {
                            motors.turn_left(angle);
                        }
                        NavigationCommand::TurnRight(angle) => {
                            motors.turn_right(angle);
                        }
                        NavigationCommand::Stop => {
                            motors.stop();
                        }
                    }
                }
            }
        }

        fps_frame_count += 1;
        if last_fps_time.elapsed() >= Duration::from_secs(1) {
            fps = fps_frame_count as f32 / last_fps_time.elapsed().as_secs_f32();
            fps_frame_count = 0;
            last_fps_time = Instant::now();
            println!("📊 System FPS: {:.1}", fps);
        }

        let frame_time = frame_start.elapsed();
        if frame_time < Duration::from_millis(33) {  // Target 30 FPS
            sleep(Duration::from_millis(33) - frame_time).await;
        }

        loop_count += 1;

        if loop_count % 100 == 0 {
            println!("🔄 Processed {} frames", loop_count);
        }
    }
}

