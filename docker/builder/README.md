# 🐳 `stencila/builder`

**A Docker image for building the Stencila command line tool**

## Purpose

This image builds the `stencila` command line tool within containers so that the binary can be copied to other containers using the same base image (e.g. `ubuntu:focal`) without bloating them with build dependencies while maintaining version compatibility with shared libraries such as `libc` and `libssl`.

## Usage

Build this image, e.g.

```sh
docker build --tag stencila/builder:focal builder/focal
```

This defaults to building the current `HEAD` of the repository but you can use the Docker build argument `STENCILA_VERSION` to build a tag, branch or commit id.

Then, in another image copy the `stencila` binary from the `/stencila/target/release/` folder of the `builder` image,

```Dockerfile
COPY --from=stencila/builder:focal /stencila/target/release/stencila /usr/local/bin/
```
