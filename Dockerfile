FROM ubuntu:20.04

ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
 && apt-get install -y \
      curl \
 && curl -sL https://deb.nodesource.com/setup_14.x | bash - \
 && apt-get update \
 && apt-get install -y \
      libcurl4-openssl-dev \
      libfontconfig1-dev \
      libfreetype6-dev \
      libgit2-dev \
      libjpeg-dev \
      libpng-dev \
      libssh2-1-dev \
      libssl-dev \
      libtiff5-dev \
      libxml2-dev \
      nodejs \
      pandoc \
      python3 \
      python3-pip \
      r-base \
 && rm -rf /var/lib/apt/lists/*

COPY . /code
WORKDIR /code

RUN make setup
