// src/pathfinding.rs
use serde::{Deserialize, Serialize};
use crate::vision::Detection;

#[derive(Debug, Clone)]
pub enum NavigationCommand {
    Forward(f32),
    TurnLeft(f32),
    TurnRight(f32),
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathStatus {
    pub has_path: bool,
    pub distance_to_goal: Option<f32>,
    pub obstacles_count: usize,
}

pub struct PathPlanner {
    current_path: Vec<PathPoint>,
    obstacles: Vec<Detection>,
    goal: Option<PathPoint>,
}

impl PathPlanner {
    pub fn new() -> Self {
        Self {
            current_path: Vec::new(),
            obstacles: Vec::new(),
            goal: None,
        }
    }
    
    pub fn update_obstacles(&mut self, detections: &[Detection]) {
        self.obstacles = detections.to_vec();
    }
    
    pub fn get_navigation_command(&self) -> NavigationCommand {
        if !self.obstacles.is_empty() {
            NavigationCommand::Stop
        } else {
            NavigationCommand::Forward(0.5)
        }
    }
    
    pub fn get_current_path(&self) -> Vec<PathPoint> {
        self.current_path.clone()
    }
    
    pub fn get_distance_to_goal(&self) -> Option<f32> {
        self.goal.as_ref().map(|_| 10.0) // Mock distance
    }
    
    pub fn get_status(&self) -> PathStatus {
        PathStatus {
            has_path: !self.current_path.is_empty(),
            distance_to_goal: self.get_distance_to_goal(),
            obstacles_count: self.obstacles.len(),
        }
    }
}
