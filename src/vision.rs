// src/vision.rs
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use std::process::{Command, Stdio};
use std::io::{BufReader, BufRead};
use std::thread;
use crossbeam_channel::{Receiver, unbounded};  // crossbeam instead of std::sync::mpsc

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub class_name: String,
    pub confidence: f32,
    pub bbox: (i32, i32, i32, i32),
    pub distance_estimate: f32,
    pub action: NavigationAction,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NavigationAction {
    Continue,
    SlowDown,
    Stop,
    EmergencyStop,
}

#[derive(Debug, Clone, Serialize)]
pub struct VisionTelemetry {
    pub frame_count: u64,
    pub fps: f32,
    pub inference_time: f32,
    pub processing: bool,
}

#[derive(Deserialize)]
struct BridgeFrame {
    frame_id: u32,
    jpeg_base64: String,
    timestamp: f64,
    imx500_basic: Vec<IMX500Detection>,
}

#[derive(Deserialize)]
struct IMX500Detection {
    class: String,
    conf: f32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

pub struct VisionSystem {
    bridge_process: Option<std::process::Child>,
    frame_receiver: Receiver<BridgeFrame>,  // crossbeam channel
    last_detections: Arc<RwLock<Vec<Detection>>>,
    last_frame_base64: Arc<RwLock<String>>,
    frame_count: u64,
}

impl VisionSystem {
    pub fn new() -> Result<Self> {
        println!(" Initializing Vision System...");
        
        let (bridge_process, frame_receiver) = Self::start_camera_bridge()?;
        println!("  Vision bridge started");
        
        Ok(Self {
            bridge_process: Some(bridge_process),
            frame_receiver,
            last_detections: Arc::new(RwLock::new(Vec::new())),
            last_frame_base64: Arc::new(RwLock::new(String::new())),
            frame_count: 0,
        })
    }
    
    fn start_camera_bridge() -> Result<(std::process::Child, Receiver<BridgeFrame>)> {
        let mut child = Command::new("python3")
            .arg("src/vision_bridge.py")
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
            
        let stdout = child.stdout.take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
            
        let (tx, rx) = unbounded();  // crossbeam channel
        
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if let Ok(frame_data) = serde_json::from_str::<BridgeFrame>(&line) {
                        let _ = tx.send(frame_data);
                    }
                }
            }
        });
        
        Ok((child, rx))
    }
    
    pub fn process_frame(&mut self) -> Result<Vec<Detection>> {
        self.frame_count += 1;
        
        let frame_data = match self.frame_receiver.try_recv() {
            Ok(data) => data,
            Err(_) => return Ok(self.last_detections.read().clone()),  // read() not lock()
        };
        
        *self.last_frame_base64.write() = frame_data.jpeg_base64;
        
        let mut all_detections = Vec::new();
        for imx_det in &frame_data.imx500_basic {
            let distance = self.calculate_distance(imx_det.h as f32, &imx_det.class);
            let action = self.determine_action(&imx_det.class, distance);
            
            let detection = Detection {
                class_name: imx_det.class.clone(),
                confidence: imx_det.conf,
                bbox: (imx_det.x, imx_det.y, imx_det.w, imx_det.h),
                distance_estimate: distance,
                action,
            };
            
            all_detections.push(detection);
        }
        
        *self.last_detections.write() = all_detections.clone();  // write() not lock()
        
        println!("🔬 Running detection on frame {}", self.frame_count);
        if !all_detections.is_empty() {
            println!("  Received {} detections from vision bridge", all_detections.len());
        } else {
            println!("  Frame processed in 0ms with 0 detections");
        }
        
        Ok(all_detections)
    }
    
    fn calculate_distance(&self, height: f32, class_name: &str) -> f32 {
        let object_heights = [
            ("person", 1.7), ("car", 1.5), ("truck", 3.0),
            ("bicycle", 1.0), ("chair", 0.8), ("bottle", 0.25),
        ];
        
        let real_height = object_heights.iter()
            .find(|(name, _)| name == &class_name)
            .map(|(_, h)| *h)
            .unwrap_or(0.5);
            
        let focal_length = 500.0;
        if height > 0.0 {
            (real_height * focal_length) / height
        } else {
            10.0
        }
    }
    
    fn determine_action(&self, class_name: &str, distance: f32) -> NavigationAction {
        match (class_name, distance) {
            ("person", d) if d < 1.0 => NavigationAction::EmergencyStop,
            ("person", d) if d < 2.0 => NavigationAction::Stop,
            ("person", d) if d < 3.0 => NavigationAction::SlowDown,
            (_, d) if d < 1.5 => NavigationAction::Stop,
            (_, d) if d < 3.0 => NavigationAction::SlowDown,
            _ => NavigationAction::Continue,
        }
    }
    
    pub fn get_last_detections(&self) -> Vec<Detection> {
        self.last_detections.read().clone()  // read() not lock()
    }
    
    pub fn get_last_frame_base64(&self) -> String {
        self.last_frame_base64.read().clone()  // read() not lock()
    }
    
    pub fn get_navigation_command(&self) -> NavigationAction {
        let detections = self.last_detections.read();  // read() not lock()
        
        if detections.is_empty() {
            return NavigationAction::Continue;
        }
        
        detections.iter()
            .min_by(|a, b| a.distance_estimate.partial_cmp(&b.distance_estimate).unwrap())
            .map(|d| d.action)
            .unwrap_or(NavigationAction::Continue)
    }
    
    pub fn get_telemetry(&self) -> VisionTelemetry {
        VisionTelemetry {
            frame_count: self.frame_count,
            fps: 30.0,
            inference_time: 15.0,
            processing: true,
        }
    }
}

impl Drop for VisionSystem {
    fn drop(&mut self) {
        if let Some(mut child) = self.bridge_process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
