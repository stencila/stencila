# 📦 Stencila Buildpack for `apt` packages

**A [Cloud Native Buildpack](https://buildpacks.io/) that installs `apt`-based packages**

This buildpack is a Rust port of [`heroku-buildpack-apt`](https://github.com/heroku/heroku-buildpack-apt) using [`libcnb.rs`](https://github.com/Malax/libcnb.rs) and Stencila's own utilities for buildpacks.

## Detection

Matches against a project that has an `Aptfile` in its root folder.

## Apt packages installed

The packages to be installed are listed in the `Aptfile`,

```sh
# List packages on separate lines
libexample-dev
another

# Or include links to .deb files
https://downloads.example.com/example.deb

# Add custom apt repos (only required if using packages outside of the standard Ubuntu APT repositories)
:repo: deb https://apt.example.com/ bionic main

# If necessary, you can add options for the repo e.g.
:repo: deb [trusted=yes] https://apt.example.com/ bionic main
```
