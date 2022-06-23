---
parts:
  - list
  - show
  - create
  - members
  - plan
  - usage
  - activity
---

<!-- Generated from doc comments in Rust. Do not edit. -->

# `orgs`: Manage organizations

## Usage

```sh
stencila orgs [options] <subcommand>
```

Use this command to list your Stencila organizations, manage their members, plans and usage, and view activity logs.

## Subcommands

| Name                   | Description                                          |
| ---------------------- | ---------------------------------------------------- |
| [`list`](list)         | List organizations                                   |
| [`show`](show)         | Show details of a org                                |
| [`create`](create)     | Create an organization                               |
| [`members`](members)   | Manage org members                                   |
| [`plan`](plan)         | Manage organization's plan and extras settings       |
| [`usage`](usage)       | View an organization's resource usage against quotas |
| [`activity`](activity) | Get activity logs for an organization                |
| `help`                 | Print help information                               |

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
