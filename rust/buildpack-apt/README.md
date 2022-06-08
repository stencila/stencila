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

### Turn off use of repository mirrors

By default, the Ubuntu `mirror://` protocol is used to speed up downloads of debs by using repositories that are closest to you. See https://launchpad.net/ubuntu/+archivemirrors or http://mirrors.ubuntu.com/ for a full list of mirrors by country, and http://mirrors.ubuntu.com/mirrors.txt for a geo-resolved list of mirrors near you.

To turn mirroring off, and use default system repositories e.g. http://archive.ubuntu.com/, add the following line to the `Aptfile` or set the `STENCILA_APT_MIRRORS` environment variable to `no`.

```
:mirrors: no
```

### Turn off removal of unused repositories

By default, this buildpack attempts to create a "clean" layer - any packages that are not specified in the `Aptfile` will be removed from the layer. This is useful for reproducibility because it reduces the risk that your code is depending on a package that was once added to the `Aptfile` but has subsequently been removed.

To turn cleaning off, and allow unspecified packages to remain in the layer, add the following line to the `Aptfile` or set the `STENCILA_APT_CLEAN` environment variable to `no`.

```
:clean: no
```
