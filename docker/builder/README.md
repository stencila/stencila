# 🐳 `stencila/builder`

**A Docker image for building the Stencila command line tool**

## Purpose

This image builds the `stencila` command line tool within containers so that the binary can be copied to other containers using the same base image (e.g. `ubuntu:focal`) without bloating them with build dependencies while maintaining version compatibility with shared libraries such as `libc` and `libssl`.

This image should not be confused with the [`stencila/build`](../stacks/) image (which is part of the Stencila's Cloud Native Buildpack ['stack'](https://buildpacks.io/docs/concepts/components/stack/)).

## Usage

This image is built for each release of the `stencila` CLI. It is built before other images that depend upon it such as, [`stencila/stencila`](../stencila/) and [`stencila/buildpacks`](../buildpacks/).
