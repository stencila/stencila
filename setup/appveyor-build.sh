#!/usr/bin/env bash

# Install MSYS2 packages
# The `base-devel` package is aleady installed on Appveyor but if
# setting up your own MSYS2 environment you'll need to include that too
pacman --needed --noconfirm -Sy pacman-mirrors
pacman --noconfirm -Sy
pacman --noconfirm -S python python2
pacman --noconfirm -S mingw-w64-x86_64-toolchain mingw-w64-x86_64-cmake mingw-w64-x86_64-python2-pip mingw-w64-x86_64-python3-pip
pacman --noconfirm -S mingw-w64-x86_64-openssl mingw-w64-x86_64-libssh2 mingw-w64-x86_64-zlib
