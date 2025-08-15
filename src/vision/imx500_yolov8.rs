// src/vision/imx500_yolov8.rs - COMPLETE FILE with Python service integration
use anyhow::Result;
use std::process::Command;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub class_name: String,
    pub confidence: f32,
    pub bbox: (i32, i32, i32, i32),
    pub distance_estimate: f32,
}

#[derive(Deserialize)]
struct PythonDetectionResult {
    status: String,
    detections: Vec<PythonDetection>,
    detection_count: usize,
    inference_time: Option<f32>,
}

#[derive(Deserialize)]
struct PythonDetection {
    class_name: String,
    confidence: f32,
    bbox: Vec<i32>,
    distance: f32,
}

pub struct IMX500YoloV8 {
    detections: Arc<RwLock<Vec<Detection>>>,
}

impl IMX500YoloV8 {
    pub fn new() -> Result<Self> {
        println!("🎥 Initializing IMX500 with YOLOv8...");

        // Test that the model loads (keep this check)
        let test = Command::new("rpicam-still")
            .args(&[
                "--width", "640",
                "--height", "480",
                "--rotation", "180",
                "--post-process-file", "/usr/share/rpi-camera-assets/imx500_yolov8.json",
                "--nopreview",
                "--immediate",
                "-o", "/dev/null",
                "--verbose", "2"
            ])
            .output()?;

        if !test.status.success() {
            println!("⚠️ Warning: {}", String::from_utf8_lossy(&test.stderr));
        }

        Ok(Self {
            detections: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub fn capture_and_detect(&self) -> Result<Vec<Detection>> {
        println!("📸 Running YOLOv8 detection...");

        // Use the Python service that actually works!
        let output = Command::new("python3")
            .arg("src/imx500_fast_detector.py")
            .output()?;

        if output.status.success() {
            let json_str = String::from_utf8_lossy(&output.stdout);
            
            // Try to parse the JSON response
            if let Ok(result) = serde_json::from_str::<PythonDetectionResult>(&json_str) {
                if result.status == "success" && !result.detections.is_empty() {
                    let detections: Vec<Detection> = result.detections.iter().map(|d| {
                        Detection {
                            class_name: d.class_name.clone(),
                            confidence: d.confidence,
                            bbox: (d.bbox[0], d.bbox[1], d.bbox[2], d.bbox[3]),
                            distance_estimate: d.distance,
                        }
                    }).collect();
                    
                    println!("  ✅ Detected {} objects", detections.len());
                    for det in &detections {
                        println!("    - {} ({}%) at distance ~{:.1}m", 
                                det.class_name, 
                                (det.confidence * 100.0) as i32,
                                det.distance_estimate);
                    }
                    
                    *self.detections.write() = detections.clone();
                    return Ok(detections);
                }
            } else {
                // If JSON parsing failed, log the error for debugging
                println!("  ⚠️ Failed to parse detection JSON: {}", json_str);
            }
        } else {
            println!("  ⚠️ Python detection service failed: {}", 
                    String::from_utf8_lossy(&output.stderr));
        }

        println!("  ℹ️ No objects detected in frame");
        *self.detections.write() = Vec::new();
        Ok(Vec::new())
    }

    // Keep all the helper functions even though they're not used right now
    fn parse_real_detection(line: &str) -> Option<Detection> {
        // Parse ACTUAL detection output - adjust based on real rpicam output format

        // Example parsing for format: "Object detected: person confidence: 0.85 bbox: [100,200,50,100]"
        if let Some(class_start) = line.find("Object detected:") {
            let rest = &line[class_start + 16..];
            let parts: Vec<&str> = rest.split_whitespace().collect();

            if parts.len() >= 5 {
                let class_name = parts[0].to_string();

                // Parse confidence
                let confidence = if let Some(conf_idx) = parts.iter().position(|&x| x == "confidence:") {
                    parts.get(conf_idx + 1)
                        .and_then(|s| s.parse::<f32>().ok())
                        .unwrap_or(0.0)
                } else {
                    0.0
                };

                // Parse bbox [x,y,w,h]
                let bbox = if let Some(bbox_idx) = parts.iter().position(|&x| x == "bbox:") {
                    if let Some(bbox_str) = parts.get(bbox_idx + 1) {
                        Self::parse_bbox(bbox_str).unwrap_or((0, 0, 0, 0))
                    } else {
                        (0, 0, 0, 0)
                    }
                } else {
                    (0, 0, 0, 0)
                };

                // Calculate REAL distance based on object height
                let distance = Self::calculate_real_distance(bbox.3 as f32, &class_name);

                return Some(Detection {
                    class_name,
                    confidence,
                    bbox,
                    distance_estimate: distance,
                });
            }
        }

        None
    }

    fn parse_bbox(bbox_str: &str) -> Option<(i32, i32, i32, i32)> {
        // Parse [x,y,w,h] format
        let cleaned = bbox_str.trim_matches(|c| c == '[' || c == ']');
        let parts: Vec<&str> = cleaned.split(',').collect();

        if parts.len() == 4 {
            let x = parts[0].parse().ok()?;
            let y = parts[1].parse().ok()?;
            let w = parts[2].parse().ok()?;
            let h = parts[3].parse().ok()?;
            Some((x, y, w, h))
        } else {
            None
        }
    }

    fn calculate_real_distance(bbox_height: f32, class_name: &str) -> f32 {
        // REAL distance calculation based on known object sizes
        // Camera FOV and focal length for IMX500
        let focal_length_pixels = 500.0; // Calibrate this for your camera

        // Real-world heights in meters
        let real_height = match class_name {
            "person" => 1.7,
            "car" => 1.5,
            "truck" | "bus" => 3.0,
            "bicycle" => 1.0,
            "chair" => 0.8,
            "dog" => 0.5,
            "cat" => 0.25,
            _ => 0.5, // Default assumption
        };

        if bbox_height > 0.0 {
            (focal_length_pixels * real_height) / bbox_height
        } else {
            10.0 // Far away if no height
        }
    }
}
