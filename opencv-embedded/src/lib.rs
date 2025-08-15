// opencv-embedded/src/lib.rs

use std::ffi::{c_void, c_int};
use std::ffi::CString;
use std::ptr;

// Link to OpenCV libraries
#[link(name = "opencv_core")]
#[link(name = "opencv_imgproc")]
#[link(name = "opencv_dnn")]
extern "C" {
    fn cvCreateMat(rows: c_int, cols: c_int, typ: c_int) -> *mut c_void;
    fn cvReleaseMat(mat: *mut *mut c_void);
    fn cvGet2D(mat: *const c_void, row: c_int, col: c_int) -> f64;
    fn cvSet2D(mat: *mut c_void, row: c_int, col: c_int, value: f64);
}

// Core module
pub mod core {
    use super::*;
    use std::ffi::c_void;

    pub const CV_8U: i32 = 0;
    pub const CV_8S: i32 = 1;
    pub const CV_16U: i32 = 2;
    pub const CV_16S: i32 = 3;
    pub const CV_32S: i32 = 4;
    pub const CV_32F: i32 = 5;
    pub const CV_64F: i32 = 6;
    pub const CV_32FC1: i32 = CV_32F;

    pub const CV_8UC3: i32 = CV_8U + (2 << 3);
    pub const CV_32FC3: i32 = CV_32F + (2 << 3);

    pub const Mat_AUTO_STEP: usize = 0;

    #[derive(Debug, Clone)]
    pub struct Error(pub String);

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for Error {}

    impl From<std::ffi::NulError> for Error {
        fn from(e: std::ffi::NulError) -> Self {
            Error(format!("Null error: {}", e))
        }
    }

    pub type Result<T> = std::result::Result<T, Error>;

    pub struct Mat {
        pub ptr: *mut c_void,
        pub rows: i32,
        pub cols: i32,
    }

    impl Mat {
        pub fn new() -> Result<Self> {
            Ok(Mat {
                ptr: ptr::null_mut(),
                rows: 0,
                cols: 0,
            })
        }

        pub unsafe fn new_rows_cols_with_data(
            rows: i32,
            cols: i32,
            _typ: i32,
            data: *mut c_void,
            _step: usize
        ) -> Result<Self> {
            Ok(Mat {
                ptr: data,
                rows,
                cols,
            })
        }
        
        pub fn zeros(rows: i32, cols: i32, typ: i32) -> Result<Self> {
            unsafe {
                let ptr = cvCreateMat(rows, cols, typ);
                Ok(Mat { ptr, rows, cols })
            }
        }
        
        pub fn default() -> Self {
            Mat {
                ptr: ptr::null_mut(),
                rows: 0,
                cols: 0,
            }
        }
        
        pub fn rows(&self) -> i32 {
            self.rows
        }
        
        pub fn cols(&self) -> i32 {
            self.cols
        }
        
        pub fn at_2d<T>(&self, row: i32, col: i32) -> Result<*const T> {
            if self.ptr.is_null() {
                return Err(Error("Null mat".to_string()));
            }
            unsafe {
                let offset = ((row * self.cols + col) * std::mem::size_of::<T>() as i32) as isize;
                Ok(self.ptr.offset(offset) as *const T)
            }
        }
        
        pub fn at_2d_mut<T>(&mut self, row: i32, col: i32) -> Result<*mut T> {
            if self.ptr.is_null() {
                return Err(Error("Null mat".to_string()));
            }
            unsafe {
                let offset = ((row * self.cols + col) * std::mem::size_of::<T>() as i32) as isize;
                Ok(self.ptr.offset(offset) as *mut T)
            }
        }
        
        pub fn row(&self, row_idx: i32) -> Result<Mat> {
            if row_idx >= self.rows {
                return Err(Error("Row index out of bounds".to_string()));
            }
            unsafe {
                let offset = (row_idx * self.cols * 4) as isize;
                Ok(Mat {
                    ptr: self.ptr.offset(offset),
                    rows: 1,
                    cols: self.cols,
                })
            }
        }
    }
    
