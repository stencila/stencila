<!-- Generated from doc comments in Rust. Do not edit. -->

# `install`: Install a binary

## Usage

```sh
stencila binaries install [options] <name> [semver]
```




## Arguments

| Name | Description |
| --- | --- |
| `name` | The name of the binary (must be a registered binary name) |
| `semver` | The semantic version requirement (the most latest version meeting the requirement will be installed; defaults to the latest version) |

## Options

| Name | Description |
| --- | --- |
| `--dest -d <dest>` | The directory to install in (defaults to the Stencila `binaries` folder). |
| `--os -o <os>` | The operating system to install for (defaults to the current). One of: `macos`, `windows`, `linux` |
| `--arch -a <arch>` | The architecture to install for (defaults to the current). One of: `x86`, `x86_64`, `arm` |

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