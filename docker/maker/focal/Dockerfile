# Builds the `stencila` CLI binary from source

FROM ubuntu:focal

ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
 && apt-get install -y \
      build-essential \
      cmake \
      curl \
      git \
      libssl-dev \
      pkg-config \
 && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN curl -SL https://deb.nodesource.com/setup_16.x | bash - \
 && apt-get install -y nodejs

ARG STENCILA_VERSION=HEAD
RUN mkdir stencila \
 && curl -sS -L https://github.com/stencila/stencila/archive/$STENCILA_VERSION.tar.gz \
  | tar --strip-components=1 -C /stencila -xz
WORKDIR stencila

RUN cargo install cargo-strip

RUN make -C rust build
