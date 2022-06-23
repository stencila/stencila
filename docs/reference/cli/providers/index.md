---
parts:
  - list
  - show
  - detect
  - enrich
  - import
  - export
  - cron
---

<!-- Generated from doc comments in Rust. Do not edit. -->

# `providers`: Manage and use source providers

## Usage

```sh
stencila providers [options] <subcommand>
```

## Subcommands

| Name               | Description                                                          |
| ------------------ | -------------------------------------------------------------------- |
| [`list`](list)     | List the providers that are available                                |
| [`show`](show)     | Show the specifications of a provider                                |
| [`detect`](detect) | Detect nodes within a file or string                                 |
| [`enrich`](enrich) | Enrich nodes within a file or string                                 |
| [`import`](import) | Import content from a remote source to a local path                  |
| [`export`](export) | Export content from a local path to a remote source                  |
| [`cron`](cron)     | Schedule import and/or export between remote source and a local path |
| `help`             | Print help information                                               |

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
