<!-- Generated from doc comments in Rust. Do not edit. -->

# `versions`: List the versions that can be installed for a binary

## Usage

```sh
stencila binaries versions [options] <name>
```

## Arguments

| Name   | Description                        |
| ------ | ---------------------------------- |
| `name` | The name of the binary e.g. pandoc |

## Options

| Name              | Description                                                                                                                                                |
| ----------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--os -o <os>`    | The operating system to list versions for (defaults to the current). One of: `macos`, `windows`, `linux`                                                   |
| `--write <write>` | The Rust file to write the the versions to. This option is usually only used by developers of Stencila to update the static list of versions for a binary. |

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
