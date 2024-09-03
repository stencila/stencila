# Use the official Node.js image to build web clients and components
FROM node:latest AS web

# Set the working directory to /build and copy files into it
WORKDIR /build
COPY . /build

# Build the web bundle to include in the binary
RUN npm ci
RUN cd ts && npm run build
RUN cd web && npm run build



# Use the official Rust image to build CLI
FROM rust:latest AS cli

# Set the working directory to /build and copy files into it
WORKDIR /build
COPY . /build

# Copy the web build into the expected location
COPY --from=web /build/web/dist /build/web/dist

# Build the binary
RUN cargo build --bin stencila --release



# Use small distroless image
FROM gcr.io/distroless/cc-debian12

# Copy the binary into the image from the CLI build stage
COPY --from=cli /build/target/release/stencila /

# Expose the default Stencila port
EXPOSE 9000

# Set the entry point to run the Stencila binary and the default command
ENTRYPOINT ["/stencila"]
CMD []