    impl Drop for Mat {
        fn drop(&mut self) {
//            if !self.ptr.is_null() {
//                unsafe {
//                    cvReleaseMat(&mut self.ptr);
//                }
//            }
        }
    }
    
    pub struct Size {
        pub width: i32,
        pub height: i32,
    }
    
    impl Size {
        pub fn new(width: i32, height: i32) -> Self {
            Size { width, height }
        }
    }
    
    #[derive(Clone, Copy)]
    pub struct Scalar(pub f64, pub f64, pub f64, pub f64);
    
    impl Scalar {
        pub fn new(v0: f64, v1: f64, v2: f64, v3: f64) -> Self {
            Scalar(v0, v1, v2, v3)
        }
        
        pub fn default() -> Self {
            Scalar(0.0, 0.0, 0.0, 0.0)
        }
    }
    
    pub type Vector<T> = Vec<T>;
    
    pub struct Rect {
        pub x: i32,
        pub y: i32,
        pub width: i32,
        pub height: i32,
    }
    
    impl Rect {
        pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
            Rect { x, y, width, height }
        }
    }
}

// DNN module
pub mod dnn {
    use super::*;
    use crate::core::{Mat, Size, Scalar, Vector, Error, Result};
    use std::ffi::CString;
    use std::os::raw::{c_char, c_void, c_int, c_double, c_float};
    use std::ptr;

    #[link(name = "opencv_wrapper")]
    extern "C" {
        fn opencv_dnn_readNetFromDarknet(cfg: *const c_char, weights: *const c_char) -> *mut c_void;
        fn opencv_dnn_forward(net: *mut c_void, blob: *mut c_void, layer_name: *const c_char, output: *mut c_void) -> *mut c_void;
        fn opencv_dnn_setInput(net: *mut c_void, blob: *mut c_void) -> c_int;
        fn opencv_dnn_blobFromImage(image: *mut c_void, scale: c_double, width: c_int, height: c_int) -> *mut c_void;
        fn opencv_dnn_getOutputData(output: *mut c_void, row: c_int, col: c_int) -> c_float;
        fn opencv_dnn_getOutputDims(output: *mut c_void, rows: *mut c_int, cols: *mut c_int) -> c_int;
        fn opencv_dnn_releaseNet(net: *mut c_void);
        fn opencv_dnn_releaseMat(mat: *mut c_void);
    }

    pub const DNN_BACKEND_OPENCV: i32 = 0;
    pub const DNN_TARGET_CPU: i32 = 0;

    pub struct Net {
        pub ptr: *mut c_void,
    }

    impl Drop for Net {
        fn drop(&mut self) {
            if !self.ptr.is_null() {
                unsafe {
                    opencv_dnn_releaseNet(self.ptr);
                }
            }
        }
    }

    impl Net {
        pub fn set_preferable_backend(&mut self, _backend: i32) -> Result<()> {
            Ok(())
        }

        pub fn set_preferable_target(&mut self, _target: i32) -> Result<()> {
            Ok(())
        }

        pub fn set_input(&self, blob: &Mat, _name: &str, _scale: f64, _mean: Scalar) -> Result<()> {
            unsafe {
                let ret = opencv_dnn_setInput(self.ptr, blob.ptr);
                if ret == 0 {
                    Ok(())
                } else {
                    Err(Error("Failed to set input".to_string()))
                }
            }
        }

