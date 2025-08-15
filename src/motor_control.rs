// src/motor_control.rs
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorStatus {
    pub speed: f32,
    pub heading: f32,
    pub enabled: bool,
}

pub struct MotorController {
    speed: f32,
    heading: f32,
    enabled: bool,
}

impl MotorController {
    pub fn new() -> Result<Self> {
        println!("🚗 Initializing Motor Controller...");
        Ok(Self {
            speed: 0.0,
            heading: 0.0,
            enabled: false,
        })
    }
    
    pub fn move_forward(&mut self, speed: f32) {
        self.speed = speed.clamp(0.0, 1.0);
        println!("➡️ Moving forward at speed {:.1}", self.speed);
    }
    
    pub fn move_backward(&mut self, speed: f32) {
        self.speed = -speed.clamp(0.0, 1.0);
        println!("⬅️ Moving backward at speed {:.1}", self.speed);
    }
    
    pub fn turn_left(&mut self, angle: f32) {
        self.heading -= angle;
        println!("↪️ Turning left by {:.1}°", angle);
    }
    
    pub fn turn_right(&mut self, angle: f32) {
        self.heading += angle;
        println!("↩️ Turning right by {:.1}°", angle);
    }
    
    pub fn stop(&mut self) {
        self.speed = 0.0;
        println!("⏹️ Stopped");
    }
    
    pub fn emergency_stop(&mut self) {
        self.speed = 0.0;
        self.enabled = false;
        println!("🛑 EMERGENCY STOP!");
    }
    
    pub fn get_speed(&self) -> f32 {
        self.speed
    }
    
    pub fn get_heading(&self) -> f32 {
        self.heading
    }
    
    pub fn get_status(&self) -> MotorStatus {
        MotorStatus {
            speed: self.speed,
            heading: self.heading,
            enabled: self.enabled,
        }
    }
}
