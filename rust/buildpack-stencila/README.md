# 📦 Stencila buildpack for Stencila CLI

This buildpack installs the version of Stencila specified in the `.tool-versions` file (or the latest version, if no version is specified). It is intended for projects that are not using one of the Stencila [stacks](https://github.com/stencila/stencila/tree/HEAD/docker/stacks) (which already have Stencila installed).

At present this buildpack only installs the Stencila CLI. In the future it may also install some of the binaries that Stencila depends upon for some functionality e.g. `pandoc` and `chromium`.
