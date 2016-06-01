#!/usr/bin/env bash

pacman --needed --noconfirm -Sy pacman-mirrors
pacman --noconfirm -Sy
pacman --noconfirm -S base-devel cmake gcc git mingw-w64-x86_64-python2-pip msys2-devel unzip zip
pacman --noconfirm -S openssl libssh2
