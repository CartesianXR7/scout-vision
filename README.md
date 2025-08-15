# Scout Vision

Real-time computer vision for embedded Rust on Raspberry Pi Zero 2W - developed for the Scout Rust Rover project. This project implements YOLOv8 object detection using custom OpenCV bindings and ONNX Runtime, achieving deterministic performance on severely resource-constrained hardware.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Raspberry%20Pi%20Zero%202W-red)](https://www.raspberrypi.com/products/raspberry-pi-zero-2-w/)

## Key Features

- **YOLOv8 Integration**: ONNX Runtime with Rust bindings
- **OpenCV for Rust**: For better edge detection
- **Live Web Interface**: Real-time video streaming and telemetry at 5-10 FPS
- **Deterministic Real-time Performance**: 45-50ms consistent inference time (no GC pauses)
- **Minimal Memory Footprint**: 45-55MB total RAM usage including model
- **Safety-Critical Design**: Compile-time guarantees for obstacle avoidance

### If needed: 
- [Pre-Compiled Binary] (https://github.com/CartesianXR7/scout-vision/blob/master/target/release/rover)
- [Custom OpenCV Bindings] (https://github.com/CartesianXR7/scout-vision/tree/master/opencv-embedded): Hybrid FFI approach avoiding compilation on constrained microc

## Demo

![Rover Vision Demo](docs/images/demo.gif)

[Indoor Detection](docs/images/indoor-detection.png) | [Outdoor Detection](docs/images/outdoor-detection.png)

## Architecture

This project uses a hybrid architecture combining Rust's safety guarantees with Python's hardware access capabilities:
