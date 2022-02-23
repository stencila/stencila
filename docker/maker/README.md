# 🐳 `stencila/maker`

**A Docker image for making the Stencila command line tool**

## Purpose

This image builds the `stencila` command line tool within containers so that the binary can be copied to other containers using the same base image (e.g. `ubuntu:focal`) without bloating them with build dependencies while maintaining version compatibility with shared libraries such as `libc` and `libssl`.

This image should not be confused with the [`stencila/builder`](../builder/) (which is part of the Stencila's Cloud Native Buildpack distribution.

## Usage

This image is built for each release of the `stencila` CLI. It is built before other images that depend upon it such as, [`stencila/stencila`](../stencila/) and [`stencila/buildpacks`](../buildpacks/).

## Building

The build argument `STENCILA_VERSION` can be a tag, branch or commit id.
