#!/usr/bin/env bash

# A script to download and install the latest (or specified) version
# of the Stencila CLI on MacOS or Linux
#  
# First arg:  desired version e.g. v0.42.1
#             defaults to latest
# Second arg: desired install location, e.g. /bin
#             defaults to /usr/local/bin on MacOD, to ~/.local/bin on Linux

set -e

OS=$(uname)
if [[ "${OS}" == "Linux" || "${OS}" == "Darwin" ]]; then
    case "${OS}" in
        'Linux')
            TARGET_TRIPLE="x86_64-unknown-linux-gnu"
            VERSION="${1:-latest}"
            if [ "$VERSION" == "latest" ]; then
                VERSION=$(curl --silent "https://api.github.com/repos/stencila/stencila/releases/latest" | grep -Po '"tag_name": "\K.*?(?=")')
            fi
            INSTALL_PATH="${2:-$HOME/.local/bin}"
            ;;
        'Darwin')
            TARGET_TRIPLE="x86_64-apple-darwin"
            if [ "$VERSION" == "latest" ]; then
                VERSION=$(curl --silent "https://api.github.com/repos/stencila/stencila/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
            fi
            INSTALL_PATH="${2:-/usr/local/bin}"
            ;;
    esac
    
    echo "Downloading stencila $VERSION for platform $TARGET_TRIPLE"
    curl -sL https://github.com/stencila/stencila/releases/download/$VERSION/stencila-$VERSION-$TARGET_TRIPLE.tar.gz | tar xvz
    
    echo "Installing stencila in $INSTALL_PATH"
    mv stencila* $INSTALL_PATH
    
    echo "Successfully installed stencila CLI"
else
    echo "Sorry, I don't know how to install on this operating system, please see https://github.com/stencila/stencila#readme"
fi