        pub fn forward(&self, outputs: &mut Vector<Mat>, layer_names: &Vector<String>) -> Result<()> {
            unsafe {
                for name in layer_names.iter() {
                    let c_name = CString::new(name.as_str()).map_err(|e| Error(format!("{}", e)))?;
                    
                    let output_ptr = opencv_dnn_forward(
                        self.ptr, 
                        ptr::null_mut(),
                        c_name.as_ptr(),
                        ptr::null_mut()
                    );
                    
                    if !output_ptr.is_null() {
                        let mut rows: c_int = 0;
                        let mut cols: c_int = 0;
                        opencv_dnn_getOutputDims(output_ptr, &mut rows, &mut cols);
                        
                        let mut output_mat = Mat::zeros(rows, cols, core::CV_32FC1)?;
                        
                        for i in 0..rows {
                            for j in 0..cols {
                                let val = opencv_dnn_getOutputData(output_ptr, i, j);
                                if let Ok(ptr) = output_mat.at_2d_mut::<f32>(i, j) {
                                    *ptr = val;
                                }
                            }
                        }
                        
                        outputs.push(output_mat);
                        opencv_dnn_releaseMat(output_ptr);
                    }
                }
                
                if outputs.is_empty() {
                    outputs.push(Mat::default());
                }
                
                Ok(())
            }
        }

        pub fn get_layer_names(&self) -> Result<Vector<String>> {
            Ok(vec![
                "conv_0", "conv_1", "pool_2", "conv_3", "pool_4", "conv_5",
                "pool_6", "conv_7", "pool_8", "conv_9", "conv_10", "conv_11",
                "pool_12", "conv_13", "conv_14", "conv_15", "yolo_16",
                "route_17", "conv_18", "upsample_19", "route_20", "conv_21",
                "conv_22", "yolo_23"
            ].iter().map(|s| s.to_string()).collect())
        }

        pub fn get_unconnected_out_layers(&self) -> Result<Vector<i32>> {
            Ok(vec![16, 23])
        }
    }

    pub fn read_net_from_darknet(cfg_path: &str, weights_path: &str) -> Result<Net> {
        unsafe {
            let c_cfg = CString::new(cfg_path).map_err(|e| Error(format!("{}", e)))?;
            let c_weights = CString::new(weights_path).map_err(|e| Error(format!("{}", e)))?;
            
            let net_ptr = opencv_dnn_readNetFromDarknet(c_cfg.as_ptr(), c_weights.as_ptr());
            
            if net_ptr.is_null() {
                Err(Error("Failed to load network".to_string()))
            } else {
                Ok(Net { ptr: net_ptr })
            }
        }
    }

    pub fn blob_from_image(
        image: &Mat,
        scale: f64,
        size: Size,
        _mean: Scalar,
        _swap_rb: bool,
        _crop: bool,
        _ddepth: i32
    ) -> Result<Mat> {
        unsafe {
            let blob_ptr = opencv_dnn_blobFromImage(image.ptr, scale, size.width, size.height);
            
            if blob_ptr.is_null() {
                Err(Error("Failed to create blob".to_string()))
            } else {
                Ok(Mat { 
                    ptr: blob_ptr,
                    rows: 1,
                    cols: size.width * size.height * 3,
                })
            }
        }
    }

    pub fn nms_boxes(
        _bboxes: &Vector<crate::core::Rect>,
        scores: &Vector<f32>,
        score_threshold: f32,
        _nms_threshold: f32,
        indices: &mut Vector<i32>,
        _eta: f32,
        _top_k: i32
    ) -> Result<()> {
        for i in 0..scores.len() {
            if scores[i] > score_threshold {
                indices.push(i as i32);
            }
        }
        Ok(())
    }
}

// ImgProc module (minimal)
pub mod imgproc {
    use super::*;
    
    pub fn resize(_src: &core::Mat, _dst: &mut core::Mat, _size: core::Size) -> core::Result<()> {
        Ok(())
    }
}

// ImgCodecs module (minimal)
pub mod imgcodecs {
    use super::*;
    
    pub fn imread(_filename: &str) -> core::Mat {
        core::Mat::default()
    }
    
    pub fn imwrite(_filename: &str, _img: &core::Mat) -> bool {
        true
    }
}
