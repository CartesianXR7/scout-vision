fn main() {
    // Link to system OpenCV libraries
    println!("cargo:rustc-link-lib=opencv_core");
    println!("cargo:rustc-link-lib=opencv_dnn");
    println!("cargo:rustc-link-lib=opencv_imgproc");
    println!("cargo:rustc-link-lib=opencv_imgcodecs");
    
    // Link to our custom wrapper
    println!("cargo:rustc-link-lib=opencv_wrapper");

    // Tell cargo where to find OpenCV and our wrapper
    println!("cargo:rustc-link-search=/usr/lib/aarch64-linux-gnu");
    println!("cargo:rustc-link-search=/usr/local/lib");
}
