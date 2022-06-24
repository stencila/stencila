<!-- Generated from doc comments in Rust. Do not edit. -->

# `plan`: Show the build plan for a working directory

## Usage

```sh
stencila buildpacks plan [options] [path]
```




## Arguments

| Name | Description |
| --- | --- |
| `path` | The working directory (defaults to the current directory) |

## Options

| Name | Description |
| --- | --- |
| `--all -a` | Show all buildpacks, including those that failed to match against the working directory. |
| `--cnb` | Simulate plan on a CNB platform such as Pack. |

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