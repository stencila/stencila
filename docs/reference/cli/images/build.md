<!-- Generated from doc comments in Rust. Do not edit. -->

# `build`: Build an image

## Usage

```sh
stencila images build [options] [dir]
```




## Arguments

| Name | Description |
| --- | --- |
| `dir` | The directory to build an image for |

## Options

| Name | Description |
| --- | --- |
| `--from -f <from>` | The base image to build from. Equivalent to the `FROM` directive in a Dockerfile. Defaults to the `STENCILA_IMAGE_REF` (i.e. the current image, if Stencila is running in a container), falling back to `stencila/stencila:nano` if not. Must be a valid image reference e.g. `docker.io/library/ubuntu:22.04`, `ubuntu:22.04`, `ubuntu`. |
| `--tag -t <tag>` | The registry, repository and tag to push to. Equivalent to the `--tag` option to Docker build. Must be a valid image reference e.g. `localhost:5000/my-project`. Defaults to the name of the directory plus a hash of its path (to maintain uniqueness). |
| `--layer-format <layer-format>` | The format to use for image layers. The Open Container Image spec allows for layers to be in several formats. The default `tar+zstd` format provides performance benefits over the others but may not be supported by older versions of some container tools. One of: `tar`, `tar+gzip`, `tgz`, `tar+zstd`, `tzs`. Default: tar+zstd |
| `--manifest-format <manifest-format>` | The format to use for the image manifest. Defaults to `oci`, however for compatibility with older version of some image registries it may be necessary to use `v2s2` (Docker Version 2 Schema 2). One of: `oci`, `v2s2`. Default: oci |
| `--no-workspace` | Do not create a layer for the workspace (i.e. ignore the `<dir>` argument). Mainly if you simply want to apply add `.env` and/or `.labels` files to the `--from` image and give it a new `--tag`. |
| `--no-buildpacks` | Do not run any buildpacks. Mainly useful during development for testing the writing of images, without waiting for potentially long buildpack build times. |
| `--no-diffs` | Do not calculate a changeset for each layer directory and instead represent them in their entirety. . The default behavior is to take snapshots of directories before and after the buildpacks build and generate a changeset representing the difference. This replicates the behavior of Dockerfile `RUN` directives. This option instead forces the layer to represent the entire directory after the build. |
| `--no-write` | Do not write the image to disk after building it. Mainly useful during development for testing that the image can be built without waiting for the base image manifest to be fetched or snapshot changesets to be calculated. |
| `--no-push` | Do not push the image to the repository after writing it. Mainly useful during development for testing that the image can be built without waiting for it to be pushed to the registry. |
| `--layers-dir <layers-dir>` | The directory where buildpacks build layers and which will are written into the image. Defaults to a `/layers` (the usual in CNB images) or `<dir>/.stencila/layers` (the fallback for local builds). |
| `--layout-dir <layout-dir>` | The directory to write the image to. Defaults to a temporary directory. Use this option if you want to inspect the contents of the image directory. e.g. `stencila images build ... --no-build --no-push --layout-dir temp`. If the `layout_dir` already exists, its contents are deleted - so use with care!. |
| `--layout-complete` | Whether the layout directory should be written with all layers. As an optimization, base layers are only written to the layout directory as needed (i.e. when a registry does not have the layer yet). Use this option to ensure that layout directory includes all layers  (e.g. when wanting to run the image locally). |

## Global options

| Name | Description |
| --- | --- |
| `--help` | Print help information. |
| `--version` | Print version information. |
| `--as <format>` | Format to display output values (if possible). |
| `--json` | Display output values as JSON (alias for `--as json`). |
| `--yaml` | Display output values as YAML (alias for `--as yaml`). |
| `--md` | Display output values as Markdown if possible (alias for `--as md`). |
| `--interact -i` | Enter interactive mode (with any command and options as the prefix). |
| `--debug` | Print debug level log events and additional diagnostics. Equivalent to setting `--log-level=debug` and `--log-format=detail` and overrides the both. |
| `--log-level <log-level>` | The minimum log level to print. One of: `trace`, `debug`, `info`, `warn`, `error`, `never` |
| `--log-format <log-format>` | The format to print log events. One of: `simple`, `detail`, `json` |