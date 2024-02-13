#!/bin/bash

# Script to download the ONNX runtime lib on Linux
# Used on CI for compiling for tests. 

# Define the URL and target directory
url="https://github.com/microsoft/onnxruntime/releases/download/v1.17.0/onnxruntime-linux-x64-1.17.0.tgz"
target_dir="/usr/local"

# Download the file
wget -q "$url" -O onnxruntime.tgz

# Check if the download was successful
if [ $? -ne 0 ]; then
    echo "Download failed. Please check the URL and try again."
    exit 1
fi

# Extract the contents
tar -xzf onnxruntime.tgz

# Assuming the structure of the tarball, adjust as necessary
# Copy the relevant directories to /usr/local
sudo cp -r onnxruntime-linux-x64-1.17.0/include "$target_dir"
sudo cp -r onnxruntime-linux-x64-1.17.0/lib "$target_dir"

# Run ldconfig on the library directory to update the shared library cache
sudo ldconfig "$target_dir/lib"

echo "ONNX Runtime installation completed successfully."
