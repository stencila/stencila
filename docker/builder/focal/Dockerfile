# Builds the `stencila` binary from source

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

ARG STENCILA_VERSION
RUN git clone --branch v${STENCILA_VERSION} --depth 1 https://github.com/stencila/stencila
WORKDIR stencila

RUN cargo install cargo-strip
RUN make -C rust build
