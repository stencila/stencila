FROM ubuntu:20.04

ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
 && apt-get install -y \
      libcurl4-openssl-dev \
      libfontconfig1-dev \
      libfreetype6-dev \
      libgit2-dev \
      libjpeg-dev \
      libpng-dev \
      libssh2-1-dev \
      libtiff5-dev \
      libxml2-dev \
      nodejs \
      npm \
      python3 \
      python3-pip \
      r-base \
 && rm -rf /var/lib/apt/lists/*

COPY . /code
WORKDIR /code

RUN npm install
RUN make -C py setup

# WIP: Skipping for now, until all system deps are determined and installed.
# RUN make -C r setup
