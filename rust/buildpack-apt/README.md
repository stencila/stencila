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

## Options

### Turn of use of repository mirrors

By default the Ubuntu `mirror://` protocol is used to speed up downloads of debs by using repositories that are closest to you. See https://launchpad.net/ubuntu/+archivemirrors or http://mirrors.ubuntu.com/ for a full list of mirrors by country, and http://mirrors.ubuntu.com/mirrors.txt for a geo-resolved list of mirrors near you.

To turn mirroring off, and use default system repositories e.g. http://archive.ubuntu.com/, set the `STENCILA_APT_MIRRORS` environment variable to `no` ie. `STENCILA_APT_MIRRORS=no`.
