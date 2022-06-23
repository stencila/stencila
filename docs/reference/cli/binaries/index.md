---
parts:
  - list
  - show
  - versions
  - install
  - uninstall
  - run
---

<!-- Generated from doc comments in Rust. Do not edit. -->

# `binaries`: Manage and use helper binaries

## Usage

```sh
stencila binaries [options] <subcommand>
```

## Subcommands

| Name                     | Description                                          |
| ------------------------ | ---------------------------------------------------- |
| [`list`](list)           | List binaries that can be installed using Stencila   |
| [`show`](show)           | Show information on a binary                         |
| [`versions`](versions)   | List the versions that can be installed for a binary |
| [`install`](install)     | Install a binary                                     |
| [`uninstall`](uninstall) | Uninstall a binary                                   |
| [`run`](run)             | Run a command using a binary                         |
| `help`                   | Print help information                               |

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
