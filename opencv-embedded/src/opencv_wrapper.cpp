#include <opencv2/dnn.hpp>
#include <opencv2/opencv.hpp>
#include <iostream>
#include <vector>

extern "C" {
    cv::dnn::Net* global_net = nullptr;
    
    void* opencv_dnn_readNetFromDarknet(const char* cfg, const char* weights) {
        try {
            std::cout << "Loading YOLO from " << cfg << " and " << weights << std::endl;
            cv::dnn::Net net = cv::dnn::readNetFromDarknet(cfg, weights);
            if (net.empty()) {
                std::cout << "Failed to load network!" << std::endl;
                return nullptr;
            }
            
            net.setPreferableBackend(cv::dnn::DNN_BACKEND_OPENCV);
            net.setPreferableTarget(cv::dnn::DNN_TARGET_CPU);
            
            global_net = new cv::dnn::Net(net);
            std::cout << "Network loaded successfully!" << std::endl;
            return global_net;
        } catch (const cv::Exception& e) {
            std::cout << "Error: " << e.what() << std::endl;
            return nullptr;
        }
    }
    
    int opencv_dnn_setInput(void* net_ptr, void* blob_ptr) {
        try {
            if (!net_ptr || !blob_ptr) return -1;
            cv::dnn::Net* net = (cv::dnn::Net*)net_ptr;
            cv::Mat* blob = (cv::Mat*)blob_ptr;
            net->setInput(*blob);
            return 0;
        } catch (...) {
            return -1;
        }
    }
    
    void* opencv_dnn_forward(void* net_ptr, void* blob_ptr, const char* layer_name, void* output_ptr) {
        try {
            if (!net_ptr) {
                std::cout << "Null network pointer!" << std::endl;
                return nullptr;
            }
            
            cv::dnn::Net* net = (cv::dnn::Net*)net_ptr;
            cv::Mat* output = new cv::Mat();
            
            std::vector<cv::String> outNames;
            if (layer_name && strlen(layer_name) > 0) {
                outNames.push_back(layer_name);
            } else {
                outNames = net->getUnconnectedOutLayersNames();
            }
            
            std::vector<cv::Mat> outputs;
            net->forward(outputs, outNames);
            
            if (!outputs.empty()) {
                *output = outputs[0].clone();
                std::cout << "Forward pass output: " << output->rows << "x" << output->cols << std::endl;
            }
            
            return output;
        } catch (const cv::Exception& e) {
            std::cout << "Forward error: " << e.what() << std::endl;
            return nullptr;
        }
    }
    
    void* opencv_dnn_blobFromImage(void* image_ptr, double scale, int width, int height) {
        try {
            cv::Mat image;
            if (image_ptr) {
                unsigned char* data = (unsigned char*)image_ptr;
                cv::Mat temp(480, 640, CV_8UC3, data);
                image = temp.clone();
            } else {
                image = cv::Mat::zeros(480, 640, CV_8UC3);
            }
            
            cv::Mat* blob = new cv::Mat();
            *blob = cv::dnn::blobFromImage(image, scale, cv::Size(width, height), 
                                          cv::Scalar(0,0,0), true, false);
            
            std::cout << "Created blob: " << blob->size[0] << "x" << blob->size[1] 
                      << "x" << blob->size[2] << "x" << blob->size[3] << std::endl;
            
            return blob;
        } catch (const cv::Exception& e) {
            std::cout << "Blob error: " << e.what() << std::endl;
            return nullptr;
        }
    }
    
    float opencv_dnn_getOutputData(void* mat_ptr, int row, int col) {
        if (!mat_ptr) return 0.0f;
        cv::Mat* mat = (cv::Mat*)mat_ptr;
        if (row >= mat->rows || col >= mat->cols) return 0.0f;
        return mat->at<float>(row, col);
    }
    
    int opencv_dnn_getOutputDims(void* mat_ptr, int* rows, int* cols) {
        if (!mat_ptr) {
            *rows = 0;
            *cols = 0;
            return -1;
        }
        cv::Mat* mat = (cv::Mat*)mat_ptr;
        *rows = mat->rows;
        *cols = mat->cols;
        return 0;
    }
    
    void opencv_dnn_releaseNet(void* net_ptr) {
        // Don't delete - we're using a global
    }
    
    void opencv_dnn_releaseMat(void* mat_ptr) {
        if (mat_ptr) {
            cv::Mat* mat = (cv::Mat*)mat_ptr;
            delete mat;
        }
    }
}
