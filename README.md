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
- **[Pre-Compiled Release Binary](https://github.com/CartesianXR7/scout-vision/blob/master/target/release/rover)**: Pre-compiled binary for Pi Zero 2W included to avoid slow / difficult compilation
- **[Custom OpenCV Bindings](https://github.com/CartesianXR7/scout-vision/tree/master/opencv-embedded)**: Hybrid FFI approach to avoid resource-constrained compilation

## 📋 Prerequisites

### Hardware Requirements
- Raspberry Pi Zero 2W (512MB RAM minimum)
- Raspberry Pi Camera Module or USB camera
- MicroSD card (16GB minimum)
- 5V 2.5A power supply (recommended: Waveshare UPS HAT for extended operation)

### Software Requirements
- Raspberry Pi OS Lite (64-bit)
- Rust 1.75.0 or later
- Python 3.9+
- OpenCV 4.5+ system libraries

## 🔧 Build Instructions

### Install System Dependencies

On Raspberry Pi OS / Ubuntu 20.04:

```bash
# Update package lists
sudo apt update

# Install build essentials
sudo apt install -y build-essential cmake pkg-config

# Install OpenCV and dependencies
sudo apt install -y libopencv-dev python3-opencv python3-pip

# Install additional required libraries
sudo apt install -y \
    libssl-dev \
    libfreetype6-dev \
    libxkbcommon-dev \
    libudev-dev
```
### Install Rust

```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version
```
### Clone and Build

```bash
# Clone the repository
git clone https://github.com/CartesianXR7/scout-vision.git
cd scout-vision

# Build the project (Warning: Takes 3+ hours on Pi Zero 2W!)
cargo build --release

# For faster development builds (less optimized):
cargo build
```

## Demo

![Rover Vision Demo](docs/images/demo.gif)

[Indoor Detection](docs/images/indoor-detection.png) | [Outdoor Detection](docs/images/outdoor-detection.png)

## Architecture

This project uses a hybrid architecture combining Rust's safety guarantees with Python's hardware access capabilities:
```bash
Camera → Python Bridge → Rust Vision System → Navigation Commands
            ↓                  ↓                      ↓
       MJPEG Stream     Object Detection       Motor Control
            ↓                  ↓                      ↓
       Web Interface     Grid Mapping          Path Planning
```

## Core Technologies

YOLOv8 by Ultralytics: State-of-the-art object detection model

- Jocher, G., Chaurasia, A., & Qiu, J. (2023). Ultralytics YOLO (Version 8.0.0) [Computer software]. [https://github.com/ultralytics/ultralytics]


ONNX Runtime: High-performance inference engine

- Microsoft Corporation. (2018). ONNX Runtime: Optimize and Accelerate Machine Learning Inferencing and Training. [https://onnxruntime.ai/]


OpenCV: Computer vision library

- Bradski, G. (2000). The OpenCV Library. Dr. Dobb's Journal of Software Tools.



## Rust Dependencies

- [ort](https://github.com/pykeio/ort): Rust bindings for ONNX Runtime
- [opencv-rust](https://github.com/twistedfall/opencv-rust): OpenCV bindings for Rust (inspiration for custom bindings)
- [tokio](https://tokio.rs/): Asynchronous runtime for Rust
- [Warp](https://github.com/seanmonstar/warp): Web server framework
- [Parking Lot](https://github.com/Amanieu/parking_lot): Improved synchronization primitives
- [Anyhow](https://github.com/dtolnay/anyhow): Error handling
- [Serde](https://serde.rs/): Serialization framework

**Hardware & Platform**

- [Raspberry Pi Zero 2W](https://www.raspberrypi.com/products/raspberry-pi-zero-2-w/): Target hardware platform
- [4tronix Mars Rover Kit](https://shop.4tronix.co.uk/products/marsrover): Robotics platform

**Related Work & Inspiration**

- [rust-cv](https://github.com/rust-cv/cv): Computer vision algorithms in Rust
- [TheiaSfM](http://theia-sfm.org/): Structure from Motion library (architectural inspiration)
- "Past, Present, and Future of Simultaneous Localization And Mapping: Towards the Robust-Perception Age" - Cadena et al., 2016 - Excellent overview of modern SLAM algorithms

Related Work & Inspiration

rust-cv: Computer vision algorithms in Rust
- "Past, Present, and Future of Simultaneous Localization And Mapping: Towards the Robust-Perception Age" - Cadena et al., 2016 - Excellent overview of modern SLAM algorithms

## Project Structure
```bash
scout-vision/
├── src/                        # Rust source code
│   ├── main.rs                # Entry point
│   ├── vision.rs              # Vision processing system
│   ├── vision_bridge.py       # Python camera interface
│   ├── web.rs                 # Web server & WebSocket
│   ├── motor_control.rs       # Motor control logic
│   ├── pathfinding.rs         # Navigation algorithms
│   └── vision/
│       └── imx500_yolov8.rs   # YOLOv8 implementation
├── opencv-embedded/            # Custom OpenCV FFI bindings
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
├── models/                     # ML models
│   ├── yolov8n.onnx          # YOLOv8 nano model (12.3MB)
│   └── coco.names            # Class labels
├── static/                     # Web interface
│   └── index.html
├── Cargo.toml                 # Rust dependencies
└── Cargo.lock                 # Dependency lock file
```

This project is licensed under the MIT License - see the LICENSE file for details.
