---
parts:
  - list
  - show
  - create
  - clone
  - pull
  - push
  - delete
  - members
---


<!-- Generated from doc comments in Rust. Do not edit. -->

# `projects`: Manage projects

## Usage

```sh
stencila projects [options] <subcommand>
```

Use this command to list your Stencila projects, inspect and update details for individual projects, and to manage project sources, members, and deployments etc.

## Subcommands

| Name | Description |
| --- | --- |
| [`list`](list.md) | List projects |
| [`show`](show.md) | Show details of a project |
| [`create`](create.md) | Create a project |
| [`clone`](clone.md) | Clone a project |
| [`pull`](pull.md) | Pull the current project |
| [`push`](push.md) | Push the current project |
| [`delete`](delete.md) | Delete a project |
| [`members`](members/README.md) | Manage project members |
| `help` | Print help information |



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