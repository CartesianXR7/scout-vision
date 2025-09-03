#!/usr/bin/env python3
# src/vision_bridge.py 
import os
import sys
import json
import subprocess
import base64
import signal
import time
import re
import threading

class VisionBridge:
    def __init__(self):
        self.running = True
        self.current_frame_detections = []
        self.detection_lock = threading.Lock()
        signal.signal(signal.SIGTERM, self.signal_handler)
        signal.signal(signal.SIGINT, self.signal_handler)
        
        self.coco_classes = [
            "person", "bicycle", "car", "motorcycle", "airplane", "bus", "train", "truck",
            "boat", "traffic light", "fire hydrant", "stop sign", "parking meter", "bench",
            "bird", "cat", "dog", "horse", "sheep", "cow", "elephant", "bear", "zebra",
            "giraffe", "backpack", "umbrella", "handbag", "tie", "suitcase", "frisbee",
            "skis", "snowboard", "sports ball", "kite", "baseball bat", "baseball glove",
            "skateboard", "surfboard", "tennis racket", "bottle", "wine glass", "cup",
            "fork", "knife", "spoon", "bowl", "banana", "apple", "sandwich", "orange",
            "broccoli", "carrot", "hot dog", "pizza", "donut", "cake", "chair", "couch",
            "potted plant", "bed", "dining table", "toilet", "tv", "laptop", "mouse",
            "remote", "keyboard", "cell phone", "microwave", "oven", "toaster", "sink",
            "refrigerator", "book", "clock", "vase", "scissors", "teddy bear", "hair drier",
            "toothbrush"
        ]
        
        self.min_x_raw = float('inf')
        self.max_x_raw = 0
        self.min_y_raw = float('inf')
        self.max_y_raw = 0
        
    def signal_handler(self, sig, frame):
        self.running = False
    
    def run(self):
        """Stream camera with IMX500 NPU detections"""
        cmd = [
            'rpicam-vid',
            '--width', '640',
            '--height', '480',
            '--framerate', '30',
            '--timeout', '0',
            '--rotation', '180',
            '--codec', 'mjpeg',
            '--post-process-file', '/usr/share/rpi-camera-assets/imx500_yolov8.json',
            '--verbose', '2',
            '-o', '-'
        ]
        
        process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            bufsize=65536
        )
        
        sys.stderr.write("Started rpicam-vid with IMX500 NPU YOLOv8\n")
        sys.stderr.flush()
        
        def parse_stderr():
            while self.running:
                line = process.stderr.readline()
                if not line:
                    break
                line = line.decode('utf-8', errors='ignore')
                
                num_match = re.search(r'Number of objects detected:\s*(\d+)', line)
                if num_match:
                    num = int(num_match.group(1))
                    with self.detection_lock:
                        self.current_frame_detections = []
                    continue
                
                det_pattern = r'\[(\d+)\]\s*:\s*(\w+)\[?(\d*)\]?\s*\(([0-9.]+)\)\s*@\s*(\d+),(\d+)\s*(\d+)x(\d+)'
                match = re.search(det_pattern, line)
                if match:
                    class_str = match.group(2)
                    
                    if class_str.isdigit():
                        class_id = int(class_str)
                        class_name = self.coco_classes[class_id] if class_id < len(self.coco_classes) else "object"
                    else:
                        class_name = class_str
                        try:
                            class_id = self.coco_classes.index(class_name)
                        except ValueError:
                            class_id = 0
                    
                    center_x_raw = int(match.group(5))
                    center_y_raw = int(match.group(6))
                    
                    self.min_x_raw = min(self.min_x_raw, center_x_raw)
                    self.max_x_raw = max(self.max_x_raw, center_x_raw)
                    self.min_y_raw = min(self.min_y_raw, center_y_raw)
                    self.max_y_raw = max(self.max_y_raw, center_y_raw)
                    
                    # The IMX500 seems to use a much larger coordinate space
                    # Based on the data, it looks like it might be using 12288x9216 (12K x 9K)
                    # or some other high resolution coordinate system
                    
                    # Let's try mapping from the observed range to 640x480
                    # Assuming the IMX500 uses a coordinate space of approximately 12288x9216
                    # (based on observed values like 174208 / 16 ≈ 10888)
                    
                    # Method 1: Direct scaling from assumed IMX500 resolution
                    IMX500_WIDTH = 12288  # Estimated based on max observed values
                    IMX500_HEIGHT = 9216  # Estimated based on max observed values
                    
                    center_x = int((center_x_raw / 16) * 640 / IMX500_WIDTH)
                    center_y = int((center_y_raw / 16) * 480 / IMX500_HEIGHT)
                    
                    # Alternative Method 2: Dynamic scaling based on observed range
                    # Uncomment to try this approach instead
                    # if self.max_x_raw > 0 and self.max_y_raw > 0:
                    #     center_x = int((center_x_raw - self.min_x_raw) * 640 / (self.max_x_raw - self.min_x_raw))
                    #     center_y = int((center_y_raw - self.min_y_raw) * 480 / (self.max_y_raw - self.min_y_raw))
                    # else:
                    #     continue
                    
                    center_x = max(0, min(center_x, 639))
                    center_y = max(0, min(center_y, 479))
                    
                    conf = float(match.group(4))
                    
                    if class_name == "person":
                        base_h = int(100 + conf * 60) 
                        base_w = int(base_h * 0.4)  
                    elif class_name in ["car", "truck", "bus"]:
                        base_h = int(60 + conf * 40)
                        base_w = int(base_h * 1.5)    
                    else:
                        base_h = int(50 + conf * 50)
                        base_w = base_h
                    
                    x = center_x - base_w // 2
                    y = center_y - base_h // 2
                    
                    x = max(0, min(x, 640 - base_w))
                    y = max(0, min(y, 480 - base_h))
                    
                    det = {
                        "class": class_name,
                        "class_id": class_id,
                        "conf": conf,
                        "x": x,
                        "y": y,
                        "w": base_w,
                        "h": base_h,
                        "center_x": center_x,
                        "center_y": center_y
                    }
                    
                    with self.detection_lock:
                        self.current_frame_detections.append(det)
                    
                    sys.stderr.write(f"✓ Detected: {class_name} ({conf:.2f}) at ({center_x},{center_y}) - Box: {base_w}x{base_h} at ({x},{y})\n")
                    sys.stderr.write(f"  Raw range: X[{self.min_x_raw}-{self.max_x_raw}] Y[{self.min_y_raw}-{self.max_y_raw}]\n")
                    sys.stderr.flush()
        
        stderr_thread = threading.Thread(target=parse_stderr, daemon=True)
        stderr_thread.start()
        
        buffer = b''
        frame_id = 0
        
        while self.running:
            chunk = process.stdout.read(4096)
            if not chunk:
                break
                
            buffer += chunk
            
            while True:
                start = buffer.find(b'\xff\xd8')
                if start == -1:
                    break
                    
                end = buffer.find(b'\xff\xd9', start + 2)
                if end == -1:
                    break
                    
                jpeg = buffer[start:end + 2]
                buffer = buffer[end + 2:]
                frame_id += 1
                
                with self.detection_lock:
                    detections_copy = self.current_frame_detections.copy()
                
                output = {
                    "frame_id": frame_id,
                    "jpeg_base64": base64.b64encode(jpeg).decode('utf-8'),
                    "timestamp": time.time(),
                    "imx500_basic": detections_copy
                }
                
                print(json.dumps(output))
                sys.stdout.flush()
        
        process.terminate()

if __name__ == "__main__":
    bridge = VisionBridge()
    bridge.run()
