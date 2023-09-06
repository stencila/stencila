# Use the official Rust image for the build
FROM rust:latest AS build

# Set the working directory to /build
WORKDIR /build

# Copy the project files into the build image
COPY . /build

# Build the binary
RUN cargo build --release

# Use small distroless image
FROM gcr.io/distroless/cc-debian12

# Copy the binary into the image from the build stage
COPY --from=build /build/target/release/stencila /

# Expose the default Stencila port
EXPOSE 9000

# Set the entry point to run the Stencila binary and the default command
ENTRYPOINT ["/stencila"]
CMD []
