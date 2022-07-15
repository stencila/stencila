<!-- Generated from doc comments in Rust. Do not edit. -->

# `save`: Save a container as an image layer

## Usage

```sh
stencila images save [options]
```





## Options

| Name | Description |
| --- | --- |
| `--reference -r <reference>` | The registry, repository and tag to give the image. Equivalent to the `--tag` option to Docker build. Must be a valid image reference e.g. `localhost:5000/my-project`. |
| `--base -b <base>` | The base image to build from. Equivalent to the `FROM` directive in a Dockerfile. Defaults to the `STENCILA_IMAGE_REF` (i.e. the current image, if Stencila is running in a container), falling back to `stencila/stencila:nano` if not. Must be a valid image reference e.g. `docker.io/library/ubuntu:22.04`, `ubuntu:22.04`, `ubuntu`. |
| `--snapshot -s <snapshot>` | The path of the snapshot to use as the base for the layer changeset. Defaults to `/root.snap` which is the default path for a snapshot generated from the `snap` command. |
| `--layer-format <layer-format>` | The format to use for image layers. The Open Container Image spec allows for layers to be in several formats. The default `tar+zstd` format provides performance benefits over the others but may not be supported by older versions of some container tools. One of: `tar`, `tar+gzip`, `tgz`, `tar+zstd`, `tzs`. Default: tar+zstd |
| `--manifest-format <manifest-format>` | The format to use for the image manifest. Defaults to `oci`, however for compatibility with older version of some image registries it may be necessary to use `v2s2` (Docker Version 2 Schema 2). One of: `oci`, `v2s2`. Default: oci |
| `--no-push` | Do not push the image to the repository after writing it. Mainly useful during development for testing that the image can be built without having to have a registry to push it to. |

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