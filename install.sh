#!/usr/bin/env bash

# A script to download and install the latest (or specified) version
# of the Stencila CLI on MacOS or Linux
#  
# First arg:  desired version e.g. v2.1.0
#             defaults to latest
# Second arg: desired install location, e.g. /bin
#             defaults to /usr/local/bin on MacOS, to ~/.local/bin on Linux

set -e

VERSION="${1:-latest}"

OS=$(uname)
if [ "${OS}" = "Linux" ] || [ "${OS}" = "Darwin" ]; then
    case "${OS}" in
        'Linux')
            TARGET_TRIPLE="x86_64-unknown-linux-gnu"
            if [ "$VERSION" = "latest" ]; then
                VERSION=$(curl --silent "https://api.github.com/repos/stencila/stencila/releases/latest" | grep -Po '"tag_name": "\K.*?(?=")')
            fi
            INSTALL_PATH="${2:-$HOME/.local/bin}"
            ;;
        'Darwin')
            TARGET_TRIPLE="x86_64-apple-darwin"
            if [ "$VERSION" = "latest" ]; then
                VERSION=$(curl --silent "https://api.github.com/repos/stencila/stencila/releases/latest" | grep -o '"tag_name": "[^"]*"' | cut -d '"' -f 4 )
            fi
            INSTALL_PATH="${2:-/usr/local/bin}"
            ;;
    esac

    if [ "$VERSION" = "" ]; then
        echo "Unable to determine the version of latest release"
        exit 1
    fi
    
    echo "Downloading Stencila CLI $VERSION for platform $TARGET_TRIPLE"
    curl -sL "https://github.com/stencila/stencila/releases/download/$VERSION/cli-$VERSION-$TARGET_TRIPLE.tar.gz" | tar xz -O "cli-$VERSION-$TARGET_TRIPLE/stencila" > stencila || {
        echo 
        echo "There was an error downloading cli-$VERSION-$TARGET_TRIPLE.tar.gz"
        echo "It may be that binaries are not available for the latest release yet."
        echo "Please wait, or try a previous version at https://github.com/stencila/stencila/releases."
        echo
        exit 1
    }
    
    echo "Installing stencila in $INSTALL_PATH"
    mkdir -p "$INSTALL_PATH"
    mv stencila* "$INSTALL_PATH"
    chmod +x "$INSTALL_PATH/stencila"
    
    echo "Successfully installed stencila CLI"
else
    echo "Sorry, I don't know how to install on this operating system, please see https://github.com/stencila/stencila#readme"
fi
