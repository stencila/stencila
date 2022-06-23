<!-- Generated from doc comments in Rust. Do not edit. -->

# `show`: Show information on a binary

## Usage

```sh
stencila binaries show [options] <name> [semver]
```

Searches for the binary on your path and in Stencila's "binaries" folder for versions that are installed. Use the `semver` argument to show the latest version that meets the semantic version requirement.

This command should find any binary that is on your PATH (i.e. including those not in the `stencila binaries installable` list).

## Arguments

| Name     | Description                                             |
| -------- | ------------------------------------------------------- |
| `name`   | The name of the binary e.g. pandoc                      |
| `semver` | The semantic version requirement for the binary e.g. >2 |

## Global options

| Name                        | Description                                                                                                                                          |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--help`                    | Print help information.                                                                                                                              |
| `--version`                 | Print version information.                                                                                                                           |
| `--as <format>`             | Format to display output values (if possible).                                                                                                       |
| `--json`                    | Display output values as JSON (alias for `--as json`).                                                                                               |
| `--yaml`                    | Display output values as YAML (alias for `--as yaml`).                                                                                               |
| `--md`                      | Display output values as Markdown if possible (alias for `--as md`).                                                                                 |
| `--interact -i`             | Enter interactive mode (with any command and options as the prefix).                                                                                 |
| `--debug`                   | Print debug level log events and additional diagnostics. Equivalent to setting `--log-level=debug` and `--log-format=detail` and overrides the both. |
| `--log-level <log-level>`   | The minimum log level to print. One of: `trace`, `debug`, `info`, `warn`, `error`, `never`                                                           |
| `--log-format <log-format>` | The format to print log events. One of: `simple`, `detail`, `json`                                                                                   |
