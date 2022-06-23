<!-- Generated from doc comments in Rust. Do not edit. -->

# `add`: Add a user to a team

## Usage

```sh
stencila teams members add [options] <team> <user>
```

Use this command to add a user to a team.

## Arguments

| Name   | Description                    |
| ------ | ------------------------------ |
| `team` | The id of the team             |
| `user` | The username or id of the user |

## Options

| Name             | Description                                                                                     |
| ---------------- | ----------------------------------------------------------------------------------------------- |
| `--org -o <org>` | The id of the org. If this option is not supplied, Stencila will use your default organization. |

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
