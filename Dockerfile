# Use the official Node.js image to build web clients and components
# for embedding in the CLI binary
FROM node:22 AS web

# Set the working directory to /build and copy files into it
WORKDIR /build
COPY . /build

# Build the web bundle to include in the binary
RUN npm ci
RUN cd ts && npm run build
RUN cd web && npm run build



# Use the official Rust image to build CLI binary
FROM rust:1.88 AS cli

# Set the working directory to /build and copy files into it
WORKDIR /build
COPY . /build

# Copy the web build into the expected location
COPY --from=web /build/web/dist /build/web/dist

# Build the binary
RUN apt-get update && apt-get install cmake -y
RUN cargo build --bin stencila --release



# Use Ubuntu version compatible with CLI
FROM ubuntu:24.04

# Copy the binary into the image from the CLI build stage
COPY --from=cli /build/target/release/stencila /

# Expose the default Stencila port
EXPOSE 9000

# Set the entry point to run the Stencila binary and the default command
ENTRYPOINT ["/stencila"]
CMD []
