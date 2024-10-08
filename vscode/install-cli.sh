#!/usr/bin/env bash

# A script to download the CLI binary for the platform
# and put it in the `cli` subdir.

# Detect the operating system
OS=$(uname -s)
ARCH=$(uname -m)

# Function to determine the latest version if not specified
get_latest_version() {
    curl --silent "https://api.github.com/repos/stencila/stencila/releases/latest" |
    grep '"tag_name":' |
    sed 's/.*"tag_name": *"\([^"]*\)".*/\1/'
}

# Set the version to install (default to latest)
VERSION=${1:-$(get_latest_version)}

echo "Installing Stencila $VERSION for $OS $ARCH"

# Determine the appropriate file name based on OS and architecture
case "$OS" in
    Linux*)
        OS_TYPE="linux"
        if [ "$ARCH" = "x86_64" ]; then
            FILE="cli-${VERSION}-x86_64-unknown-linux-gnu.tar.gz"
        else
            echo "Unsupported architecture: $ARCH"
            exit 1
        fi
        ;;
    Darwin*)
        OS_TYPE="darwin"
        if [ "$ARCH" = "arm64" ]; then
            FILE="cli-${VERSION}-aarch64-apple-darwin.tar.gz"
        elif [ "$ARCH" = "x86_64" ]; then
            FILE="cli-${VERSION}-x86_64-apple-darwin.tar.gz"
        else
            echo "Unsupported architecture: $ARCH"
            exit 1
        fi
        ;;
    CYGWIN*|MINGW*|MSYS*)
        OS_TYPE="windows"
        if [ "$ARCH" = "x86_64" ]; then
            FILE="cli-${VERSION}-x86_64-pc-windows-msvc.zip"
        else
            echo "Unsupported architecture: $ARCH"
            exit 1
        fi
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

# Construct the download URL
URL="https://github.com/stencila/stencila/releases/download/${VERSION}/${FILE}"

# Download the file
echo "Downloading $URL..."
curl -L -o "$FILE" "$URL"

# Verify download success
if [ $? -ne 0 ]; then
    echo "Download failed. Please check your internet connection and the version number."
    exit 1
fi

# Extract the file
echo "Extracting $FILE..."
if [[ "$FILE" == *.tar.gz ]]; then
    tar -xzf "$FILE"
    if [ $? -ne 0 ]; then
        echo "Extraction failed. Please ensure you have 'tar' installed."
        exit 1
    fi
elif [[ "$FILE" == *.zip ]]; then
    unzip -o "$FILE"
    if [ $? -ne 0 ]; then
        echo "Extraction failed. Please ensure you have 'unzip' installed."
        exit 1
    fi
else
    echo "Unknown file format: $FILE"
    exit 1
fi

# Move the extract `cli-*` folder to `cli`
rm -rf cli
mv cli-${VERSION}-*/ cli/

# Cleanup the downloaded file
rm "$FILE"
