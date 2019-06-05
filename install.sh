#!/usr/bin/env bash

# A script to download and install the latest version

OS=$(uname)
APPLICATION_NAME=stencila
REPOSITORY_NAME=stencila

if [[ "${OS}" == "Linux" || "${OS}" == "Darwin" ]]; then
    case "${OS}" in
        'Linux')
            PLATFORM="linux-x64"
            if [ -z "$1" ]; then
                VERSION=$(curl --silent "https://api.github.com/repos/stencila/${REPOSITORY_NAME}/releases/latest" | grep -Po '"tag_name": "\K.*?(?=")')
            else
                VERSION=$1
            fi
            INSTALL_PATH="${HOME}/.local/bin"
            ;;
        'Darwin')
            PLATFORM="macos-x64"
            if [ -z "$1" ]; then
                VERSION=$(curl --silent "https://api.github.com/repos/stencila/${REPOSITORY_NAME}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
            else
                VERSION=$1
            fi
            INSTALL_PATH="/usr/local/bin"
            ;;
    esac
    
    echo "Downloading ${APPLICATION_NAME} ${VERSION}"
    curl -Lo /tmp/${APPLICATION_NAME}.tar.gz https://github.com/stencila/${REPOSITORY_NAME}/releases/download/${VERSION}/${APPLICATION_NAME}-${PLATFORM}.tar.gz
    tar xvf /tmp/${APPLICATION_NAME}.tar.gz
    rm -f /tmp/${APPLICATION_NAME}.tar.gz
    
    echo "Installing ${APPLICATION_NAME} to ${INSTALL_PATH}/${APPLICATION_NAME}-${VERSION}"
    mkdir -p ${INSTALL_PATH}/${APPLICATION_NAME}-${VERSION}
    mv -f ${APPLICATION_NAME} ${INSTALL_PATH}/${APPLICATION_NAME}-${VERSION}
    # Unpack `node_modules` etc into the ${INSTALL_PATH}/${APPLICATION_NAME}-${VERSION}
    ${INSTALL_PATH}/${APPLICATION_NAME}-${VERSION}/${APPLICATION_NAME} setup
    
    echo "Pointing ${APPLICATION_NAME} to ${INSTALL_PATH}/${APPLICATION_NAME}-${VERSION}/${APPLICATION_NAME}"
    ln -sf ${APPLICATION_NAME}-${VERSION}/${APPLICATION_NAME} ${INSTALL_PATH}/${APPLICATION_NAME}
else
    echo "Sorry, I don't know how to install on this OS, please see https://github.com/stencila/${REPOSITORY_NAME}#install"
fi
