# 📦 Stencila buildpack for Dockerfiles

This is not a Cloud Native Buildpack (e.g. it lacks a `detect` or `build` executables). However, it uses the same API (e.g. has a `buildpack.toml`) so that, for example, it appears in the list at `stencila buildpacks list`. We never point an external CNB platform, such as Pack, at this buildpack. Instead, in the `buildpacks` crate, we run its `detect` method before any other buildpacks and build an
image from the Dockerfile if it passes.

## Detection

Matches against a project that has a `Dockerfile` or `Containerfile` in its root folder.

## Building container images

Currently uses `podman`, rather than `docker`, to build the image because the former runs in userspace and is thus more secure. In the future this may first attempt to use `podman` if already installed, fallback to `docker` if installed, fallback to installing `podman`.
