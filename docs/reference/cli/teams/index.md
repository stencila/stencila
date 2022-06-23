---
parts:
  - list
  - create
  - delete
  - members
---

<!-- Generated from doc comments in Rust. Do not edit. -->

# `teams`: Manage teams

## Usage

```sh
stencila teams [options] <subcommand>
```

Use this command to list your Stencila teams, inspect and update details for individual teams, and to manage team sources, members, and deployments etc.

## Subcommands

| Name                 | Description            |
| -------------------- | ---------------------- |
| [`list`](list)       | List teams             |
| [`create`](create)   | Create a team          |
| [`delete`](delete)   | Delete a team          |
| [`members`](members) | Manage team members    |
| `help`               | Print help information |

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
