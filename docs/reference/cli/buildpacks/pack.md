<!-- Generated from doc comments in Rust. Do not edit. -->

# `pack`: Create a container image for a working directory

## Usage

```sh
stencila buildpacks pack [options] [path]
```

If the directory has a `Dockerfile` (or `Containerfile`) then the image will be built directly from that. Otherwise, the image will be built using using [`pack`](https://buildpacks.io/docs/tools/pack/) and the Stencila `builder` container image which include the buildpacks listed at `stencila buildpacks list`.

Of course, you can use either `docker` or `pack` directly. This command just provides a convenient means of testing Stencila's image building logic locally an is mainly intended for developers.


## Arguments

| Name | Description |
| --- | --- |
| `path` | The working directory (defaults to the current directory) |


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