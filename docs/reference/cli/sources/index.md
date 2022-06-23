---
parts:
  - list
  - show
  - add
  - remove
  - import
  - start
  - stop
  - run
---

<!-- Generated from doc comments in Rust. Do not edit. -->

# `sources`: Manage and use project sources

## Usage

```sh
stencila sources [options] <subcommand>
```

## Subcommands

| Name               | Description                                           |
| ------------------ | ----------------------------------------------------- |
| [`list`](list)     | List the sources for a project                        |
| [`show`](show)     | Show a source for a project                           |
| [`add`](add)       | Add a source to a project                             |
| [`remove`](remove) | Remove a source from a project                        |
| [`import`](import) | Import one or all of a project's sources              |
| [`start`](start)   | Start cron and watch tasks for a project's sources    |
| [`stop`](stop)     | Stop any cron and watch tasks for a project's sources |
| [`run`](run)       | Run cron and watch tasks for a project's sources      |
| `help`             | Print help information                                |

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
